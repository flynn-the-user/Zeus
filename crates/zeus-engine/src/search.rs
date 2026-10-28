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
//! - `Reference/search/Quiescence_Search.md` — `quiesce` below, which
//!   `alpha_beta` now calls at `depth_left == 0` instead of `evaluate()`
//!   directly (that page's own pseudocode is for the negamax framework and
//!   already named this `quiesce(alpha, beta)` call; v0.1/v1 used
//!   `evaluate()` there as a documented stopgap until this stage).
//! - `Reference/search/MVV-LVA.md`, `Reference/search/Killer_Heuristic.md`,
//!   `Reference/search/Move_Ordering.md` — `order_moves`/`capture_score`
//!   below. Applied in `alpha_beta` (captures + killers) and in
//!   `quiesce`'s non-check branch (captures only - killers are specifically
//!   for quiet moves, and quiescence's quiet-ish moves are check evasions,
//!   scoped out, see `Reference/search/Killer_Heuristic.md`).
//!
//! `SearchContext.use_quiescence` is a temporary, explicit toggle (not
//! itself drawn from any reference - see `CITATIONS.md`) added specifically
//! to let `zeus-bench` measure move ordering's own Elo contribution with
//! quiescence held constant (off, matching the v1 baseline it's compared
//! against), then measure quiescence's contribution again now that
//! ordering exists, isolating each variable in turn.
use crate::eval::{evaluate, PIECE_VALUE};
use chess::{Board, BoardStatus, ChessMove, MoveGen, EMPTY};
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

/// Captures sort far above killers, which sort above ordinary quiet moves
/// (`Reference/search/Move_Ordering.md`'s collapsed tier order for a
/// program without a hash/PV move or SEE yet). Both constants are Zeus's
/// own choice of margin, not drawn from a reference, sized so the tiering
/// holds for every legal piece combination (worst-case capture, a queen
/// taking a pawn, still scores above the killer band; the killer band
/// still scores above 0).
const CAPTURE_BASE: i32 = 1_000_000;
const KILLER_SCORE: [i32; 2] = [2_000, 1_000];

/// `victim_value*16 - attacker_value`, per `Reference/search/MVV-LVA.md`
/// (the `*16` multiplier is Zeus's own choice, documented there). `None`
/// for non-captures, including en passant (also documented there as a
/// known gap: the destination square is empty for en passant, so
/// `piece_on` misses it).
fn capture_score(board: &Board, mv: ChessMove) -> Option<i32> {
    board.piece_on(mv.get_dest()).map(|victim| {
        let attacker = board
            .piece_on(mv.get_source())
            .expect("move source always has a piece");
        PIECE_VALUE[victim.to_index()] * 16 - PIECE_VALUE[attacker.to_index()]
    })
}

/// Shared state for one `go` search: lets the UCI layer abort a search
/// asynchronously (on a `stop` command) and bounds how long a single
/// iteration may run past the hard deadline. Also owns the per-ply killer
/// table (`Reference/search/Killer_Heuristic.md`) and the quiescence
/// on/off toggle used for the move-ordering ablation study (see module
/// doc comment).
pub struct SearchContext {
    pub stop: Arc<AtomicBool>,
    pub hard_deadline: Instant,
    pub nodes: Arc<AtomicU64>,
    pub use_quiescence: bool,
    local_node_count: u64,
    killers: Vec<[Option<ChessMove>; 2]>,
}

