---
source: https://www.chessprogramming.org/MVV-LVA
retrieved: 2026-10-28
---

# MVV-LVA

> Most Valuable Victim - Least Valuable Aggressor, is a simple heuristic to
> generate or sort capture moves in a reasonable order.

Two nested cycles: find the most valuable victim first (Queen, Rook,
Bishop, Knight, Pawn, descending), then among moves capturing that victim,
prefer the least valuable aggressor (Pawn, Knight, Bishop, Rook, Queen,
King, ascending) — i.e. prefer "PxR" over "BxP".

> The heuristic may fail, if victims attacked by more valuable attackers
> are defended. In such situations, most programs employ attack tables or
> static exchange evaluation (SEE) for more accurate move assessment.

No scoring table/formula is given on this page. `Reference/search/Move_Ordering.md`
explicitly endorses using plain MVV-LVA (without SEE) as a standalone,
simpler alternative for capture ordering — SEE/winning-vs-losing-capture
distinction is out of scope for this stage.

## Zeus usage

`crates/zeus-engine/src/search.rs::mvv_lva_score`: `victim_value * 16 -
attacker_value`, using the same `eval::PIECE_VALUE` table as the static
evaluator (one source of truth for piece values). The `* 16` multiplier is
Zeus's own choice (not from this page, which gives no formula) — large
enough that victim value always dominates attacker value for any legal
piece pair (max attacker value 900 for a queen; 16x any lesser victim
value exceeds that), so the two cycles described above fall out correctly
from a single sort key without needing two explicit passes.

Known limitation (not addressed by this page, same footing as the
quiescence en passant note): en passant captures have an empty
destination square, so `board.piece_on(dest)` returns `None` for them and
they are scored as non-captures (0) rather than pawn-takes-pawn. Noted in
`CITATIONS.md`.
