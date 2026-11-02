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
- Roster: `2000,2300,2600` (3 anchors + Zeus = 4 participants, 6 pairs) as of v1.5's
  re-measurement below — this is now `zeus-bench`'s own default
  (`crates/zeus-bench/src/main.rs`), **replacing** the original
  1320/1500/1700/1900 spread per the user's explicit choice, so
  `--roster` doesn't need to be passed explicitly going forward unless
  overriding it. The switch happened once Zeus itself crossed ~2000 Elo
  and the old anchors stopped giving bayeselo's joint fit enough
  variance to calibrate the scale precisely — Zeus was winning 92-98%
  against all of them (the v1.5 entry below explains the finding that
  prompted this). Entries before the "v1.5 (2000+ roster)" one below
  used the original 4-anchor roster — noted per entry, not retroactively
  re-measured.
- Command: `zeus-bench --games-per-pair 100 --concurrency 5 --base-ms
  60000 --inc-ms 0` (roster now defaulted, see above), run on a 10-core
  Apple Silicon Mac. `tools/bayeselo` built via `scripts/setup_bayeselo.sh`.

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

## v1.3 — transposition table + Zobrist hashing

Commit: `3547ad9` ("Add transposition table with Zobrist hashing").
Engine change from v1.2b: `alpha_beta` probes/stores a transposition
table (`Reference/search/Transposition_Table.md`) keyed by a Zobrist hash
(`Reference/search/Zobrist_Hashing.md`), both for alpha-beta cutoffs and
a new hash-move ordering tier above MVV-LVA captures. Persists across
`go` calls within a game via a `Hash` UCI option (default 16MB).
Methodology unchanged (240 games, 24/pair, opening suite, 1-minute
bullet, same roster) — clean single-variable comparison against v1.2b's
1529 Elo, no rebaseline needed. Full game log:
`results/zeus_v1.3_tt_tournament.pgn` + `.moves.log` (committed, 240
games).

| Participant | Raw (bayeselo) | Calibrated real Elo | +/- | Games | Score% | Draws% |
|---|---|---|---|---|---|---|
| Stockfish-1900 | 192 | **1875** | 63/57 | 96 | 76% | 4% |
| **Zeus-v1.3-TT** | 78 | **1729** | 56/55 | 96 | 60% | 4% |
| Stockfish-1700 | 62 | **1709** | 56/55 | 96 | 58% | 6% |
| Stockfish-1500 | -68 | **1543** | 54/56 | 96 | 40% | 4% |
| Stockfish-1320 | -263 | **1293** | 63/74 | 96 | 16% | 2% |

`real_elo = 1629.6 + 1.2797*raw_elo`. Anchors (excluding Zeus) properly
monotonic (1875 > 1709 > 1543 > 1293).

**Zeus v1.3 ≈ 1729 Elo, up +200 over v1.2b's 1529.** The largest single
jump so far, and for the first time Zeus out-rates one of the Stockfish
anchors it's anchored against (1729 vs Stockfish-1700's 1709) rather
than just an easier anchor than itself. Qualitatively confirmed too: at
the same `go movetime 800` test position used for earlier stages, node
counts dropped roughly 3x at matched depths and the engine reached depth
9 in less time than it previously took to reach depth 8 — consistent
with TT cutoffs actually firing, not just the hash-move ordering tier
helping a little.

**One honest caveat:** one game (round 65, 70-ish range, Stockfish-1700
vs Zeus, English Opening, 195 plies) ended in a **time loss for Zeus** —
0.4% of the 240 games, the first `Timeout` result observed since v0.1.
It was a long, sharp king-and-queen endgame with repeated promotions
late in the game (a classic "many forced-looking moves burn clock fast
under 0 increment" scenario). Not investigated further this stage (one
occurrence isn't enough signal to diagnose, and the soft/hard bound
formulas in `crates/zeus-uci/src/time.rs` are otherwise unchanged from
v1 where zero timeouts were observed over 240+1000 games) — noted here
so it isn't quietly forgotten; worth re-checking if the rate increases
in a future stage's measurement.

