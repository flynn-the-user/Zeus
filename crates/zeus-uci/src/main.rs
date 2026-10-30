//! UCI loop for Zeus, now with iterative deepening + time management.
//!
//! Reference: `Reference/protocols/UCI_Protocol.md`. Search now runs on a
//! background thread (spawned in `handle_go`) so `stop` can set the shared
//! abort flag and the main loop stays responsive to `isready` while
//! thinking, per the spec: "isready ... can be sent also when the engine
//! is calculating in which case the engine should also immediately answer
//! with readyok without stopping the search."
mod time;

use chess::{Board, ChessMove};
use std::io::{self, BufRead, Write};
use std::str::FromStr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::Instant;
use time::GoClock;
use zeus_engine::search::{search_iterative, IterationInfo, MAX_DEPTH};
use zeus_engine::tt::TranspositionTable;

const ENGINE_NAME: &str = "Zeus";
const ENGINE_AUTHOR: &str = "Zeus project (see ROADMAP.md / CITATIONS.md)";
/// "the engine should use a very small hash first as default" -
/// Reference/protocols/UCI_Protocol.md's Hash option description.
const DEFAULT_HASH_MB: usize = 16;

struct EngineState {
    board: Board,
    stop: Option<Arc<AtomicBool>>,
    search_thread: Option<JoinHandle<()>>,
    /// Ablation-study toggle for `zeus-bench` (see
    /// `crates/zeus-engine/src/search.rs`'s module doc comment) - a custom
    /// engine option in the same style as the "Nullmove"/"Selectivity"
    /// examples in `Reference/protocols/UCI_Protocol.md`'s own spec.
    use_quiescence: bool,
    /// Persists across `go` calls within a game (cleared on `ucinewgame`),
    /// per `Reference/search/Transposition_Table.md`. `Arc<Mutex<_>>`
    /// because the search thread needs it too; only one search ever runs
    /// at a time (`join_search` is called before starting another), so
    /// the lock is taken once per `go` and held for that whole search,
    /// not per-node.
    tt: Arc<Mutex<TranspositionTable>>,
}

impl EngineState {
    fn join_search(&mut self) {
        if let Some(handle) = self.search_thread.take() {
            let _ = handle.join();
        }
        self.stop = None;
    }
}

fn main() {
    let stdin = io::stdin();
    let mut state = EngineState {
        board: Board::default(),
        stop: None,
        search_thread: None,
        use_quiescence: true,
        tt: Arc::new(Mutex::new(TranspositionTable::new(DEFAULT_HASH_MB))),
    };

    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => break,
        };
        let tokens: Vec<&str> = line.split_whitespace().collect();
        let Some(&cmd) = tokens.first() else {
            continue;
        };

        match cmd {
            "uci" => {
                println!("id name {ENGINE_NAME}");
                println!("id author {ENGINE_AUTHOR}");
                println!("option name UseQuiescence type check default true");
                println!(
                    "option name Hash type spin default {DEFAULT_HASH_MB} min 1 max 4096"
                );
                println!("uciok");
            }
            "isready" => println!("readyok"),
            "setoption" => handle_setoption(&mut state, &tokens[1..]),
            "ucinewgame" => {
                state.join_search();
                state.board = Board::default();
                // New game: stale entries from a different game shouldn't
                // be reused (Reference/search/Transposition_Table.md's
                // "age" field is this project's substitute for proper
                // aging - see that file).
                state.tt.lock().unwrap().clear();
            }
            "position" => {
                // By protocol a new "position" only arrives after the
                // previous search's "bestmove", but join defensively.
                state.join_search();
                handle_position(&mut state, &tokens[1..]);
            }
            "go" => {
                state.join_search();
                handle_go(&mut state, &tokens[1..]);
            }
            "stop" => {
                if let Some(stop) = &state.stop {
                    stop.store(true, Ordering::Relaxed);
                }
                state.join_search();
            }
            "quit" => {
                if let Some(stop) = &state.stop {
                    stop.store(true, Ordering::Relaxed);
                }
                state.join_search();
                break;
            }
            // Unknown/unsupported command: ignore per Reference/protocols/UCI_Protocol.md.
            _ => {}
        }
        let _ = io::stdout().flush();
    }
}

