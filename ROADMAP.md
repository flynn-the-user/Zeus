# Zeus Roadmap

Rule for every stage: fetch the cited CPW page(s) into `Reference/` *before*
writing the feature, implement it, measure its Elo with `zeus-bench`'s
round-robin tournament against the Stockfish Elo roster (methodology fixed
in `ELO_LOG.md`'s header: opening suite, 1-minute bullet, rated with the
real `bayeselo` CLI — see `Reference/testing/BayesElo.md`; games/pair
scales down from 100 once the engine is slow enough that 100 isn't
practical, noted per entry), record the result in `ELO_LOG.md`, then
commit and push. A stage is not "done" until its Elo delta is measured and
logged.

## Done

- [x] v0.1 — Negamax + Alpha-Beta (fixed depth) + material-only eval
      (`Reference/search/Negamax.md`, `Reference/search/Alpha-Beta.md`,
      `Reference/evaluation/Simplified_Evaluation_Function.md`).
      Measured (superseded single-pair methodology): **~1141 Elo** (95% CI
      984..1249) vs Stockfish's UCI_Elo floor of 1320. Re-measured with the
      tournament methodology (round-robin, 1000 games, 1+0 bullet,
      bayeselo, no opening suite yet — see its caveat in `ELO_LOG.md`):
      **~1081 Elo**.
- [x] v1 **Iterative Deepening** + **Time Management**
      (`Reference/search/Iterative_Deepening.md`,
      `Reference/search/Time_Management.md`) + async UCI `stop`. Measured
      (round-robin, 240 games, 1-minute bullet, 12-opening suite,
      bayeselo): **~1325 Elo**, up ~244 from v0.1 — see `ELO_LOG.md`.

## Planned, in order

1. **Quiescence Search** (captures-only search at leaf nodes to avoid the
   horizon effect) — needs `Reference/search/Quiescence_Search.md`.
2. **Move Ordering**: MVV-LVA for captures, killer-move heuristic for quiets
   — needs `Reference/search/Move_Ordering.md`, `Reference/search/MVV-LVA.md`,
   `Reference/search/Killer_Heuristic.md`.
3. **Transposition Table** with **Zobrist Hashing** — needs
   `Reference/search/Transposition_Table.md`, `Reference/search/Zobrist_Hashing.md`.
4. **Piece-Square Tables** added to evaluation (isolate their Elo contribution
   from the search-side gains above) — needs
   `Reference/evaluation/Piece-Square_Tables.md`.
5. **Aspiration Windows** around the previous iteration's score.
6. **Null Move Pruning**.
7. **Principal Variation Search** (replace plain alpha-beta).
8. **Late Move Reductions**.
9. **King Safety / Mobility / Pawn Structure** evaluation terms.
10. (stretch) **NNUE** evaluation — large scope increase (training data,
    network format); only attempted after 1–9 are measured and logged, and
    only with its own dedicated Reference fetch + citation set.

Each numbered stage gets its own commit(s) and its own `ELO_LOG.md` entry
showing before/after Elo with confidence interval, per
`Reference/testing/Match_Statistics.md`.
