---
source: https://www.chessprogramming.org/Quiescence_Search
retrieved: 2026-10-27
---

# Quiescence Search

> Simply stopping your search when you reach the desired depth and then
> evaluate, is very dangerous.

Example given: if the last move considered is QxP, a fixed-depth search
might see "won a pawn" when one ply deeper reveals PxQ — actually losing
the queen. This is the **horizon effect**. The fix: "only evaluate 'quiet'
positions, or positions where there are no winning tactical moves to be
made" — i.e. at the normal search's leaf, keep searching captures (only)
until no capture improves on the standing evaluation.

## Stand-pat

> the static evaluation as a 'stand-pat' score [...] there is at least one
> move that can either match or beat the lower bound

Used as both the initial `best_value` and a lower bound on `alpha` — this
assumes *not* capturing is always an option (true unless in check, see
below).

## Pseudocode (adopted verbatim, translated to Rust)

```
int Quiesce( int alpha, int beta ) {
    int static_eval = Evaluate();

    // Stand Pat
    int best_value = static_eval;
    if( best_value >= beta )
        return best_value;
    if( best_value > alpha )
        alpha = best_value;

    until( every_capture_has_been_examined )  {
        MakeCapture();
        score = -Quiesce( -beta, -alpha );
        TakeBackMove();

        if( score >= beta )
            return score;
        if( score > best_value )
            best_value = score;
        if( score > alpha )
            alpha = score;
    }

    return best_value;
}
```

## Check handling

> If in check, stand-pat is disallowed and all evasions must be searched.

Zeus implements this: when the side to move is in check, `quiesce` skips
the stand-pat/capture-only path entirely and searches **all** legal moves
(full check evasions), exactly as this page specifies — not an
enhancement, the base algorithm's own correctness requirement (a king in
check is not "quiet" just because no capture improves the static eval).

## Search explosion risk / what Zeus does NOT do yet

> It's crucial to introduce simple capture ordering such as MVV-LVA before
> implementing qsearch [...] delta pruning can be used [...] no explicit
> formula [given].

Move ordering (MVV-LVA) is the *next* roadmap stage, not this one. Zeus
v1's quiescence therefore examines captures in whatever order
`MoveGen`/`set_iterator_mask` produces them, unordered. This is slower
than it could be, but not unbounded: quiescence recursion is still capped
by the same hard-deadline node check as the main search
(`SearchContext::visit_node`, `Reference/search/Time_Management.md`), and
each ply of quiescence strictly reduces material (a capture removes a
piece), so the recursion itself cannot be infinite even unordered. Delta
pruning is explicitly left for later (CPW gives no formula here to
implement against; it belongs with the move-ordering stage where its
cost/benefit can be measured against ordered captures).

## Zeus usage

`zeus-engine::search::quiesce` replaces the `depth_left == 0 ->
evaluate(board)` leaf case in `alpha_beta` with this function, per
`Reference/search/Alpha-Beta.md`'s own negamax-framework pseudocode which
already named `quiesce(alpha, beta)` as the intended leaf call (v0.1/v1
used `evaluate()` directly there as a documented stopgap until this
stage).
