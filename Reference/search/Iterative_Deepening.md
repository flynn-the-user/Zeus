---
source: https://www.chessprogramming.org/Iterative_Deepening
retrieved: 2026-10-26
---

# Iterative Deepening

> the program starts with a one ply search, then increments the search
> depth and does another search. This process is repeated until the time
> allocated for the search is exhausted.

## Stopping criterion

Time-based, not depth-based: "the program always has the option to fall
back to the move selected in the last iteration of the search" if time runs
out before a deeper iteration completes. I.e. a search that is aborted
mid-iteration must **discard that iteration's result** and report the best
move from the last *fully completed* iteration instead — an incomplete
alpha-beta pass has not proven its score is correct (a cutoff near the root
could still be hiding a refutation found deeper).

## Why it's faster than searching the target depth directly

> iterative deepening is faster than searching for the given depth
> immediately. This is due to dynamic move ordering techniques such as
> PV-, hash- and refutation moves determined in previous iteration(s), as
> well the history heuristic ... this move is searched first in the next
> iteration

(Zeus does not yet have a transposition table or move ordering — see
`ROADMAP.md` stages 3-4 — so this particular speed benefit is not realized
yet. ID is still adopted now because it is, per this page, "the basic time
management strategy in depth-first searches": it's what makes "search until
the clock says stop, not until a fixed depth" possible at all.)

No pseudocode is given on this page.

## Zeus usage

`zeus-engine::search::search_iterative` runs depth = 1, 2, 3, ... calling
the existing fail-soft alpha-beta (`Reference/search/Alpha-Beta.md`) at each
depth, under a deadline (`Reference/search/Time_Management.md`). If a depth
is aborted partway through, its result is discarded and the last complete
depth's move is kept, exactly as described above — with one addition not
covered by this page: at depth 1 there is no "previous iteration" to fall
back to, so the root loop also tracks the best move found among whichever
sibling root moves were evaluated before an abort, as a last-resort
fallback so the engine can never return no move at all. That fallback is
Zeus's own addition for robustness, not something this page specifies.
