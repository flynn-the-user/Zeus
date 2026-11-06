---
source: https://www.chessprogramming.org/Late_Move_Reductions
retrieved: 2026-11-06
---

# Late Move Reductions (LMR)

> save search by reducing moves that are ordered closer to the end.

Reductions are "a class of search heuristics that decrease the depth to
which a certain branch of the tree is searched, also interpreted as
negative extension." Relies on move ordering being good enough that late
moves are genuinely unlikely to be best (the same prerequisite PVS
already depends on).

## Eligibility — moves commonly *not* reduced

- Depth < 3 (the common minimum-depth threshold before LMR applies at
  all).
- Captures and promotions.
- Moves made while in check.
- Moves giving check / causing extensions.
- PV-node moves in a PVS search.
- Killer moves, passed-pawn moves, moves with good relative history
  values, moves of threatened pieces to safety — a longer list of
  "uncommon"/more-advanced exclusions not every engine implements.

> popularized by Fruit and Glaurung [...] Spark [...] all moves, except
> the first one, can be reduced by up to two plies depending on static
> criteria

## Reduction amount

Modern engines (Obsidian, Weiss, Ethereal, Halogen) use logarithmic
formulas in depth and move count (e.g. Ethereal's quiet-move formula
`0.7844 + ln(depth)*ln(moves)/2.4696`). No single simple/classical
formula is given — the page's own description of the *original*
Fruit/Glaurung-era technique is just "reduce late moves", with Spark's
"up to two plies" as the only concrete small number quoted. Zeus uses a
**fixed R = 1** (the simplest possible, below even Spark's "up to two"),
consistent with this project's pattern of starting at the plainest
version of each technique (fixed-R null move pruning, 1/4-pawn
aspiration window, etc.) before any adaptive/formula-based refinement.

## Re-search strategy

> Classical implementation: re-search at full depth if reduced search
> exceeds alpha.

No pseudocode is given for exactly how this composes with PVS's own
null-window-then-full-window structure. Zeus's composition (documented
as its own choice, not from this page): reduced-depth null-window search
first; if it beats alpha, re-search at **full depth** with the same null
window; only if *that* also lands inside `(alpha, beta)` does PVS's
existing full-window re-search happen — a three-tier cascade that only
ever spends more search effort than the depth-R baseline when a late
move actually turns out to be worth a closer look.

## Zeus usage

`crates/zeus-engine/src/search.rs::alpha_beta` only (not
`search_root_pass` — the root is effectively always a PV node, one of
this page's own listed "not reduced" cases, so scoped out for this
basic implementation; not `quiesce`, same scope boundary as every prior
search-enhancement stage). A move is eligible when: `depth_left >= 3`
(remaining depth *before* the reduction, the page's own threshold), its
move index in the ordered list is `>= 4` (Zeus's own "late" cutoff — not
given a number by this page), it is not a capture/promotion
(`capture_score(...).is_some()`), the side to move is not in check, and
it is not the transposition-table move. Killers, checks given, and the
longer "uncommon" exclusion list are **not** implemented this stage —
documented scope choice, same footing as every other "basic version
first" decision in this project.
