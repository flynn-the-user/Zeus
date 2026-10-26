//! Iterative-deepening Negamax search with fail-soft Alpha-Beta pruning,
//! abortable by a shared stop flag and a hard deadline.
//!
//! References:
//! - `Reference/search/Negamax.md` / `Reference/search/Alpha-Beta.md` — the
//!   recursive search itself, unchanged from v0.1 apart from the
//!   abort-checking added for time management (see below).
//! - `Reference/search/Iterative_Deepening.md` — depth = 1, 2, 3, ...,
//!   keep the last *fully completed* depth's result, discard an aborted
//!   (incomplete) iteration, with Zeus's own depth-1 partial-result
//!   fallback documented there so the engine can never return no move.
//! - `Reference/search/Time_Management.md` — the soft/hard two-level
//!   deadline this module is abortable against (soft bound is enforced by
//!   the caller deciding whether to start another iteration; hard bound is
//!   enforced here, mid-search, via periodic checks).
use crate::eval::evaluate;
use chess::{Board, BoardStatus, ChessMove, MoveGen};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Instant;

const MATE_VALUE: i32 = 30_000;
/// Sentinel: deepest ply-count from the root. Prevents runaway iteration
/// in near-trivial endgame positions when there's no time pressure (e.g.
/// `go infinite`); not drawn from a reference, just a safety cap.
pub const MAX_DEPTH: u32 = 64;
/// How often (in visited nodes) the hard deadline is polled. Not drawn
/// from a reference either — Reference/search/Time_Management.md only
/// says the hard bound is "checked periodically", not how often; this is
/// frequent enough to abort within a few ms of the deadline without
/// calling Instant::now() on every single node.
const TIME_CHECK_INTERVAL: u64 = 2048;

/// Shared state for one `go` search: lets the UCI layer abort a search
/// asynchronously (on a `stop` command) and bounds how long a single
/// iteration may run past the hard deadline.
pub struct SearchContext {
    pub stop: Arc<AtomicBool>,
    pub hard_deadline: Instant,
    pub nodes: Arc<AtomicU64>,
    local_node_count: u64,
}

impl SearchContext {
    pub fn new(stop: Arc<AtomicBool>, hard_deadline: Instant) -> SearchContext {
        SearchContext {
            stop,
            hard_deadline,
            nodes: Arc::new(AtomicU64::new(0)),
            local_node_count: 0,
        }
    }

    fn visit_node(&mut self) {
        self.local_node_count += 1;
        self.nodes.fetch_add(1, Ordering::Relaxed);
        if self.local_node_count % TIME_CHECK_INTERVAL == 0 && Instant::now() >= self.hard_deadline
        {
            self.stop.store(true, Ordering::Relaxed);
        }
    }
}

fn terminal_score(board: &Board) -> Option<i32> {
    match board.status() {
        BoardStatus::Checkmate => Some(-MATE_VALUE),
        BoardStatus::Stalemate => Some(0),
        BoardStatus::Ongoing => None,
    }
}