fn handle_position(state: &mut EngineState, args: &[&str]) {
    let (board, idx) = if args.first() == Some(&"startpos") {
        (Board::default(), 1)
    } else if args.first() == Some(&"fen") {
        let fen_tokens: Vec<&str> = args[1..]
            .iter()
            .take_while(|&&t| t != "moves")
            .copied()
            .collect();
        let idx = 1 + fen_tokens.len();
        match Board::from_str(&fen_tokens.join(" ")) {
            Ok(b) => (b, idx),
            Err(_) => return,
        }
    } else {
        return;
    };

    let mut board = board;
    if args.get(idx) == Some(&"moves") {
        for mv_str in &args[idx + 1..] {
            match ChessMove::from_str(mv_str) {
                Ok(mv) => board = board.make_move_new(mv),
                Err(_) => break,
            }
        }
    }
    state.board = board;
}

/// `setoption name <id> [value <x>]`, per
/// `Reference/protocols/UCI_Protocol.md`. Only `UseQuiescence` is
/// recognized; anything else is ignored, per that same spec ("if the
/// engine or the GUI receives an unknown command or token it should just
/// ignore it").
fn handle_setoption(state: &mut EngineState, args: &[&str]) {
    if args.first() != Some(&"name") {
        return;
    }
    let Some(value_idx) = args.iter().position(|&t| t == "value") else {
        return;
    };
    let name = args[1..value_idx].join(" ");
    let value = args[value_idx + 1..].join(" ");
    if name.eq_ignore_ascii_case("UseQuiescence") {
        state.use_quiescence = value.eq_ignore_ascii_case("true");
    } else if name.eq_ignore_ascii_case("Hash") {
        if let Ok(mb) = value.parse::<usize>() {
            // Resizing mid-search would be unsound (the search thread
            // holds the lock for its whole run, see EngineState::tt) -
            // by protocol this only arrives between searches anyway, but
            // join defensively first.
            state.join_search();
            *state.tt.lock().unwrap() = TranspositionTable::new(mb);
        }
    }
}

fn handle_go(state: &mut EngineState, args: &[&str]) {
    let mut clock = GoClock::default();
    let mut explicit_depth: Option<u32> = None;

    let mut i = 0;
    while i < args.len() {
        match args[i] {
            "depth" => {
                explicit_depth = args.get(i + 1).and_then(|s| s.parse().ok());
                i += 2;
            }
            "wtime" => {
                clock.wtime = args.get(i + 1).and_then(|s| s.parse().ok());
                i += 2;
            }
            "btime" => {
                clock.btime = args.get(i + 1).and_then(|s| s.parse().ok());
                i += 2;
            }
            "winc" => {
                clock.winc = args.get(i + 1).and_then(|s| s.parse().ok());
                i += 2;
            }
            "binc" => {
                clock.binc = args.get(i + 1).and_then(|s| s.parse().ok());
                i += 2;
            }
            "movetime" => {
                clock.movetime = args.get(i + 1).and_then(|s| s.parse().ok());
                i += 2;
            }
            "infinite" => {
                clock.infinite = true;
                i += 1;
            }
            _ => i += 1,
        }
    }

    let now = Instant::now();
    let white_to_move = state.board.side_to_move() == chess::Color::White;
    let (soft_deadline, hard_deadline) = time::compute_deadlines(&clock, white_to_move, now);
    let max_depth = explicit_depth.unwrap_or(MAX_DEPTH).min(MAX_DEPTH);

    let stop = Arc::new(AtomicBool::new(false));
    state.stop = Some(Arc::clone(&stop));

    let board = state.board;
    let use_quiescence = state.use_quiescence;
    let tt = Arc::clone(&state.tt);
    let search_start = now;
    state.search_thread = Some(std::thread::spawn(move || {
        let on_iteration = |info: &IterationInfo| {
            let elapsed_ms = search_start.elapsed().as_millis().max(1) as u64;
            let nps = info.nodes * 1000 / elapsed_ms;
            println!(
                "info depth {} score cp {} nodes {} time {} nps {} pv {}",
                info.depth, info.score, info.nodes, elapsed_ms, nps, info.best_move
            );
            let _ = io::stdout().flush();
        };

        let mut tt_guard = tt.lock().unwrap();
        match search_iterative(
            &board,
            stop,
            hard_deadline,
            soft_deadline,
            max_depth,
            use_quiescence,
            &mut tt_guard,
            on_iteration,
        ) {
            Some((mv, _score)) => println!("bestmove {mv}"),
            // No legal moves (checkmate/stalemate at the root) - the
            // spec's null move 0000, per Reference/protocols/UCI_Protocol.md.
            None => println!("bestmove 0000"),
        }
        let _ = io::stdout().flush();
    }));
}
