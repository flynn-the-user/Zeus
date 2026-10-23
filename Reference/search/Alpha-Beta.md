---
source: https://www.chessprogramming.org/Alpha-Beta
retrieved: 2026-10-23
---

# Alpha-Beta

Alpha-Beta pruning is a branch-and-bound enhancement of Minimax/Negamax that
discards subtrees which cannot influence the final decision, without ever
overlooking a better move. It maintains two bounds:

- **alpha**: the minimum score the side to move is already assured of.
- **beta**: the maximum score the opponent will allow (a score the opponent
  can already guarantee for themselves elsewhere, so the mover must not let
  their score reach or exceed it).

> If one already has found a quite good move and search for alternatives, one
> refutation is enough to avoid it.

In a well-ordered tree, node count approaches the square root of the
equivalent plain Minimax tree size, i.e. roughly double the achievable depth
for the same node budget. This is the formal justification (not an estimate
from our own experience) for treating Alpha-Beta as mandatory before any
other search enhancement.

## Fail-soft, Negamax framework (adopted by Zeus v0.1)

```
int alphaBeta( int alpha, int beta, int depthleft ) {
   if( depthleft == 0 ) return quiesce( alpha, beta );
   bestValue = -infinity;
   for ( all moves)  {
      score = -alphaBeta( -beta, -alpha, depthleft - 1 );
      if( score > bestValue )
      {
         bestValue = score;
         if( score > alpha )
            alpha = score;
      }
      if( score >= beta )
         return bestValue;
   }
   return bestValue;
}
```

Zeus v0.1 has no quiescence search yet (that is a later roadmap stage), so the
`depthleft == 0` leaf calls the static evaluator directly instead of
`quiesce(alpha, beta)`; everything else is this pseudocode verbatim,
translated to Rust.

## Fail-soft vs fail-hard

The page also gives Max/Min-framework and fail-hard variants. Zeus uses
**fail-soft negamax** specifically because fail-soft "can return scores
outside the original bounds, preserving more information for transposition
tables" — relevant for later roadmap stages once a transposition table is
added, so we standardize on fail-soft from the start rather than switching
conventions mid-project.

## Zeus usage

Implemented in `zeus-engine::search::alpha_beta`. The `who2move`-relative
evaluation convention required by Negamax (see
`Reference/search/Negamax.md`) still applies unchanged.