Takeaways for the next roadmap stage (`ROADMAP.md`):
- Piece-Square Tables are next (first evaluation-side change since
  v0.1's material-only baseline) — expect a real, substantial Elo gain
  given search-side improvements have now mostly run their course for
  the "basic" techniques and the evaluation function still can't tell a
  knight on a rim square from one in the center.
- Keep an eye on the timeout rate in future measurements given the one
  observed here.

## v1.4 — piece-square tables

Commit: `77d7802` ("Add piece-square tables to evaluation"). Engine
change from v1.3: evaluation is no longer material-only — adds all 7 PST
tables (`Reference/evaluation/Simplified_Evaluation_Function.md`'s exact
values) per piece, with a binary middlegame/endgame king-table switch
(`Reference/evaluation/Piece-Square_Tables.md`). Methodology unchanged
(240 games, 24/pair, opening suite, 1-minute bullet, same roster) —
clean single-variable comparison against v1.3's 1729 Elo. Full game log:
`results/zeus_v1.4_pst_tournament.pgn` + `.moves.log` (committed, 240
games).

| Participant | Raw (bayeselo) | Calibrated real Elo | +/- | Games | Score% | Draws% |
|---|---|---|---|---|---|---|
| **Zeus-v1.4-PST** | 260 | **2109** | 71/62 | 96 | 83% | 4% |
| Stockfish-1900 | 92 | **1849** | 58/56 | 96 | 63% | 4% |
| Stockfish-1700 | -10 | **1691** | 55/55 | 96 | 48% | 7% |
| Stockfish-1500 | -67 | **1602** | 55/56 | 96 | 41% | 8% |
| Stockfish-1320 | -276 | **1278** | 64/74 | 96 | 15% | 3% |

`real_elo = 1706.1 + 1.5497*raw_elo`. Anchors (excluding Zeus) properly
monotonic (1849 > 1691 > 1602 > 1278).

**Zeus v1.4 ≈ 2109 Elo, up +380 over v1.3's 1729 — the largest jump of
any stage so far**, and Zeus now decisively out-rates every single
Stockfish anchor in the roster (83% score vs the floor, 70.8% even
against Stockfish-1900) rather than just one. Large, but not
implausible: `Reference/evaluation/Piece-Square_Tables.md` quotes the
source material itself calling material + PST "enough for a program to
play a semi-decent game of chess" — a material-only evaluator has
*zero* defense against systematic positional blunders (undeveloped
pieces, a king wandering to a bad square, no sense of center control)
regardless of how deep or efficient its search is, so fixing that
across the board is a bigger lever than any single search technique
measured so far. Spot-verified by hand before this run (see the v1.4
commit message): `go depth 1` from the startpos scores `Nf3` at exactly
+50cp, matching the table lookup by hand.

**Second honest timeout note:** another time loss (round 10, Zeus as
Black vs Stockfish-1320, Sicilian Kan, 153 plies) — 1/240 again (0.4%),
same profile as v1.3's (a long, maneuvering middlegame/endgame grind).
Two timeouts now across 480 total games at this stage's methodology
(v1.3 + v1.4), both in long games with heavy non-forcing shuffling.
Still not investigated in depth, but the pattern (long games, not short
ones) is consistent enough across both occurrences that it's worth
flagging explicitly as a hypothesis: Zeus's hard time bound
(`crates/zeus-uci/src/time.rs`: `min(soft*4, time_left/2)`) may not
account well for *cumulative* drift over very long games with many
near-hard-bound moves in a row, as opposed to any single move
overrunning badly. Worth a dedicated look if a future stage's
measurement shows a third occurrence or a climbing rate.

Takeaways for the next roadmap stage (`ROADMAP.md`):
- Aspiration windows are next (search-side again). Given PST's outsized
  gain, it may be worth sanity-checking that evaluation-side low-hanging
  fruit isn't being systematically underweighted relative to search-side
  features in this roadmap's ordering — but staying with the existing
  planned order for now rather than re-prioritizing off one data point.
- The timeout pattern above is worth a dedicated look if it recurs.

## v1.5 — aspiration windows

Commit: `5e85235` ("Add aspiration windows"). Engine change from v1.4:
`search_root_once` (depth >= 2) searches a narrow window around the
previous iteration's score (`Reference/search/Aspiration_Windows.md`)
instead of the full range, re-searching with the failing side widened on
a miss. Methodology unchanged (240 games, 24/pair, opening suite,
1-minute bullet, same roster) — nominally a clean comparison against
v1.4's 2109, but read the roster-saturation caveat below before taking
the headline number at face value. Full game log:
`results/zeus_v1.5_aspw_tournament.pgn` + `.moves.log` (committed, 240
games).

| Participant | Raw (bayeselo) | Calibrated real Elo | +/- | Games | Score% | Draws% |
|---|---|---|---|---|---|---|
| **Zeus-v1.5-AspW** | 319 | **2030** | 76/65 | 96 | 89% | 8% |
| Stockfish-1900 | 88 | **1784** | 60/59 | 96 | 59% | 6% |
| Stockfish-1700 | 54 | **1748** | 59/59 | 96 | 56% | 5% |
| Stockfish-1500 | -76 | **1609** | 59/61 | 96 | 40% | 4% |
| Stockfish-1320 | -386 | **1279** | 77/99 | 96 | 7% | 3% |

`real_elo = 1690.3 + 1.0661*raw_elo`.

**Headline number: -79 vs v1.4 (2030 vs 2109). Not read as a
regression** — here's why. v1.5's head-to-head scores against the same
four anchors were *equal or higher* than v1.4's in 3 of 4 cases:

| Opponent | v1.4 score% | v1.5 score% |
|---|---|---|
| Stockfish-1320 | 91.7% | **97.9%** |
| Stockfish-1500 | 85.4% | **93.8%** |
| Stockfish-1700 | 85.4% | **91.7%** |
| Stockfish-1900 | 70.8% | 70.8% (tied) |

Zeus is now winning 92-98% of games against the three weaker anchors,
which leaves very little variance in those pairings for bayeselo's
joint fit to anchor the scale with — the fitted slope this run (1.0661)
is much shallower than v1.4's (1.5497), compressing the whole scale and
pulling Zeus's calibrated number down despite stronger raw performance.
This is a **roster-saturation problem**, not evidence aspiration
windows hurt: our 1320-1900 anchor spread is no longer wide enough to
precisely calibrate an engine this strong, and every future stage will
face the same growing problem.

**Action taken**: not re-measuring v1.4/v1.5 retroactively (the
head-to-head comparison above is already informative enough, and
redoing both under a wider roster would cost another several hours for
a result unlikely to overturn "no clear change, plausibly slightly
positive"). **Action for the future**: the roster should gain a
higher anchor (e.g. 2100+) starting with the next stage's measurement,
so later stages don't run into this same calibration ceiling as Zeus
keeps improving.

Another long-game timeout observed (two games both hit exactly 175
plies this run; moves.log doesn't record which specific one timed out,
only the live progress log did) — third occurrence of the same pattern
flagged since v1.3. Still not investigated in depth, but three data
points now makes this worth a dedicated look soon rather than continuing
to defer it.

Takeaways for the next roadmap stage (`ROADMAP.md`):
- **Widen the Stockfish roster** before the next calibration run — see
  the entry immediately below, which does exactly this.
- Null Move Pruning is next (search-side).
- The timeout pattern (3 occurrences now, all long maneuvering games)
  deserves a real look soon, not just another passive note.

## v1.5 (2000+ roster) — resolving the saturation ambiguity

Per the user's explicit request: the default Stockfish roster is now
`2000,2300,2600` (3 anchors above Zeus's current strength range),
**replacing** the original `1320,1500,1700,1900` rather than extending
it (`crates/zeus-bench/src/main.rs`). Same engine as the v1.5 entry
above (aspiration windows, no code change) — this is not a new
engine-feature measurement, it's a re-measurement of the same binary
under a roster that doesn't saturate. 6 pairs x 24 games = 144 games,
same TC/opening-suite/concurrency as every other entry. Full game log:
`results/zeus_v1.5_highroster_tournament.pgn` + `.moves.log` (committed,
144 games).

| Participant | Raw (bayeselo) | Calibrated real Elo | +/- | Games | Score% | Draws% |
|---|---|---|---|---|---|---|
| Stockfish-2600 | 190 | **2612** | 65/58 | 72 | 82% | 17% |
| **Zeus-v1.5-AspW-HighRoster** | 16 | **2334** | 57/57 | 72 | 51% | 11% |
| Stockfish-2300 | -23 | **2272** | 57/58 | 72 | 46% | 8% |
| Stockfish-2000 | -183 | **2016** | 60/68 | 72 | 21% | 8% |

`real_elo = 2308.5 + 1.5978*raw_elo`. Anchors properly monotonic
(2612 > 2272 > 2016) and close to their nominal UCI_Elo targets — no
saturation this time (score% against every anchor is in a healthy
21-82% range, nothing near a shutout), exactly what the roster change
was meant to fix.

**Zeus v1.5 ≈ 2334 Elo** — a well-calibrated absolute number, not
directly comparable as a "delta" to the earlier low-roster entries
(the roster changed entirely, so it's not a matched-methodology
comparison). What it resolves: the earlier ~2030/2109 figures were
compressed by saturation against anchors Zeus had already outgrown;
2334 is a materially higher, better-supported estimate of where Zeus
actually stands right now. Scores are sensible and monotonic (51%
vs Stockfish-2300, 73% vs Stockfish-2000-equivalent-strength, 23% vs
Stockfish-2600) — a believable profile for an engine with TT +
PST + move ordering + aspiration windows but still only material/PST
evaluation and no null-move pruning, LMR, or king-safety terms yet.

Going forward: this 3-anchor (2000/2300/2600) roster is the new
default for every future stage's measurement until Zeus outgrows it
too, at which point repeat this same fix (shift the anchors up again)
rather than accumulate anchors indefinitely.

## v1.6 — null move pruning

Commit: `85f1e29` ("Add null move pruning"). Engine change from v1.5:
`alpha_beta` tries a reduced-depth null-window search after passing the
move before generating any real moves
(`Reference/search/Null_Move_Pruning.md`), R=3, guarded against check,
depth < 4, zugzwang-prone (pawn-only) positions, and chained null
moves. Methodology identical to v1.5's 2000+-roster entry (144 games, 6
pairs, same TC/opening-suite) — clean comparison, no saturation this
time either (anchors land properly monotonic again). Full game log:
`results/zeus_v1.6_nmp_tournament.pgn` + `.moves.log` (committed, 144
games).

| Participant | Raw (bayeselo) | Calibrated real Elo | +/- | Games | Score% | Draws% |
|---|---|---|---|---|---|---|
| Stockfish-2600 | 170 | **2609** | 66/59 | 72 | 77% | 7% |
| **Zeus-v1.6-NMP** | -4 | **2291** | 57/57 | 72 | 50% | 6% |
| Stockfish-2300 | -10 | **2280** | 56/57 | 72 | 48% | 7% |
| Stockfish-2000 | -157 | **2011** | 58/64 | 72 | 25% | 8% |

`real_elo = 2298.2 + 1.8287*raw_elo`. Anchors properly monotonic
(2609 > 2280 > 2011).

**Zeus v1.6 ≈ 2291 Elo, vs v1.5's 2334: -43 Elo — not a statistically
meaningful change at this sample size.** The 95% CIs on the individual
pairwise deltas (e.g. +/-150-300 Elo wide) comfortably contain zero
movement; -43 reads as noise, the same conclusion v1.1's +9 reached, not
a sign the feature hurts. The qualitative evidence for the feature
working correctly stands independent of this measurement: node counts
at matched depths dropped further on the same fixed test position used
for every prior stage (~2.7x at depth 8 vs pre-NMP), and the engine
reached one ply deeper in the same time budget. A pawn-only-material
test position correctly triggered the zugzwang guard and returned a
sane move rather than a pruning-induced blunder.

Plausible (unproven) reasons a clean positive signal didn't show up
here, parallel to the quiescence/move-ordering interaction found
earlier: null move pruning's main value is usually realized together
with **Principal Variation Search** (a later roadmap stage) and deeper
search generally helps less without evaluation refinements (king
safety, mobility) to make the extra depth count for something beyond
material/PST — both still missing. Not re-measuring now; flagged for
the same "revisit once the prerequisite lands" treatment as v1.1.

**Fourth timeout observed** (183 plies this run) — same long-game
profile as the three prior occurrences (v1.3, v1.4, v1.5). Four data
points across four different engine versions now, all in long
maneuvering/endgame-heavy games, none in short tactical ones. This
pattern has been deferred three times already; it should be looked at
directly before deferring a fourth time.

Takeaways for the next roadmap stage (`ROADMAP.md`):
- Principal Variation Search is next — also the natural point to
  revisit whether null move pruning's contribution becomes clearer
  once PVS is in place, mirroring the quiescence/move-ordering pattern.
- The timeout pattern (now 4 occurrences, 100% in long games) should get
  a real look: read a sample of the affected games'
  `crates/uci/src/time.rs` clock accounting, or add instrumentation to
  `zeus-bench`/`zeus-uci` logging actual time used per move vs budgeted,
  rather than continuing to note it passively.

## v1.7 — Principal Variation Search

Commit: `1934be9` ("Add Principal Variation Search"). Engine change from
v1.6: `alpha_beta` and `search_root_pass` both restructured to
`Reference/search/Principal_Variation_Search.md`'s fail-soft pseudocode
— first move full window, later moves null-window with a conditional
full re-search. Methodology identical to v1.6 (144 games, 6 pairs, same
2000/2300/2600 roster, no saturation). Full game log:
`results/zeus_v1.7_pvs_tournament.pgn` + `.moves.log` (committed, 144
games).

| Participant | Raw (bayeselo) | Calibrated real Elo | +/- | Games | Score% | Draws% |
|---|---|---|---|---|---|---|
| Stockfish-2600 | 197 | **2585** | 66/59 | 72 | 81% | 14% |
| **Zeus-v1.7-PVS** | 28 | **2352** | 57/56 | 72 | 53% | 15% |
| Stockfish-2300 | 10 | **2327** | 58/58 | 72 | 51% | 13% |
| Stockfish-2000 | -235 | **1988** | 65/76 | 72 | 15% | 6% |

`real_elo = 2312.9 + 1.3806*raw_elo`. Anchors properly monotonic
(2585 > 2327 > 1988).

**Zeus v1.7 ≈ 2352 Elo, up +61 over v1.6's 2291.** Modest but positive —
consistent with PVS's own stated expectation ("with a good move
ordering we expect to save about 10% of a search effort", not a huge
jump on its own). Confirmed qualitatively too and more rigorously than
usual: on the same fixed test position used for every prior stage,
scores at every depth 1-9 matched the pre-PVS run *exactly* (not just
plausibly similar), proving PVS computes identical results — this
stage's efficiency gain (reached depth 10 vs 9 in the same time) is
pure speedup, not a side-effect of a subtly different search.

**Fifth timeout observed this run** (201 plies) — same long-game
profile as all four prior occurrences. Five occurrences across five
consecutive engine versions, every single one in a long game. Actually
investigated this time instead of deferring again.

### Timeout investigation: found and fixed (commit `2c92550`)

Root cause: a panic inside the search thread was **silently discarded**
by `join_search`'s `let _ = handle.join();` — the engine would simply
never send `bestmove`, which any arbiter waiting on the clock cannot
distinguish from a slow move; it just runs the clock out. That's exactly
what every one of these five "Timeout" results actually was.

Found by replaying this run's actual 201-ply losing game back through
the engine move-by-move (`position ... moves ...` + `go wtime/btime`
with the real shrinking clock), which reproduced a panic:
`crates/zeus-engine/src/search.rs`'s `capture_score` hit `move source
always has a piece` — `board.piece_on(mv.get_source())` returned `None`
for a move `MoveGen` had just generated as legal, meaning the `Board`
itself had become internally inconsistent. Traced to
`crates/zeus-uci/src/main.rs::handle_position`: it applied moves from
`position ... moves ...` via `make_move_new` without checking
`board.legal(mv)` first (unlike `zeus-bench`'s own `game.rs`, which
always validates). An illegal/malformed move string reaching that path
would silently corrupt the board, and a later search node built from it
could violate invariants the search code assumes always hold.

Two fixes, both in `crates/zeus-uci/src/main.rs`:
1. `handle_position` now checks `board.legal(mv)` before applying each
   move from `position ... moves ...`, closing the specific corruption
   path found.
2. **Defense in depth, independent of root cause**: the search thread
   now wraps its work in `catch_unwind` and falls back to the first
   legal move on *any* panic, so the engine can never again go silent
   and strand the GUI/arbiter waiting out the clock — this should
   eliminate the failure mode even if some other, not-yet-found panic
   trigger exists.

Not re-run as a dedicated 144-game calibration on its own — the
historical rate (5 of ~700+ games across all stages, <1%) isn't large
enough to justify one in isolation, and a regular future-stage
measurement will show whether the pattern is actually gone (zero
Timeouts expected going forward; any further occurrence would now at
least degrade to a cheap fallback move rather than a lost game).

Takeaways for the next roadmap stage (`ROADMAP.md`):
- Late Move Reductions is next.
- Confirm in the next tournament's logs whether Timeout results have
  actually stopped (expected) or just degraded to cheap fallback moves
  (the defense-in-depth path firing, implying some other panic trigger
  still exists and is worth finding).
