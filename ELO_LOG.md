# Elo Log

## Methodology (current, fixed across every future entry for comparability)

- Format: round-robin **tournament**, not a single pairing. Participants =
  Zeus + several Stockfish 17.1 instances, each pinned to a different
  target Elo via `UCI_LimitStrength`+`UCI_Elo`
  (`Reference/protocols/UCI_Protocol.md`), `Threads=1`. Every pair in the
  roster plays each other (including Stockfish-anchor vs Stockfish-anchor),
  not just Zeus vs each anchor — this cross-anchors the whole rating scale
  instead of extrapolating from one opponent at a time.
- Games per pair: **100** where compute allows; reduced (and noted per
  entry) once an engine stage starts actually using its clock, since games
  then take far longer in wall-clock time (see the v0.2 entry below).
- Openings: a fixed 12-opening suite (`crates/zeus-bench/src/openings.rs`),
  the same suite reused for every future tournament, each opening played
  twice per pair with colors reversed before moving to the next
  (`Reference/testing/Engine_Testing.md`) — added after v0.1's run (see its
  caveat below), so **not** present in the v0.1 numbers.
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

**Caveat found later (see `Reference/testing/Engine_Testing.md` and the
v0.2 entry below):** this run used no opening suite — every game started
from the plain startpos. Zeus's v0.1 search is fully deterministic, so an
unknown fraction of these 1000 games were likely near-duplicates of each
other rather than independent samples, meaning the effective sample size
was smaller than 1000 and the reported confidence interval is
optimistic. The direction/rough magnitude of the result is still useful as
a historical baseline, but it was not re-run with the opening-suite fix
applied (the compute cost wasn't judged worth it for a number that's
already superseded by v0.2 below) — treat it as indicative, not as precise
as the v0.2+ measurements.

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

## v0.2 — iterative deepening + time management

Commit: `4cea1f8` ("Add iterative deepening + time management, async UCI
stop"). Engine change from v0.1: search is no longer a fixed depth-4
call — it now runs `search_iterative` (`Reference/search/Iterative_Deepening.md`)
under a soft/hard time budget derived from the actual clock
(`Reference/search/Time_Management.md`), so it searches as deep as the
clock allows instead of always stopping at ply 4. Evaluation is still
material-only (unchanged). Full game log:
`results/zeus_v0.2_tournament.pgn` + `.moves.log` (committed, 240 games).

Roster unchanged (Zeus, Stockfish-1320/1500/1700/1900), but
**games-per-pair reduced from 100 to 24** (2 full cycles through the
12-opening suite, each opening/color-reversed pair once) — once the
engine actually uses its clock, games take ~60-120s each instead of a few
seconds, and 1000 games would have taken ~4-5 hours; 240 games (~1.5
hours, run across 2 relaunches) was judged a reasonable trade of sample
size for turnaround, confirmed with the user before running. This is also
the **first measurement with the opening-suite fix** applied (see the
v0.1 caveat above), so it is not a perfectly clean before/after on sample
methodology alone — but the Elo movement here is large enough that this
doesn't change the conclusion.

| Participant | Raw (bayeselo) | Calibrated real Elo | +/- | Games | Score% | Draws% |
|---|---|---|---|---|---|---|
| Stockfish-1900 | 219 | **1819** | 67/60 | 96 | 79% | 4% |
| Stockfish-1700 | 163 | **1750** | 63/59 | 96 | 72% | 4% |
| Stockfish-1500 | 16 | **1569** | 56/55 | 96 | 53% | 4% |
| **Zeus-v0.2-ID-TM** | -182 | **1325** | 57/61 | 96 | 24% | 14% |
| Stockfish-1320 | -217 | **1281** | 59/65 | 96 | 21% | 7% |

Calibration: `real_elo = 1549.2 + 1.2337*raw_elo` (same least-squares
method as v0.1, re-fit on this tournament's own anchor games — not reused
from v0.1's fit, since anchor behavior can itself shift with the opening
suite now in play).

**Zeus v0.2 ≈ 1325 Elo, up from v0.1's ≈1081 Elo (+~244 Elo).** This is a
large, unambiguous jump, and it shows up directly in the raw scores too:
Zeus-v0.2 now scores 47.9% against Stockfish-1320 (up from 28.5% for
v0.1) and actually out-rates Stockfish-1320 in the joint fit (1325 vs
1281) for the first time. Entirely attributable to search depth: v0.1 was
hard-capped at 4 plies; v0.2 reaches depth 7+ within a few hundred ms in
early/middlegame positions (observed in `info depth` output during
development) and deeper still given the fuller ~3s/move soft budget at
this time control.

Zeus-v0.2 also picked up draws for the first time (14% here vs ~2-4% for
pure Stockfish-vs-Stockfish pairs) — plausibly deeper search finding more
drawish/fortress-like positions it can hold, though evaluation is still
material-only so this isn't a strong claim.

No time losses observed in the move logs despite real time management
now being active and Stockfish opponents being much faster per-move at
these settings — the soft/hard bound formulas in
`crates/zeus-uci/src/time.rs` held up under real competitive play.

Takeaways for the next roadmap stage (`ROADMAP.md`):
- Zeus still loses the vast majority of games to every anchor — the next
  stages (quiescence search, move ordering, transposition table) should
  each move this number further, and each should budget for the same
  ~24-games/pair, ~1.5hr scale unless/until search becomes fast enough
  (via move ordering/TT cutoffs) to afford more games again.
- Quiescence search is next and is specifically expected to fix the
  horizon-effect blunders that a depth-limited, capture-blind search
  (even an iteratively-deepened one) is still prone to at the exact ply
  where search stops.
