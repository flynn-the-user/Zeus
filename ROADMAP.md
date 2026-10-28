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
      **~1081 Elo**. Rebaselined with the opening-suite fix, matched
      exactly to v1's methodology below: **~1050 Elo** — this is the
      number v1's delta is computed against.
- [x] v1 **Iterative Deepening** + **Time Management**
      (`Reference/search/Iterative_Deepening.md`,
      `Reference/search/Time_Management.md`) + async UCI `stop`. Measured
      (round-robin, 240 games, 1-minute bullet, 12-opening suite,
      bayeselo): **~1325 Elo**, up **+275** from the matched-methodology
      v0.1 rebaseline (single-variable comparison) — see `ELO_LOG.md`.
- [x] v1.1 **Quiescence Search** (`Reference/search/Quiescence_Search.md`)
      at the main search's leaf, with check-evasion handling per that
      page's explicit requirement. Measured (round-robin, 240 games,
      1-minute bullet, same methodology as v1, no rebaseline needed):
      **~1334 Elo**, +9 from v1 — **not a statistically meaningful
      change at this sample size** (see `ELO_LOG.md`'s noise-floor
      discussion: a known anchor pair inverted their nominal Elo order in
      this same run). Feature correctness confirmed qualitatively
      instead (horizon-effect score oscillation present before, gone
      after); its Elo contribution at the current (still
      move-ordering-free, material-only) configuration is not resolved
      by this measurement and should be revisited once move ordering
      lands.
- [x] v1.2 **Move Ordering**: MVV-LVA for captures
      (`Reference/search/MVV-LVA.md`) + killer-move heuristic for quiets
      (`Reference/search/Killer_Heuristic.md`), per
      `Reference/search/Move_Ordering.md`'s collapsed tier order for an
      engine without a hash/PV move or SEE yet. Measured as two
      tournaments per the user's request, to isolate it from quiescence:
      **Stage A** (move ordering alone, quiescence off, vs v1's 1325):
      **~1435 Elo, +110**. **Stage B** (quiescence back on, vs Stage A):
      **~1529 Elo, +94** — quiescence's contribution is now clearly
      measurable, confirming v1.1's hypothesis that it was being masked
      by unordered captures. Net from both features together: **+204**
      over v1. Full detail (including the temporary `UseQuiescence`
      ablation toggle) in `ELO_LOG.md`.

## Planned, in order

1. **Transposition Table** with **Zobrist Hashing** — needs
   `Reference/search/Transposition_Table.md`, `Reference/search/Zobrist_Hashing.md`.
   Also expected to unlock a real hash-move for move ordering's top tier
   (not yet implemented - see `ELO_LOG.md`'s v1.2 takeaways).
2. **Piece-Square Tables** added to evaluation (isolate their Elo contribution
   from the search-side gains above) — needs
   `Reference/evaluation/Piece-Square_Tables.md`.
3. **Aspiration Windows** around the previous iteration's score.
4. **Null Move Pruning**.
5. **Principal Variation Search** (replace plain alpha-beta).
6. **Late Move Reductions**.
7. **King Safety / Mobility / Pawn Structure** evaluation terms.
8. (stretch) **NNUE** evaluation — large scope increase (training data,
   network format); only attempted after 1–7 are measured and logged, and
   only with its own dedicated Reference fetch + citation set.

Each numbered stage gets its own commit(s) and its own `ELO_LOG.md` entry
showing before/after Elo with confidence interval, per
`Reference/testing/Match_Statistics.md`.
