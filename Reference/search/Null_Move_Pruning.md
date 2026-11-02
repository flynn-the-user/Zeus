---
source: https://www.chessprogramming.org/Null_Move_Pruning
retrieved: 2026-11-02
---

# Null Move Pruning

## Null Move Observation

> In almost all chess positions, making a null move (passing a turn) is
> worse than the best legal move.

So: pass the turn, search the resulting position at a reduced depth with
a null (zero-width) window; if even after giving the opponent a free
tempo the score still fails high over `beta`, the real best legal move
would too — prune without searching any real moves.

## Pseudocode (adopted verbatim)

```
if ( ... ) // NMP Conditions
{
    int r = ...; // NMP Reduction
    pos.make_null_move();
    int v = -search(pos, -beta, -(beta - 1), depth - r);
    pos.undo_null_move();
    if (v >= beta)
        return v;
}
```

## Reduction (R)

> A basic implementation of NMP uses a fixed reduction of 3 or 4.

Zeus uses **R = 3** (the lower end of that basic-implementation range).

## When not to apply

- **In check**: not a legal "pass".
- **Pawn-only endgames**: zugzwang-prone, the null move observation
  breaks down exactly there.
- **Consecutive null moves**: don't null-move twice in a row on the same
  path.
- **Shallow depths**: "generally skip if depth < 4".
- PV/non-cut nodes: "optional guard depending on implementation" — Zeus
  doesn't distinguish PV nodes structurally yet (that's part of
  Principal Variation Search, a later roadmap stage), so this guard is
  not applied; noted as a scope choice, not an oversight.

## Making a null move

> the position must support `make_null_move()`/`undo_null_move()` [...]
> toggling the side to move and clearing the en passant target square

The `chess` crate's `Board::null_move() -> Option<Board>` already does
exactly this (confirmed by reading its source: flips `side_to_move`,
clears en passant, recomputes pin info, and itself returns `None` when
in check) — used directly rather than reimplemented, same footing as
`make_move_new` (board representation/movement, not search algorithm,
per this project's standing rule on what the `chess` crate may be used
for).

## Zeus usage

`crates/zeus-engine/src/search.rs::alpha_beta`: right after the
transposition-table probe, before move generation, when `depth_left >=
4`, not in check, the side to move has at least one piece besides pawns
and king (Zeus's own concrete reading of "pawn-only endgame"), and no
null move was just made on this path (`allow_null`, a new parameter
threaded through `alpha_beta`). Exactly the pseudocode above: null
window `[-beta, -beta+1]`, reduced depth `depth_left - 3`, `allow_null =
false` for the recursive call so it can't chain. No transposition-table
store on the resulting cutoff — not mentioned in the page's pseudocode,
so not added as an unverified extra.
