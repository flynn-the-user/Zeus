---
source: https://www.chessprogramming.org/Piece-Square_Tables
retrieved: 2026-10-30
---

# Piece-Square Tables (general technique)

> provide a simple way to assign values to specific pieces on specific
> squares [...] Material and piece-square tables alone are enough for a
> program to play a semi-decent game of chess.

## Middlegame/endgame separation

> Modern engines use two sets of tables, one for the opening/middlegame
> and another for the endgame [...] interpolated between these two based
> on game stage (often material on the board) [...] allows for changes in
> material as well, notably pawns gaining value in endgames.

This page describes the general case as a *smooth interpolation*
("Tapered Eval" — a separate CPW page, not fetched this stage, not in
`ROADMAP.md`'s current scope). Zeus implements the simpler **binary
switch** already specified in
`Reference/evaluation/Simplified_Evaluation_Function.md` (Michniewski's
proposal, which is where this stage's actual table values come from —
this page gives no values of its own): use the middlegame table normally,
switch to the endgame table once "both sides have no queens, or each
side with a queen has no other piece or one minor piece maximum" (that
page's own exact endgame-begins condition, already quoted there).
Tapering is left as a possible future refinement, not this stage's scope.

No indexing/layout convention or Black-mirroring formula is given on
this general page either — `Reference/evaluation/Simplified_Evaluation_Function.md`
already covers that ("Black uses mirrored values of all tables").

## Zeus usage

`crates/zeus-engine/src/eval.rs`: for each piece on the board, look up
its PST value (White: table indexed by `(7 - rank)*8 + file`, i.e. row 0
= rank 8 as printed on that page; Black: `rank*8 + file`, the vertical
mirror of White's indexing — a pawn on White's 7th rank and a pawn on
Black's 2nd rank hit the same table row) using the middlegame table or
endgame table per the binary switch above, add to the material sum
already computed per
`Reference/evaluation/Simplified_Evaluation_Function.md`.
