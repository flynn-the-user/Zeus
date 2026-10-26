---
source: https://www.chessprogramming.org/Engine_Testing
retrieved: 2026-10-26
---

# Engine Testing — Opening selection

> During testing the engines should ideally play the same style of
> openings they would play in a normal tournament, so not to optimize them
> for different types of positions.

> One option is to use the engines own opening book or one can use opening
> suites, a set of quiet test positions. In the latter case the same
> opening suite would be used for each tournament conducted and
> furthermore each position is played a second time with colors reversed.

> With these measures one can try to minimize the disparity between tests
> caused by different openings.

This page does not name a specific suite (e.g. a "2moves"/"8moves"-style
file) or give a target opening count.

## Why this matters for Zeus specifically

`crates/zeus-engine`'s search is fully deterministic (no randomness
anywhere) and, before this fix, `zeus-bench` always started every game
from the plain startpos with no book. Two engines that are each
deterministic given a position will tend to replay near-identical games
when repeatedly started from the same position — so "N games" was not
really N independent samples. This was caught from the repeated `pv a2a3`
first move showing up across supposedly-separate games in earlier `info`
output.

## Zeus usage

`crates/zeus-bench/src/openings.rs` is a small, fixed opening suite (named,
standard, quiet tabiyas from mainstream opening theory — not an engine
algorithm decision, so not itself CPW-sourced, same footing as the
standard chess draw rules in `src/game.rs`). Per this page: the **same**
suite is reused for every future tournament (comparability, like the fixed
time control), and each opening is played **twice per pair, colors
reversed** (`crates/zeus-bench/src/main.rs`'s game-scheduling loop) before
moving to the next opening. Opening moves are applied via `position ...
moves ...` without going through a timed `go`, so they don't consume
either side's clock — only the moves after the opening are played for
real, under the real time control.
