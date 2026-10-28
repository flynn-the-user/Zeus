---
source: https://www.chessprogramming.org/Move_Ordering
retrieved: 2026-10-28
---

# Move Ordering

## Full recommended sequence (per this page)

1. PV-move from the previous iterative-deepening iteration
2. Hash move from the transposition table
3. Winning captures/promotions
4. Equal captures/promotions
5. Killer moves (non-capture)
6. Non-captures sorted by history heuristic
7. Losing captures

> most programs use a selection sort approach rather than sorting the
> entire move list upfront

## What Zeus implements at this stage, and what's deferred

Zeus has neither a transposition table nor a PV-table yet (both arrive
with the next roadmap stage, where this page notes the PV move is often
provided "implicitly by the transposition table"), nor a history
heuristic, nor SEE (so no winning/losing-capture split, no "equal
captures" tier). Per this page's own "Capture Ordering Without Advanced
Scoring" guidance, plain MVV-LVA stands on its own as a simpler
alternative to the winning/equal/losing split. Zeus's ordering this stage
is therefore the subset: **captures (MVV-LVA) → killer moves → other
quiet moves** — tiers 3/4/7 collapsed into one MVV-LVA-ordered group,
tiers 1/2/6 deferred to later stages (noted in `ROADMAP.md`).

Zeus uses a full sort of the scored move list rather than a selection
sort, for implementation simplicity — a legitimate engineering
simplification given move lists here are small (at most a few dozen legal
moves), not a correctness difference from this page's algorithm, and
noted as a deviation from the "most programs" guidance above.

## Zeus usage

`crates/zeus-engine/src/search.rs::order_moves`: collects legal moves,
scores each (MVV-LVA for captures per `Reference/search/MVV-LVA.md`,
killer-slot bonus for matching quiets per
`Reference/search/Killer_Heuristic.md`, 0 otherwise), sorts descending.
Used in `alpha_beta`'s root and recursive move loops; not used inside
`quiesce` for killers (captures there are still ordered by MVV-LVA, since
that part of the heuristic doesn't depend on ply-indexed state).
