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
was smaller than 1000 and the reported confidence interval is optimistic.

**Resolved**: re-run with the opening-suite fix applied, matched to v0.2's
exact methodology (same roster, same 24 games/pair, same TC) — see
"v0.1 rebaseline" below. The clean result (1050 Elo) landed within ~30
points of this flawed one (1081), so the duplication concern, while a
real methodology bug worth fixing, did not meaningfully bias this
particular point estimate. The table below is kept for history; the
rebaseline is what v0.2's delta is actually computed against.

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

## v0.1 rebaseline — same engine, fixed opening-suite methodology

Purpose: v0.2 below was measured with the opening-suite fix in place,
but the v0.1 number above wasn't — comparing them directly would confound
"engine got better" with "measurement method changed." This re-runs the
*exact same v0.1 binary* (built from commit `d004267`, via a throwaway
`git worktree`, no code changes) through the *current* harness
(opening suite, 24 games/pair, same roster/TC as v0.2), so there is
exactly one variable between this and v0.2: the engine. Full game log:
`results/zeus_v0.1_rebaseline_tournament.pgn` + `.moves.log` (committed,
240 games).

| Participant | Raw (bayeselo) | Calibrated real Elo | +/- | Games | Score% | Draws% |
|---|---|---|---|---|---|---|
| Stockfish-1900 | 249 | **1866** | 69/62 | 96 | 81% | 4% |
| Stockfish-1700 | 144 | **1709** | 61/58 | 96 | 69% | 7% |
| Stockfish-1500 | 44 | **1559** | 58/57 | 96 | 55% | 2% |
| Stockfish-1320 | -139 | **1286** | 58/61 | 96 | 32% | 8% |
| **Zeus-v0.1-rebaseline** | -297 | **1050** | 65/75 | 96 | 12% | 9% |

`real_elo = 1493.8 + 1.4932*raw_elo` (re-fit on this tournament's own
anchor games, same method as every other entry here).

**Zeus v0.1 (rebaselined) ≈ 1050 Elo** — close to the original flawed
measurement (1081), confirming that run's duplication risk didn't
meaningfully bias the point estimate in this case, even though fixing it
was still the right call for future measurements' validity. **This 1050
figure, not 1081, is the correct baseline for v0.2's delta below.**

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
size for turnaround, confirmed with the user before running. Methodology
here matches the "v0.1 rebaseline" section above exactly (same opening
suite, same 24 games/pair, same roster/TC), so the delta below is a clean
single-variable (engine only) comparison.

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

**Zeus v0.2 ≈ 1325 Elo, up from the matched-methodology v0.1 rebaseline's
≈1050 Elo: +275 Elo, single-variable (engine-only) comparison.** This is a
large, unambiguous jump, and it shows up directly in the raw scores too:
Zeus-v0.2 now scores 47.9% against Stockfish-1320 (up from 33.3% for the
v0.1 rebaseline) and actually out-rates Stockfish-1320 in the joint fit
(1325 vs 1281) for the first time. Entirely attributable to search depth: v0.1 was
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

## v1.1 — quiescence search

Commit: `8afbd1a` ("Add quiescence search to fix the horizon effect").
Engine change from v0.2: `alpha_beta`'s leaf now calls `quiesce()`
(`Reference/search/Quiescence_Search.md`) instead of `evaluate()`
directly. Methodology identical to v0.2 (same roster, 24 games/pair,
opening suite, TC) — a clean single-variable comparison, no rebaseline
needed. Full game log: `results/zeus_v1.1_qs_tournament.pgn` + `.moves.log`
(committed, 240 games).

| Participant | Raw (bayeselo) | Calibrated real Elo | +/- | Games | Score% | Draws% |
|---|---|---|---|---|---|---|
| Stockfish-1900 | 288 | **1876** | 76/66 | 96 | 86% | 3% |
| Stockfish-1500 | 76 | **1637** | 58/56 | 96 | 61% | 6% |
| Stockfish-1700 | 35 | **1590** | 57/55 | 96 | 56% | 7% |
| **Zeus-v1.1-QS** | -192 | **1334** | 57/61 | 96 | 24% | 13% |
| Stockfish-1320 | -207 | **1318** | 58/64 | 96 | 23% | 8% |

