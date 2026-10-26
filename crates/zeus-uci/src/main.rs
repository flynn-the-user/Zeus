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
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::Instant;
use time::GoClock;
use zeus_engine::search::{search_iterative, IterationInfo, MAX_DEPTH};

const ENGINE_NAME: &str = "Zeus";
const ENGINE_AUTHOR: &str = "Zeus project (see ROADMAP.md / CITATIONS.md)";

struct EngineState {
    board: Board,
    stop: Option<Arc<AtomicBool>>,
    search_thread: Option<JoinHandle<()>>,
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
                println!("uciok");
            }
            "isready" => println!("readyok"),
            "ucinewgame" => {
                state.join_search();
                state.board = Board::default();
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

        match search_iterative(&board, stop, hard_deadline, soft_deadline, max_depth, on_iteration) {
            Some((mv, _score)) => println!("bestmove {mv}"),
            // No legal moves (checkmate/stalemate at the root) - the
            // spec's null move 0000, per Reference/protocols/UCI_Protocol.md.
            None => println!("bestmove 0000"),
        }
        let _ = io::stdout().flush();
    }));
}
