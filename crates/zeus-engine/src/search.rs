//! Fixed-depth Negamax search with fail-soft Alpha-Beta pruning.
//!
//! References:
//! - `Reference/search/Negamax.md` — the negamax recursion and the
//!   requirement that `evaluate()` be side-to-move-relative (see
//!   `zeus_engine::eval::evaluate`).
//! - `Reference/search/Alpha-Beta.md` — the fail-soft, negamax-framework
//!   pseudocode this function is a direct translation of. The one
//!   deviation from that pseudocode: it calls `quiesce(alpha, beta)` at
//!   `depthleft == 0`; Zeus v0.1 has no quiescence search yet (see
//!   `ROADMAP.md`), so the leaf case calls `evaluate()` directly instead.
use crate::eval::evaluate;
use chess::{Board, BoardStatus, ChessMove, MoveGen};

/// Sentinel score for checkmate, well outside any reachable material-eval
/// value (king value cancels between sides since both sides always have
/// exactly one king in any position that isn't itself already mate, so the
/// true evaluation range is roughly +/-4000). Not drawn from a reference:
/// this is just a safe-margin sentinel, not a cited algorithmic choice.
const MATE_VALUE: i32 = 30_000;

/// Terminal-state handling required to apply the Negamax/Alpha-Beta
/// pseudocode (written for abstract game trees) to chess: when there are no
/// legal moves, the `for (all moves)` loop in the reference pseudocode is
/// simply empty, which would wrongly leave `bestValue` at `-infinity`
/// instead of distinguishing a loss (checkmate) from a draw (stalemate).
/// This is base chess rules, not a search-algorithm citation.
fn terminal_score(board: &Board) -> Option<i32> {
    match board.status() {
        BoardStatus::Checkmate => Some(-MATE_VALUE),
        BoardStatus::Stalemate => Some(0),
        BoardStatus::Ongoing => None,
    }
}

/// Fail-soft negamax alpha-beta, per `Reference/search/Alpha-Beta.md`.
/// Returns a score relative to the side to move at `board`.
fn alpha_beta(board: &Board, mut alpha: i32, beta: i32, depth_left: u32) -> i32 {
    if let Some(score) = terminal_score(board) {
        return score;
    }
    if depth_left == 0 {
        return evaluate(board);
    }

    let mut best_value = -MATE_VALUE - 1;
    for mv in MoveGen::new_legal(board) {
        let child = board.make_move_new(mv);
        let score = -alpha_beta(&child, -beta, -alpha, depth_left - 1);
        if score > best_value {
            best_value = score;
            if score > alpha {
                alpha = score;
            }
        }
        if score >= beta {
            return best_value;
        }
    }
    best_value
}

/// Root call: per `Reference/search/Negamax.md` ("Implementation Details"),
/// the root needs its own wrapper around the recursive search that also
/// tracks which move produced the best score, since the recursive function
/// only returns a score.
pub fn search_fixed_depth(board: &Board, depth: u32) -> Option<(ChessMove, i32)> {
    let mut best: Option<(ChessMove, i32)> = None;
    let mut alpha = -MATE_VALUE - 1;
    let beta = MATE_VALUE + 1;
    for mv in MoveGen::new_legal(board) {
        let child = board.make_move_new(mv);
        let score = -alpha_beta(&child, -beta, -alpha, depth.saturating_sub(1));
        if best.is_none_or(|(_, best_score)| score > best_score) {
            best = Some((mv, score));
        }
        if score > alpha {
            alpha = score;
        }
    }
    best
}