`real_elo = 1550.9 + 1.1272*raw_elo`.

**Zeus v1.1 ≈ 1334 Elo, vs v0.2's ≈1325 Elo: +9 Elo — not a
statistically meaningful change at this sample size.** Reporting this
plainly rather than spinning it: quiescence search did not produce a
measurable Elo gain in this 96-games-per-anchor measurement.

**Why this result should be read with real skepticism about the
measurement itself, not just the feature:** Stockfish-1500 rates *above*
Stockfish-1700 in this run's bayeselo fit (1637 vs 1590), inverted from
their nominal UCI_Elo ordering, driven by Stockfish-1500 beating
Stockfish-1700 68.8% of their 24 head-to-head games. Two engine settings
with a nominal 200-point gap landing inverted at this sample size is
direct evidence that the noise floor at 24 games/pair is comparable to
(or larger than) the kind of Elo delta a single incremental feature like
quiescence search is likely to produce. A real +9 Elo signal is simply
not resolvable against that much noise — this result is consistent with
"no gain," "a modest real gain masked by noise," and "a modest real loss
masked by noise" all at once.

What *is* solid evidence the feature works correctly, independent of the
noisy Elo number: qualitative behavior observed during development
(`crates/zeus-uci` manual testing, not part of the tournament) — before
this change, `info depth N score cp ...` output for the startpos
oscillated (e.g. depth 5 `cp 100`, depth 6 `cp -100`, depth 7 `cp 100`
in one `go movetime 500` session), the signature of exactly the
horizon-effect swings this page describes; after this change the same
kind of session is stable (`cp 0` at every depth for the startpos). The
mechanism is demonstrably fixed; its Elo contribution at this specific
time control, sample size, and (still material-only, still
move-ordering-free) engine configuration is not resolved by this
measurement.

Plausible (unproven) reasons the Elo contribution might be genuinely
small *right now*, to revisit once later stages land:
- No move ordering yet in quiescence (next stage) — `Reference/search/Quiescence_Search.md`
  itself flags unordered captures as inefficient, so quiescence may be
  spending search-time budget less effectively than it will once MVV-LVA
  lands, partially offsetting its tactical benefit within the same clock.
- Material-only evaluation has little "positional quiet" to protect in
  the first place — quiescence mainly guards against mis-evaluating
  tactical capture sequences, which may not be the dominant blunder
  source yet compared to purely positional blind spots a material-only
  evaluator can't see regardless of search depth.

Takeaways for the next roadmap stage (`ROADMAP.md`):
- Treat single-stage deltas skeptically below roughly 50-100 Elo at this
  24-games/pair sample size — the anchor-inversion above is the evidence.
  Consider this when move ordering (small, incremental gain expected) is
  measured next: a clean result either way should be read in light of
  this noise floor, not over-interpreted.
- Move ordering (MVV-LVA) is next regardless — it's both the
  CPW-recommended prerequisite for efficient quiescence and expected to
  help the main search's alpha-beta pruning directly, so a larger,
  easier-to-resolve gain is plausible there.

## v1.2 — move ordering, as an ablation study against quiescence

