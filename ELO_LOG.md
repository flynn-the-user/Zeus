# Elo Log

Methodology (fixed across every entry so results are comparable — see
`Reference/testing/Match_Statistics.md` and `crates/zeus-bench`):

- Opponent: local Stockfish 17.1, strength pinned via `UCI_LimitStrength`
  + `UCI_Elo` (`Reference/protocols/UCI_Protocol.md`), `Threads=1`.
- Time control: 10s base + 0.1s increment per side, enforced by the harness
  itself (real wall-clock deadline per move), not trusted to either engine.
- Games alternate colors (half Zeus-as-White, half Zeus-as-Black).
- Elo estimate: `delta = 400*log10(E/(1-E))`, `zeus_elo = opponent_elo +
  delta`, with a 95% Wald CI on the observed score propagated through the
  same formula (`crates/zeus-bench/src/elo.rs`).
- Command: `zeus-bench --games-per-rung 40 --concurrency 5 --base-ms 10000
  --inc-ms 100 --rungs <elo>`, run on a 10-core Apple Silicon Mac.

## v0.1 — Negamax + Alpha-Beta (fixed depth 4) + material-only eval

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
