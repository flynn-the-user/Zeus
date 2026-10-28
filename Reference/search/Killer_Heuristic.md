---
source: https://www.chessprogramming.org/Killer_Heuristic
retrieved: 2026-10-28
---

# Killer Heuristic

> a dynamic, path-dependent move ordering technique [that] consider[s]
> moves which caused a beta-cutoff in a sibling node as killer moves and
> orders them high on the list.

> if a program decides that expelling a black bishop from b4 by a move
> a2-a3 is good, then it is likely to work whatever Black played on the
> previous move

## Storage

> When a node fails high, a quiet move that caused a cutoff is stored in
> a table indexed by ply, typically containing two or three moves per
> ply [...] most of the cutoffs come from the first killer slot [...] the
> second slot preserves important defensive plans amid tactical noise.

Only **quiet** (non-capture) moves are stored — captures are already
ordered by MVV-LVA (`Reference/search/MVV-LVA.md`).

## Ordering priority

> killer moves usually come right after the hash move and good captures
> [...] sometimes killer moves are sorted below all the captures,
> sometimes - between equal and losing captures.

Zeus has no hash move or winning/losing-capture split yet (no
transposition table, no SEE — both later roadmap stages), so the
simplified ordering is: all captures (MVV-LVA) first, then killers, then
other quiet moves.

No pseudocode is given on this page.

## Zeus usage

`crates/zeus-engine/src/search.rs::SearchContext.killers`: 2 slots per
ply (the page's "two or three", and explicitly the slot count where "most
of the cutoffs come from the first... second slot preserves" — 2 is the
minimum that distinguishes those two roles). Indexed by ply-from-root
(`alpha_beta` now threads an explicit `ply: u32` parameter, separate from
`depth_left`, for this). On a beta cutoff caused by a non-capture move,
it's inserted at slot 0 for that ply, pushing the previous slot-0 killer
(if different) into slot 1. Scoped to the main search only — not applied
inside `quiesce`, which this page doesn't address and which Zeus's
evasion-search branch treats as out of scope for this stage.