/// Fail-soft negamax alpha-beta, per `Reference/search/Alpha-Beta.md`, plus
/// an abort check at the top of every call (`ctx.visit_node`), which both
/// polls the hard deadline and lets an externally-set `stop` flag (from a
/// UCI `stop` command) cut the search short within `TIME_CHECK_INTERVAL`
/// nodes.
fn alpha_beta(board: &Board, mut alpha: i32, beta: i32, depth_left: u32, ctx: &mut SearchContext) -> i32 {
    ctx.visit_node();
    if ctx.stop.load(Ordering::Relaxed) {
        return 0; // discarded by the caller; value is irrelevant
    }
    if let Some(score) = terminal_score(board) {
        return score;
    }
    if depth_left == 0 {
        return evaluate(board);
    }

    let mut best_value = -MATE_VALUE - 1;
    for mv in MoveGen::new_legal(board) {
        let child = board.make_move_new(mv);
        let score = -alpha_beta(&child, -beta, -alpha, depth_left - 1, ctx);
        if ctx.stop.load(Ordering::Relaxed) {
            return best_value; // discarded by the caller
        }
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

/// One complete (or aborted) root pass at a fixed depth. Returns the best
/// (move, score) among whichever root moves were fully evaluated before an
/// abort — per `Reference/search/Iterative_Deepening.md`'s depth-1
/// fallback note, a move whose sub-search was interrupted is never
/// recorded (its score is unreliable), but moves evaluated earlier in this
/// same pass are kept.
fn search_root_once(board: &Board, depth: u32, ctx: &mut SearchContext) -> Option<(ChessMove, i32)> {
    let mut best: Option<(ChessMove, i32)> = None;
    let mut alpha = -MATE_VALUE - 1;
    let beta = MATE_VALUE + 1;

    for mv in MoveGen::new_legal(board) {
        // Once there is at least one evaluated move to fall back on,
        // respect an abort before starting the next. The very first move
        // is always evaluated regardless of `stop` (even if that means
        // its own sub-search gets cut short and its score is unreliable),
        // so the root can never end up with zero candidate moves even if
        // `stop` is set before a single move finishes - the
        // Iterative_Deepening.md fallback note is for falling back
        // between *iterations*, this is the same idea one level lower, for
        // falling back between *moves* within the first iteration.
        if best.is_some() && ctx.stop.load(Ordering::Relaxed) {
            break;
        }
        let child = board.make_move_new(mv);
        let score = -alpha_beta(&child, -beta, -alpha, depth.saturating_sub(1), ctx);
        let aborted_mid_move = ctx.stop.load(Ordering::Relaxed);
        if best.is_none() || (!aborted_mid_move && score > best.unwrap().1) {
            best = Some((mv, score));
        }
        if aborted_mid_move {
            break;
        }
        if score > alpha {
            alpha = score;
        }
    }
    best
}

pub struct IterationInfo {
    pub depth: u32,
    pub score: i32,
    pub best_move: ChessMove,
    pub nodes: u64,
}

/// Iterative deepening driver (`Reference/search/Iterative_Deepening.md`):
/// depth = 1, 2, 3, ..., stopping when `stop` is set, the hard deadline
/// passes, `soft_deadline` has already elapsed (checked only *between*
/// iterations, per `Reference/search/Time_Management.md`), or `max_depth`
/// is reached. Returns `None` only if the root position has no legal
/// moves (checkmate/stalemate).
pub fn search_iterative(
    board: &Board,
    stop: Arc<AtomicBool>,
    hard_deadline: Instant,
    soft_deadline: Option<Instant>,
    max_depth: u32,
    mut on_iteration: impl FnMut(&IterationInfo),
) -> Option<(ChessMove, i32)> {
    let mut overall_best: Option<(ChessMove, i32)> = None;
    let mut total_nodes = 0u64;

    for depth in 1..=max_depth {
        let mut ctx = SearchContext::new(Arc::clone(&stop), hard_deadline);
        let result = search_root_once(board, depth, &mut ctx);
        total_nodes += ctx.nodes.load(Ordering::Relaxed);
        let aborted = stop.load(Ordering::Relaxed);

        match result {
            Some((mv, score)) if !aborted => {
                overall_best = Some((mv, score));
                on_iteration(&IterationInfo {
                    depth,
                    score,
                    best_move: mv,
                    nodes: total_nodes,
                });
            }
            Some((mv, score)) if overall_best.is_none() => {
                // Depth-1 safety net (see Reference/search/Iterative_Deepening.md):
                // no fully-completed iteration exists yet, so a partial
                // result is better than returning no move at all.
                overall_best = Some((mv, score));
            }
            _ => {}
        }

        if aborted {
            break;
        }
        if let Some(soft) = soft_deadline {
            if Instant::now() >= soft {
                break;
            }
        }
        if Instant::now() >= hard_deadline {
            break;
        }
    }

    overall_best
}