impl SearchContext {
    pub fn new(stop: Arc<AtomicBool>, hard_deadline: Instant, use_quiescence: bool) -> SearchContext {
        SearchContext {
            stop,
            hard_deadline,
            nodes: Arc::new(AtomicU64::new(0)),
            use_quiescence,
            local_node_count: 0,
            killers: vec![[None, None]; MAX_DEPTH as usize + 1],
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

    /// Stores `mv` as the new slot-0 killer for `ply`, demoting the
    /// previous slot-0 killer to slot 1 (unless it's the same move) -
    /// per `Reference/search/Killer_Heuristic.md`'s "replacement scheme
    /// ensuring all slots contain different moves".
    fn store_killer(&mut self, ply: usize, mv: ChessMove) {
        let slots = &mut self.killers[ply];
        if slots[0] != Some(mv) {
            slots[1] = slots[0];
            slots[0] = Some(mv);
        }
    }

    /// Legal moves ordered captures-first (MVV-LVA), then this ply's
    /// killer moves, then everything else, per
    /// `Reference/search/Move_Ordering.md`.
    fn ordered_moves(&self, board: &Board, ply: usize) -> Vec<ChessMove> {
        let killers = self.killers.get(ply);
        let mut scored: Vec<(ChessMove, i32)> = MoveGen::new_legal(board)
            .map(|mv| {
                let score = if let Some(cap) = capture_score(board, mv) {
                    CAPTURE_BASE + cap
                } else if killers.is_some_and(|k| k[0] == Some(mv)) {
                    KILLER_SCORE[0]
                } else if killers.is_some_and(|k| k[1] == Some(mv)) {
                    KILLER_SCORE[1]
                } else {
                    0
                };
                (mv, score)
            })
            .collect();
        scored.sort_by(|a, b| b.1.cmp(&a.1));
        scored.into_iter().map(|(mv, _)| mv).collect()
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
fn alpha_beta(
    board: &Board,
    mut alpha: i32,
    beta: i32,
    depth_left: u32,
    ply: u32,
    ctx: &mut SearchContext,
) -> i32 {
    ctx.visit_node();
    if ctx.stop.load(Ordering::Relaxed) {
        return 0; // discarded by the caller; value is irrelevant
    }
    if let Some(score) = terminal_score(board) {
        return score;
    }
    if depth_left == 0 {
        return if ctx.use_quiescence {
            quiesce(board, alpha, beta, ctx)
        } else {
            evaluate(board)
        };
    }

    let mut best_value = -MATE_VALUE - 1;
    for mv in ctx.ordered_moves(board, ply as usize) {
        let child = board.make_move_new(mv);
        let score = -alpha_beta(&child, -beta, -alpha, depth_left - 1, ply + 1, ctx);
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
            // Killer Heuristic (Reference/search/Killer_Heuristic.md):
            // only non-capture cutoffs are remembered - captures are
            // already well-ordered by MVV-LVA.
            if board.piece_on(mv.get_dest()).is_none() {
                ctx.store_killer(ply as usize, mv);
            }
            return best_value;
        }
    }
    best_value
}

/// Captures-only search to a quiet position, per
/// `Reference/search/Quiescence_Search.md`'s pseudocode: stand-pat as the
/// initial lower bound (unless in check, where stand-pat is disallowed and
/// all legal evasions are searched instead), then try captures to see if
/// any beat it, with the same alpha-beta cutoffs as the main search.
fn quiesce(board: &Board, mut alpha: i32, beta: i32, ctx: &mut SearchContext) -> i32 {
    ctx.visit_node();
    if ctx.stop.load(Ordering::Relaxed) {
        return 0; // discarded by the caller
    }
    if let Some(score) = terminal_score(board) {
        return score;
    }

    let in_check = *board.checkers() != EMPTY;

    let mut best_value;
    let moves: Vec<ChessMove>;
    if in_check {
        // "If in check, stand-pat is disallowed and all evasions must be
        // searched" - every legal move is an evasion, so no mask/ordering.
        best_value = -MATE_VALUE - 1;
        moves = MoveGen::new_legal(board).collect();
    } else {
        let stand_pat = evaluate(board);
        best_value = stand_pat;
        if best_value >= beta {
            return best_value;
        }
        if best_value > alpha {
            alpha = best_value;
        }
        // Captures only: moves landing on an opponent-occupied square,
        // MVV-LVA ordered (Reference/search/MVV-LVA.md) - per
        // Reference/search/Quiescence_Search.md, unordered captures risk
        // a search explosion. Known simplification: en passant captures
        // land on an empty square and are missed by this mask, same as
        // most simple engines' first quiescence pass.
        let mut gen = MoveGen::new_legal(board);
        gen.set_iterator_mask(*board.color_combined(!board.side_to_move()));
        let mut scored: Vec<(ChessMove, i32)> = gen
            .map(|mv| (mv, capture_score(board, mv).unwrap_or(0)))
            .collect();
        scored.sort_by(|a, b| b.1.cmp(&a.1));
        moves = scored.into_iter().map(|(mv, _)| mv).collect();
    }

    for mv in moves {
        let child = board.make_move_new(mv);
        let score = -quiesce(&child, -beta, -alpha, ctx);
        if ctx.stop.load(Ordering::Relaxed) {
            return best_value;
        }
        if score >= beta {
            return score;
        }
        if score > best_value {
            best_value = score;
        }
        if score > alpha {
            alpha = score;
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

    for mv in ctx.ordered_moves(board, 0) {
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
        let score = -alpha_beta(&child, -beta, -alpha, depth.saturating_sub(1), 1, ctx);
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
#[allow(clippy::too_many_arguments)]
pub fn search_iterative(
    board: &Board,
    stop: Arc<AtomicBool>,
    hard_deadline: Instant,
    soft_deadline: Option<Instant>,
    max_depth: u32,
    use_quiescence: bool,
    mut on_iteration: impl FnMut(&IterationInfo),
) -> Option<(ChessMove, i32)> {
    let mut overall_best: Option<(ChessMove, i32)> = None;
    let mut total_nodes = 0u64;

    for depth in 1..=max_depth {
        let mut ctx = SearchContext::new(Arc::clone(&stop), hard_deadline, use_quiescence);
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
