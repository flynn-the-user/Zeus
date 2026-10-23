---
source: https://www.chessprogramming.org/Negamax
retrieved: 2026-10-23
---

# Negamax

Negamax is a simplified implementation of Minimax that relies on the zero-sum
property of a two-player game: `max(a, b) == -min(-a, -b)`. Instead of writing
separate Max-player and Min-player subroutines, a single recursive function is
used, and the returned score is negated at each ply as control passes to the
opponent.

## Evaluation sign convention (critical invariant)

> In order for negaMax to work, your Static Evaluation function must return a
> score relative to the side being evaluated.

A minimal illustration given on the page:

```
score = materialWeight * (numWhitePieces - numBlackPieces) * who2move
```

where `who2move = 1` for white and `-1` for black. Every leaf evaluation must
be expressed "from the point of view of the player to move", not
White-relative, or the sign-flip at each recursive level is meaningless.

## Basic recursive pseudocode (as given on the page)

```
int negaMax( int depth ) {
    if ( depth == 0 ) return evaluate();
    int max = -oo;
    for ( all moves)  {
        score = -negaMax( depth - 1 );
        if( score > max )
            max = score;
    }
    return max;
}
```

## Root call

The page notes the root call needs its own wrapper that, in addition to the
above loop, records *which* move produced the best score (the plain
`negaMax` above only returns a score, not a move) so the root can report a
`bestmove`.

## Zeus usage

This is the basis for `zeus-engine`'s `search::negamax` fixed-depth search in
v0.1, before Alpha-Beta pruning (see `Reference/search/Alpha-Beta.md`) is
layered on top. The sign convention above is why `Evaluator::evaluate()` in
Zeus returns a side-to-move-relative centipawn score, not a White-relative
one.
