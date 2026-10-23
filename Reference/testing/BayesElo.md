---
source-cpw: https://www.chessprogramming.org/Match_Statistics
source-primary: https://www.remi-coulom.fr/Bayesian-Elo/
source-build: https://github.com/ddugovic/BayesianElo (mirror of Rémi Coulom's GPL-licensed source, used only to build the unmodified `bayeselo` CLI binary as an external tool; not vendored into this repo, not a basis for any Zeus engine code)
retrieved: 2026-10-24
---

# BayesElo

`Reference/testing/Match_Statistics.md` (chessprogramming.org) explicitly
names and links this tool:

> "BayesElo builds tournament-statistics from a pgn-file" by Rémi Coulom

This is why `zeus-bench`'s tournament-mode Elo computation shells out to the
real `bayeselo` CLI rather than only using the simple pairwise formula in
`crates/zeus-bench/src/elo.rs` (which is still used for the quick
single-pair printout, but the authoritative, logged number for multi-anchor
tournaments now comes from this tool).

## Model

- **Generalized Bradley-Terry model** (David R. Hunter, "MM Algorithms for
  Generalized Bradley-Terry Models", 2004) extended to handle draws and a
  white-piece-advantage term, simultaneously fitting ratings for every
  player in a pool of pairwise game results rather than extrapolating one
  opponent at a time.
- **Bayesian prior**: `P(Elos|Results) ∝ P(Results|Elos) * prior`, which
  regularizes ratings instead of letting a small/extreme sample (e.g. a
  shutout) produce an undefined or wildly unstable estimate — the exact
  failure mode `crates/zeus-bench/src/elo.rs::EloEstimate::AllLosses/AllWins`
  has to punt on for a single pairing.
- **Minorization-Maximization (MM) algorithm** to find the maximum-a-
  posteriori ratings iteratively.
- Fitted nuisance parameters: white-piece advantage (~32.8 Elo in Coulom's
  own data) and a draw-elo parameter (~97.3 Elo), estimated from the data
  itself rather than assumed.

## CLI session (from the tool's own documented usage)

```
$ bayeselo
ResultSet>readpgn results.pgn
N game(s) loaded, M game(s) with unknown result ignored.
ResultSet>elo
ResultSet-EloRating>mm
Iteration 100: <convergence residual>
ResultSet-EloRating>exactdist
ResultSet-EloRating>ratings
Rank Name          Elo    +    - games score oppo. draws
   1 PlayerA       120  167  148     8   75%     5   50%
   ...
```

`readpgn` loads a PGN file whose games only need valid `[White]`, `[Black]`,
`[Result]` tags (and any other standard tags) to be counted — the rating
computation uses the result, not the move text.

## Build

GPL-licensed C++ source, built with a plain `make` (see
`scripts/setup_bayeselo.sh`). The resulting binary is treated purely as an
external measurement tool invoked as a subprocess by `zeus-bench`
(`crates/zeus-bench/src/bayeselo.rs`) — it is not linked into, vendored
into, or used as a model for any Zeus engine code, exactly like the
Stockfish binary used as a calibration opponent.

## Zeus usage

`zeus-bench`'s tournament mode writes every game (Zeus and every Stockfish
anchor, all pairs) to a single PGN file, then drives `bayeselo` through
exactly the command sequence above (`readpgn` -> `elo` -> `mm` ->
`exactdist` -> `ratings`) and parses the `ratings` table to get each
participant's Elo with error bars, logged in `ELO_LOG.md`.
