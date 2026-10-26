---
source: https://www.chessprogramming.org/Time_Management
retrieved: 2026-10-26
---

# Time Management

## Time control types

- **Sudden death**: all remaining moves must be played in the remaining
  time. "Typically programs estimate the game will last further 25..40
  moves, and divide the remaining time by this number."
- **Cyclic**: a fixed number of moves (`movestogo` in UCI) must be made in
  a fixed amount of time.
- **Increment**: Fischer (unconditional increment added each move) vs
  Bronstein (never increases remaining time, only delays its decrease) —
  UCI's `winc`/`binc` map to Fischer-style increment.

## Basic formula (adopted by Zeus)

> a basic time management technique is to use `base / 20 + increment / 2`
> time per move. This is very competitive with advanced time management
> schemes, and is typically chosen as the foundation to build upon.

## Soft bound / hard bound

> introduce two levels of time allocation: one optimum time threshold
> (soft bound), and one maximum time threshold (hard bound). During
> search, the optimum time threshold is checked on each iteration of
> iterative deepening, while the maximum time threshold is checked
> periodically.

This page does **not** give a formula for deriving the hard bound from the
soft bound/remaining time — only that it exists and is checked more
frequently (mid-iteration, not just between iterations). Zeus's specific
hard-bound formula (`crates/zeus-uci/src/time.rs`) is therefore an
engineering choice, not drawn from this page, and is documented as such
where it's defined.

## Zeus usage

- Soft bound = `time_left/20 + increment/2` (the formula above), computed
  from `wtime`/`winc` or `btime`/`binc` depending on side to move, exactly
  as given — checked between iterations of `search_iterative`
  (`Reference/search/Iterative_Deepening.md`): if the soft bound has
  already elapsed, don't start another (deeper) iteration.
- Hard bound: Zeus's own choice, checked periodically (every 2048 nodes)
  *during* a search to abort a single iteration that's overrunning, per
  this page's two-level description.
- `movetime <x>`: soft = hard = `x` (UCI's own single explicit deadline,
  nothing to derive).
- `infinite` / no time info at all: no deadline — keep deepening until an
  explicit `stop` (`Reference/protocols/UCI_Protocol.md`) or a hard depth
  cap.
