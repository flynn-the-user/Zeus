---
source: https://www.chessprogramming.org/Aspiration_Windows
retrieved: 2026-10-30
---

# Aspiration Windows

> a way to reduce the search space in an alpha-beta search

Search a narrow `[alpha, beta]` window centered on a predicted score
(typically the previous iterative-deepening iteration's score) instead
of the full `(-infinity, +infinity)` window. A narrower window produces
more cutoffs and a faster search, but a score outside the window ("fail
low" if `score <= alpha`, "fail high" if `score >= beta`) only proves a
bound, not the true value, and requires a re-search.

## Window size

> Typical window sizes are 1/2 to 1/4 of a pawn on either side of the
> guess.

## Re-search: widen only the side that failed

> if the initial window is `[g - 1/4, g + 1/4]` and the search fails
> high, the next window becomes `[g - 1/4, g + 1]` [...] the bound that
> didn't fail is unchanged

Explicitly warned against re-searching with a window based on the new
bound alone (e.g. `[g + 1/4, g + 1]` after a fail-high) — "a fully
fledged search is typically full of search instability, and it will
often happen that the above re-search will fail low!" Keep the
non-failing bound fixed at its original value and only move the failing
one.

> Modern engines [...] start with a rather small aspiration window, and
> increase the bound that fails in an exponential fashion.

No pseudocode is given. The worked example (1/4 pawn -> 1 pawn on a
fail-high) is a 4x widening of that side's margin from center.

## Caveats

- **Search instability**: re-searches with different windows "can leave
  contradicting results", attributed partly to transposition table
  interactions — not specially handled by Zeus beyond the standard TT
  depth/bound logic already in place
  (`Reference/search/Transposition_Table.md`), which doesn't need
  changes for this (a stale bound from a narrower window simply doesn't
  trigger an incorrect cutoff for a wider one, by the existing rules).
- **PVS interaction**: flagged as having "additional complications" on
  its own dedicated page, not fetched — Zeus doesn't have PVS yet (it's
  a separate, later roadmap stage), so out of scope here.
- No guidance on what to do if widening keeps failing. Zeus falls back
  to the full `(-infinity, +infinity)` window after a small, fixed
  number of widenings, purely as its own termination guarantee — not
  from this page.

## Zeus usage

`crates/zeus-engine/src/search.rs::search_root_once`: depth 1 (no prior
score to center on) always searches the full window, as does any depth
whose widening exhausts the fallback count above. From depth 2 on,
starts at the previous iteration's score ± 25 centipawns (the 1/4-pawn
end of the page's stated range), and on a fail, widens *only* the
failing side by 4x its current margin from the guess each re-search (the
page's own worked-example ratio), re-searching at the same depth with
the other side held fixed, exactly as described above.
