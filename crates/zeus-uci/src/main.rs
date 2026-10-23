//! Minimal UCI loop for Zeus v0.1.
//!
//! Reference: `Reference/protocols/UCI_Protocol.md` (the April 2006 UCI
//! specification). Implements the subset of commands listed in that file's
//! "Zeus usage" section: `uci`, `isready`, `ucinewgame`, `position`, `go`
//! (subset), `stop`, `quit`, and emits `id`, `uciok`, `readyok`, `info`,
//! `bestmove`.
//!
//! No time management yet (see `ROADMAP.md`, stage 1): `go` always searches
//! a small fixed depth and ignores `wtime`/`btime`/`winc`/`binc`/
//! `movestogo`/`movetime`. `DEFAULT_DEPTH` is deliberately shallow so this
//! is safe (fast enough) under the fast calibration time controls used by
//! `zeus-bench` even though the clock is being ignored. This is an
//! engineering placeholder, not an algorithmic choice drawn from a
//! reference — real time management is the very next roadmap stage.
use chess::{Board, ChessMove};
use std::io::{self, BufRead, Write};
use std::str::FromStr;
use zeus_engine::search::search_fixed_depth;

const ENGINE_NAME: &str = "Zeus";
const ENGINE_AUTHOR: &str = "Zeus project (see ROADMAP.md / CITATIONS.md)";
const DEFAULT_DEPTH: u32 = 4;

struct EngineState {
    board: Board,
}

fn main() {
    let stdin = io::stdin();
    let mut state = EngineState {
        board: Board::default(),
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
            "ucinewgame" => state.board = Board::default(),
            "position" => handle_position(&mut state, &tokens[1..]),
            "go" => handle_go(&state, &tokens[1..]),
            "stop" => {
                // v0.1's search is synchronous fixed-depth: by the time a
                // "stop" line could be read, the (fast, shallow) search for
                // the current "go" has already finished and "bestmove" was
                // already sent. Nothing to interrupt yet.
            }
            "quit" => break,
            // Unknown/unsupported command: ignore per Reference/protocols/UCI_Protocol.md
            // ("if the engine or the GUI receives an unknown command or
            // token it should just ignore it").
            _ => {}
        }
        let _ = io::stdout().flush();
    }
}

fn handle_position(state: &mut EngineState, args: &[&str]) {
    let (board, idx) = if args.first() == Some(&"startpos") {
        (Board::default(), 1)
    } else if args.first() == Some(&"fen") {
        // FEN can contain spaces; collect tokens until "moves" or end.
        let fen_tokens: Vec<&str> = args[1..]
            .iter()
            .take_while(|&&t| t != "moves")
            .copied()
            .collect();
        let idx = 1 + fen_tokens.len();
        match Board::from_str(&fen_tokens.join(" ")) {
            Ok(b) => (b, idx),
            Err(_) => return, // malformed input: ignore, keep previous position
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

fn handle_go(state: &EngineState, args: &[&str]) {
    let mut depth = DEFAULT_DEPTH;
    let mut i = 0;
    while i < args.len() {
        if args[i] == "depth" {
            if let Some(d) = args.get(i + 1).and_then(|s| s.parse::<u32>().ok()) {
                depth = d;
            }
            i += 2;
        } else {
            i += 1;
        }
    }

    match search_fixed_depth(&state.board, depth) {
        Some((mv, score)) => {
            println!("info depth {depth} score cp {score} pv {mv}");
            println!("bestmove {mv}");
        }
        None => {
            // No legal moves (checkmate/stalemate at the root). The spec's
            // "Move format" section defines the null move 0000 for exactly
            // this case: "A nullmove from the Engine to the GUI should be
            // sent as 0000."
            println!("bestmove 0000");
        }
    }
}
