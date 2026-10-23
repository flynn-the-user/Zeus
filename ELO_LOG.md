# Elo Log

## Methodology (current, fixed across every future entry for comparability)

- Format: round-robin **tournament**, not a single pairing. Participants =
  Zeus + several Stockfish 17.1 instances, each pinned to a different
  target Elo via `UCI_LimitStrength`+`UCI_Elo`
  (`Reference/protocols/UCI_Protocol.md`), `Threads=1`. Every pair in the
  roster plays each other (including Stockfish-anchor vs Stockfish-anchor),
  not just Zeus vs each anchor — this cross-anchors the whole rating scale
  instead of extrapolating from one opponent at a time.
- Games per pair: **100**, alternating colors.
- Time control: **1-minute bullet (60s + 0s increment)**, enforced by the
  harness itself (real wall-clock deadline per move), not trusted to either
  engine — chosen specifically so a large, statistically meaningful game
  count is feasible to run in parallel.
- Rating computation: the real **bayeselo** CLI (Rémi Coulom; generalized
  Bradley-Terry model + Bayesian prior, MM algorithm —
  `Reference/testing/BayesElo.md`, which `Reference/testing/Match_Statistics.md`
  itself names and links) fit jointly over every game in the tournament.
  The simple single-pair logistic formula in `crates/zeus-bench/src/elo.rs`
  is still printed per-pair as a quick sanity cross-check, but bayeselo's
  joint fit is the number of record.
- Command: `zeus-bench --roster 1320,1500,1700,1900 --games-per-pair 100
  --concurrency 5 --base-ms 60000 --inc-ms 0`, run on a 10-core Apple
  Silicon Mac. `tools/bayeselo` built via `scripts/setup_bayeselo.sh`.

## Superseded: v0.1 single-pair calibration (first measurement, pre-tournament methodology)

Commit: `d004267` ("Zeus v0.1: negamax+alpha-beta search, material eval, UCI loop").

| Opponent UCI_Elo | N | W | D | L | Score | Estimated Zeus Elo (95% CI) |
|---|---|---|---|---|---|---|
| 1320 (Stockfish's declared floor) | 40 | 9 | 3 | 28 | 26.2% | **1141** (984 .. 1249) |

Stockfish's `UCI_Elo` floor (1320) was queried from the engine itself at
run time, not assumed. 1320 is the lowest rung Stockfish supports via this
option, so this is also the lowest rung we can calibrate against; Zeus
v0.1 scoring 26.2% against it is why a finite estimate exists at all at
this stage (0% would have given no finite number, see
`crates/zeus-bench/src/elo.rs::EloEstimate::AllLosses`).

Takeaways for the next roadmap stage (`ROADMAP.md`):
- No losses on time were observed in the raw game log — the fixed shallow
  search depth (4 plies) made clock-ignorance safe for this stage, as
  anticipated in `crates/zeus-uci/src/main.rs`'s `DEFAULT_DEPTH` comment.
- One game hit `MaxPlies` (240) and two hit `InsufficientMaterial` rather
  than a clean mate/stalemate — expected for a material-only evaluator with
  no sense of progress in won endgames.
- Next stage (iterative deepening + time management) should re-run this
  exact command and the same rung for a before/after comparison.

This single-pair, 10+0.1, naive-formula run is kept here for history but is
**superseded** by the tournament methodology above per the user's request
to move to a round-robin/bayeselo/bullet methodology. See the next section
for the first tournament result.

## v0.1 — tournament calibration (round-robin + bayeselo, bullet)

Engine unchanged from the superseded measurement above: negamax+alpha-beta,
fixed depth 4, material-only eval. Full raw game log:
`results/zeus_v0.1_tournament.pgn` (committed, 1000 games).

Roster: Zeus-v0.1, Stockfish-1320, Stockfish-1500, Stockfish-1700,
Stockfish-1900. 10 pairs x 100 games = 1000 games, 1+0 bullet, all played
to completion (checkmate/stalemate/50-move/repetition/insufficient
material — no time losses observed).

bayeselo's `ratings` output is on an arbitrary internal scale (no fixed
zero point — see `Reference/testing/BayesElo.md`), so
`crates/zeus-bench/src/bayeselo.rs::fit_real_elo_scale` maps it onto real
Elo via a least-squares fit against the 4 known Stockfish `UCI_Elo`
anchors (`real_elo = 1500.2 + 1.2225*raw_elo`), rather than anchoring to
just one of them:

| Participant | Raw (bayeselo) | Calibrated real Elo | +/- | Games | Score% | Draws% |
|---|---|---|---|---|---|---|
| Stockfish-1900 | 316 | **1886** | 38/35 | 400 | 85% | 2% |
| Stockfish-1700 | 155 | **1690** | 32/31 | 400 | 68% | 2% |
| Stockfish-1500 | 39 | **1548** | 30/30 | 400 | 55% | 2% |
| Stockfish-1320 | -167 | **1296** | 31/32 | 400 | 30% | 4% |
| **Zeus-v0.1** | -343 | **1081** | 37/40 | 400 | 12% | 4% |

**Zeus v0.1 ≈ 1081 Elo** under this methodology (bullet, round-robin,
bayeselo-calibrated) — in the same ballpark as the superseded single-pair
measurement (1141), with the gap plausibly explained by the much faster
time control (1+0 bullet vs 10+0.1) giving less room for Zeus's search to
matter relative to Stockfish, plus this being a more statistically robust
multi-anchor fit rather than one noisy pairing.

Calibration-quality note: the fitted anchors aren't perfectly linear —
recovered Stockfish Elo (1296/1548/1690/1886) deviates from the nominal
UCI_Elo targets (1320/1500/1700/1900) by up to ~48 points. This says
Stockfish's own `UCI_LimitStrength` calibration isn't perfectly linear in
observed game outcomes at 1+0 bullet specifically (plausibly because
`UCI_Elo`'s documented range/calibration isn't guaranteed to hold at
extremely fast time controls) — worth keeping in mind when comparing
future Elo deltas, though it does not change which direction/roughly how
large a given stage's improvement is.

Operational note: the actual run needed 8 separate `zeus-bench` launches
(a background-process duration limit in the dev environment, unrelated to
Zeus or the methodology, killed each one after a few hundred games) — made
lossless by `crates/zeus-bench`'s incremental, flush-per-game PGN writing
and `read_existing_games`-based resume, which simply continues an
under-filled roster from wherever a previous run stopped.

Takeaways for the next roadmap stage (`ROADMAP.md`):
- No time losses despite the much faster 1+0 bullet control — consistent
  with the superseded run's finding that fixed shallow-depth search is
  safe without real time management, for now.
- Score drops sharply as opponent strength rises (28.5% -> 14% -> 4% -> 2%
  vs 1320/1500/1700/1900) — expected for a material-only, no-quiescence,
  no-move-ordering engine; gives a clear baseline to show stage-by-stage
  Elo gains against going forward.
- Next stage (iterative deepening + time management) should re-run this
  exact command/roster for a before/after comparison.