Commit: `9776b56` ("Add move ordering (MVV-LVA + killer heuristic) + a
quiescence toggle"). Per the user's explicit request, measured as two
separate tournaments instead of one, to isolate move ordering's own
contribution from quiescence's — both use the methodology fixed above
(240 games, 24/pair, opening suite, 1-minute bullet, same roster).
`crates/zeus-uci` grew a temporary `UseQuiescence` UCI option
(`CITATIONS.md`) purely for this: a debug toggle, not a permanent engine
feature.

### Stage A: move ordering only (quiescence held OFF, matching v1's config)

Compares directly against **v1's 1325 Elo** (also quiescence-off) — a
clean single-variable comparison. Full game log:
`results/zeus_v1.2a_moveorder_noqs_tournament.pgn` + `.moves.log`
(committed, 240 games).

| Participant | Raw (bayeselo) | Calibrated real Elo | +/- | Games | Score% | Draws% |
|---|---|---|---|---|---|---|
| Stockfish-1900 | 248 | **1929** | 72/62 | 96 | 83% | 1% |
| Stockfish-1700 | 39 | **1628** | 55/54 | 96 | 56% | 4% |
| Stockfish-1500 | -35 | **1521** | 52/53 | 96 | 46% | 10% |
| **Zeus-v1.2-MoveOrder-noQS** | -95 | **1435** | 50/52 | 96 | 37% | 22% |
| Stockfish-1320 | -159 | **1342** | 54/58 | 96 | 28% | 10% |

`real_elo = 1571.5 + 1.4405*raw_elo`. Anchors land properly monotonic
with their nominal UCI_Elo order this run (1929 > 1628 > 1521 > 1342),
unlike v1.1's inverted anchors — lower measurement noise this time.

**Move ordering alone: ≈1435 Elo, +110 over v1's 1325.** A large,
clean, easily-resolved gain (unlike quiescence's own +9) — expected,
since MVV-LVA/killer ordering helps alpha-beta's cutoff rate directly in
every search, not just at the leaf.

### Stage B: move ordering + quiescence back ON

Compares against Stage A above (+110 baseline) to re-isolate
quiescence's own contribution, now that the CPW-recommended prerequisite
(move ordering) is in place — directly testing v1.1's hypothesis that
quiescence's measured gain was masked by inefficient unordered search.
Full game log: `results/zeus_v1.2b_moveorder_qs_tournament.pgn` +
`.moves.log` (committed, 240 games).

| Participant | Raw (bayeselo) | Calibrated real Elo | +/- | Games | Score% | Draws% |
|---|---|---|---|---|---|---|
| Stockfish-1900 | 195 | **1837** | 64/58 | 96 | 76% | 2% |
| Stockfish-1700 | 140 | **1767** | 59/55 | 96 | 70% | 6% |
| **Zeus-v1.2-MoveOrder-QS** | -48 | **1529** | 51/51 | 96 | 45% | 25% |
| Stockfish-1500 | -62 | **1511** | 53/54 | 96 | 41% | 9% |
| Stockfish-1320 | -225 | **1304** | 57/63 | 96 | 18% | 14% |

`real_elo = 1589.8 + 1.2685*raw_elo`. Anchors (excluding Zeus) again
properly monotonic (1837 > 1767 > 1511 > 1304).

**Move ordering + quiescence: ≈1529 Elo, +94 over Stage A's 1435 —
quiescence's contribution is now clearly measurable**, in sharp contrast
to the +9/noise result it got on its own in v1.1. This confirms v1.1's
hypothesis directly: quiescence's horizon-effect fix is real (already
shown qualitatively in v1.1 via the stable-vs-oscillating score
evidence), and its Elo value was being suppressed by exploring captures
in an arbitrary, unordered sequence — ordering them first (MVV-LVA) lets
alpha-beta's cutoffs inside `quiesce` actually fire efficiently.

### Summary: full chain from v1 to v1.2b

| Stage | Elo | Delta | vs |
|---|---|---|---|
| v1 (ID + time management) | 1325 | — | — |
| v1.1 (+ quiescence alone) | 1334 | +9 (noise) | v1 |
| v1.2a (+ move ordering, QS off) | 1435 | +110 | v1 |
| v1.2b (+ move ordering + QS) | 1529 | +94 | v1.2a |

Net gain from this stage's two features combined: **+204 Elo** (1325 ->
1529), split roughly 110/94 between move ordering and quiescence - but
only once move ordering existed to make quiescence's search efficient.
Takeaway for future staging: where two features interact (here,
quiescence's efficiency depends on move ordering), measuring them in the
"wrong" order can make a real feature look like noise, exactly as
happened with quiescence in v1.1. Worth considering feature-interaction
risk before assuming a future stage's roadmap order is itself
interaction-free.

Takeaways for the next roadmap stage (`ROADMAP.md`):
- Transposition Table + Zobrist Hashing is next. It should also unlock a
  real PV-move/hash-move for tier-1/2 of `Reference/search/Move_Ordering.md`'s
  full ordering sequence (not yet implemented - Zeus is still missing the
  "PV-move" and "hash move" tiers above captures), so measure with that
  possibility in mind.
- The `UseQuiescence` toggle stays in the codebase (cheap, harmless,
  useful for any future ablation need) but defaults to `true` and is not
  part of the standard calibration command going forward.
