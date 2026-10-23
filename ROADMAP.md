# Zeus Roadmap

Rule for every stage: fetch the cited CPW page(s) into `Reference/` *before*
writing the feature, implement it, measure its Elo with `zeus-bench` against
the Stockfish calibration ladder (see `Reference/testing/Match_Statistics.md`
and `docs/CALIBRATION.md`), record the result in `ELO_LOG.md`, then commit
and push. A stage is not "done" until its Elo delta is measured and logged.

## Done

- [x] v0.1 — Negamax + Alpha-Beta (fixed depth) + material-only eval
      (`Reference/search/Negamax.md`, `Reference/search/Alpha-Beta.md`,
      `Reference/evaluation/Simplified_Evaluation_Function.md`).
      Measured: **~1141 Elo** (95% CI 984..1249) vs Stockfish's UCI_Elo
      floor of 1320 — see `ELO_LOG.md`.

## Planned, in order

1. **Iterative Deepening** + basic **Time Management** (soft/hard move time
   budget from `wtime`/`btime`/`winc`/`binc`/`movestogo`) — needs
   `Reference/search/Iterative_Deepening.md`, `Reference/search/Time_Management.md`.
   This is the feature the user flagged as critical: without it the engine
   risks losing games on time at fast calibration time controls.
2. **Quiescence Search** (captures-only search at leaf nodes to avoid the
   horizon effect) — needs `Reference/search/Quiescence_Search.md`.
3. **Move Ordering**: MVV-LVA for captures, killer-move heuristic for quiets
   — needs `Reference/search/Move_Ordering.md`, `Reference/search/MVV-LVA.md`,
   `Reference/search/Killer_Heuristic.md`.
4. **Transposition Table** with **Zobrist Hashing** — needs
   `Reference/search/Transposition_Table.md`, `Reference/search/Zobrist_Hashing.md`.
5. **Piece-Square Tables** added to evaluation (isolate their Elo contribution
   from the search-side gains above) — needs
   `Reference/evaluation/Piece-Square_Tables.md`.
6. **Aspiration Windows** around the previous iteration's score.
7. **Null Move Pruning**.
8. **Principal Variation Search** (replace plain alpha-beta).
9. **Late Move Reductions**.
10. **King Safety / Mobility / Pawn Structure** evaluation terms.
11. (stretch) **NNUE** evaluation — large scope increase (training data,
    network format); only attempted after 1–10 are measured and logged, and
    only with its own dedicated Reference fetch + citation set.

Each numbered stage gets its own commit(s) and its own `ELO_LOG.md` entry
showing before/after Elo with confidence interval, per
`Reference/testing/Match_Statistics.md`.
