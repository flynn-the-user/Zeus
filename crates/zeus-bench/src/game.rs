//! Plays one game between two already-spawned UCI engines, enforcing a
//! real wall-clock time control from the harness side (like a tournament
//! arbiter), independent of whether either engine manages its own clock.
//! This matters for Zeus v0.1 specifically: it has no time management yet
//! (see ROADMAP.md), so this is the thing standing between it and losing
//! every game on time once an opponent is fast.
//!
//! Game-termination rules (checkmate/stalemate from `chess::Board::status`,
//! plus the 50-move rule, threefold repetition, and insufficient material)
//! are standard chess rules used to adjudicate match results; they are not
//! an "engine algorithm" decision and so are not drawn from a
//! chessprogramming.org citation the way Zeus's own search/eval are.
use crate::uci_client::UciEngine;
use chess::{Board, BoardStatus, ChessMove, Color, Piece, ALL_PIECES};
use std::collections::HashMap;
use std::str::FromStr;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy)]
pub struct TimeControl {
    pub base_ms: i64,
    pub inc_ms: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    WhiteWins,
    BlackWins,
    Draw,
}

#[derive(Debug, Clone, Copy)]
pub enum EndReason {
    Checkmate,
    Stalemate,
    FiftyMoveRule,
    ThreefoldRepetition,
    InsufficientMaterial,
    MaxPlies,
    Timeout,
    IllegalOrNoMove,
}

pub struct GameRecord {
    pub outcome: Outcome,
    pub reason: EndReason,
    pub plies: u32,
}

pub fn play_game(
    white: &mut UciEngine,
    black: &mut UciEngine,
    tc: TimeControl,
    max_plies: u32,
) -> GameRecord {
    let _ = white.new_game();
    let _ = black.new_game();

    let mut board = Board::default();
    let mut uci_moves: Vec<String> = Vec::new();
    let mut white_ms = tc.base_ms;
    let mut black_ms = tc.base_ms;
    let mut repetition: HashMap<u64, u32> = HashMap::new();
    repetition.insert(board.get_hash(), 1);
    let mut halfmove_clock: u32 = 0;

    for ply in 0..max_plies {
        let mover_is_white = board.side_to_move() == Color::White;
        let (mover, mover_time, opp_time) = if mover_is_white {
            (&mut *white, &mut white_ms, &mut black_ms)
        } else {
            (&mut *black, &mut black_ms, &mut white_ms)
        };

        let _ = mover.set_position_moves(&uci_moves);
        let (wtime, btime) = if mover_is_white {
            (*mover_time, *opp_time)
        } else {
            (*opp_time, *mover_time)
        };
        // Fixed buffer over the clock-allotted time to absorb IPC /
        // process-scheduling latency, so we forfeit on an actual
        // thinking-time overrun, not on OS jitter.
        let hard_timeout = Duration::from_millis((*mover_time).max(0) as u64 + 500);
        let start = Instant::now();
        let mv_str =
            mover.go_and_wait_bestmove(wtime, btime, tc.inc_ms, tc.inc_ms, hard_timeout);
        let elapsed_ms = start.elapsed().as_millis() as i64;

        *mover_time = *mover_time - elapsed_ms + tc.inc_ms;
        if *mover_time <= 0 {
            return GameRecord {
                outcome: if mover_is_white {
                    Outcome::BlackWins
                } else {
                    Outcome::WhiteWins
                },
                reason: EndReason::Timeout,
                plies: ply,
            };
        }

        let mv_str = match mv_str {
            Some(s) if s != "0000" => s,
            _ => {
                return GameRecord {
                    outcome: if mover_is_white {
                        Outcome::BlackWins
                    } else {
                        Outcome::WhiteWins
                    },
                    reason: EndReason::IllegalOrNoMove,
                    plies: ply,
                };
            }
        };

        let mv = match ChessMove::from_str(&mv_str) {
            Ok(m) if board.legal(m) => m,
            _ => {
                return GameRecord {
                    outcome: if mover_is_white {
                        Outcome::BlackWins
                    } else {
                        Outcome::WhiteWins
                    },
                    reason: EndReason::IllegalOrNoMove,
                    plies: ply,
                };
            }
        };

        let is_capture = board.piece_on(mv.get_dest()).is_some();
        let is_pawn_move = board.piece_on(mv.get_source()) == Some(Piece::Pawn);
        halfmove_clock = if is_capture || is_pawn_move {
            0
        } else {
            halfmove_clock + 1
        };

        board = board.make_move_new(mv);
        uci_moves.push(mv_str);

        match board.status() {
            BoardStatus::Checkmate => {
                return GameRecord {
                    outcome: if mover_is_white {
                        Outcome::WhiteWins
                    } else {
                        Outcome::BlackWins
                    },
                    reason: EndReason::Checkmate,
                    plies: ply + 1,
                };
            }
            BoardStatus::Stalemate => {
                return GameRecord {
                    outcome: Outcome::Draw,
                    reason: EndReason::Stalemate,
                    plies: ply + 1,
                };
            }
            BoardStatus::Ongoing => {}
        }

        if halfmove_clock >= 100 {
            return GameRecord {
                outcome: Outcome::Draw,
                reason: EndReason::FiftyMoveRule,
                plies: ply + 1,
            };
        }

        if insufficient_material(&board) {
            return GameRecord {
                outcome: Outcome::Draw,
                reason: EndReason::InsufficientMaterial,
                plies: ply + 1,
            };
        }

        let count = repetition.entry(board.get_hash()).or_insert(0);
        *count += 1;
        if *count >= 3 {
            return GameRecord {
                outcome: Outcome::Draw,
                reason: EndReason::ThreefoldRepetition,
                plies: ply + 1,
            };
        }
    }

    GameRecord {
        outcome: Outcome::Draw,
        reason: EndReason::MaxPlies,
        plies: max_plies,
    }
}

/// Conservative under-approximation (K-vs-K, K+single-minor-vs-K only): it
/// can only end a hopeless game *faster*, never misjudge one, since missed
/// draws simply fall through to the 50-move rule / max-ply cutoff instead.
fn insufficient_material(board: &Board) -> bool {
    let non_king_pieces: u32 = ALL_PIECES
        .iter()
        .filter(|&&p| p != Piece::King)
        .map(|&p| (board.pieces(p) & board.combined()).popcnt())
        .sum();
    if non_king_pieces == 0 {
        return true;
    }
    if non_king_pieces == 1 {
        return (board.pieces(Piece::Knight) | board.pieces(Piece::Bishop)).popcnt() == 1;
    }
    false
}
