---
source: https://www.chessprogramming.org/Transposition_Table
retrieved: 2026-10-30
---

# Transposition Table

A hash table, indexed by a position's Zobrist key
(`Reference/search/Zobrist_Hashing.md`), caching previous search results
so a position reached by a different move order doesn't get re-searched
from scratch.

## Entry contents

> Hash Key [...] Best Move [...] Depth [...] Score/Value [...] Bound Type
> [...] Age

Bound type, one of three:

- **Exact** (PV-node): the true value.
- **Lower bound** (Cut-node): the value is *at least* this high (a beta
  cutoff occurred; the true value could be higher).
- **Upper bound** (All-node): the value is *at most* this high (no move
  improved alpha; the true value could be lower).

## Probing

> A cutoff occurs when the stored entry's depth is greater than or equal
> to the current search depth AND one of these conditions holds: Entry
> type is EXACT; Entry type is LOWERBOUND and the score ≥ beta; Entry
> type is UPPERBOUND and the score ≤ alpha.

> Beyond cutoffs, best move/Refutation move information from the
> transposition table is the most important part of move ordering

I.e. even when an entry's depth is too shallow to trust for a cutoff, its
stored best move is still used first in move ordering.

## Storing

> EXACT: when a PV-node returns a score within the alpha-beta window.
> LOWERBOUND: when a cutoff occurs (score ≥ beta at a cut-node).
> UPPERBOUND: when no move improves alpha (score ≤ alpha at an all-node).

(Zeus's search is fail-soft, so "score ≥ beta"/"score ≤ alpha" are
evaluated against `best_value`, consistent with how
`Reference/search/Alpha-Beta.md`'s fail-soft cutoffs already work.)

## Replacement scheme

Several named: always-replace, depth-preferred, two-tier, bucket,
age-based. No pseudocode or a single recommended default is given.

## Zeus usage

`crates/zeus-engine/src/tt.rs`: a single fixed-size table (power-of-two
slot count, index = `hash & mask`), **always-replace** — the simplest of
the named schemes above and Zeus's own choice absent a page-recommended
default, revisit if a later stage's measurements suggest a smarter scheme
is worth it. Each slot stores the full 64-bit key (to detect index
collisions between different positions sharing the same low bits), depth,
score, bound, and best move, exactly the fields this page lists except
"age" (not implemented this stage — the table is cleared instead on
`ucinewgame`, a simpler substitute for now).

Probing/storing follow this page's rules exactly, applied only in the
main search (`alpha_beta`) — not `quiesce`, where "depth" doesn't have
the same meaning and this page doesn't address quiescence-table
interaction.

Sized via a `Hash` UCI option (MB) — the standard option name this
project's own `Reference/protocols/UCI_Protocol.md` already documents for
exactly this purpose, foreshadowed in `CITATIONS.md` since v0.1.
