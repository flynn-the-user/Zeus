# Citations Ledger

Every non-trivial algorithmic decision in Zeus must have a row here pointing
at the exact file in `Reference/` it came from. If it isn't in this table,
it isn't allowed in the engine. "Non-trivial" excludes plain Rust/software
engineering (error handling, CLI parsing, process spawning) — it means
anything that affects *how the engine thinks* (search, evaluation, time
management, move ordering) or *how strength is measured*.

| Decision | File(s) | Reference |
|---|---|---|
| Fixed-depth recursive search, side-to-move-relative score sign convention | `crates/zeus-engine/src/search.rs` | `Reference/search/Negamax.md` |
| Alpha-beta pruning, fail-soft negamax form | `crates/zeus-engine/src/search.rs` | `Reference/search/Alpha-Beta.md` |
| Material-only evaluation, piece values 100/320/330/500/900 | `crates/zeus-engine/src/eval.rs` | `Reference/evaluation/Simplified_Evaluation_Function.md`, `Reference/evaluation/Point_Value.md` (Michniewski 1995 set) |
| UCI command set implemented (`uci`, `isready`, `position`, `go`, `stop`, `ucinewgame`, `quit`, `id`, `uciok`, `readyok`, `info`, `bestmove`) | `crates/zeus-uci/src/main.rs` | `Reference/protocols/UCI_Protocol.md` |
| Pairwise Elo estimate (quick cross-check only): `Δ = 400·log10(E/(1-E))`, `engine_elo = opponent_elo + Δ` | `crates/zeus-bench/src/elo.rs` | `Reference/testing/Match_Statistics.md` |
| Calibration opponent strength set via `UCI_LimitStrength`+`UCI_Elo` | `crates/zeus-bench/src/uci_client.rs` | `Reference/protocols/UCI_Protocol.md` (option table) |
| Authoritative tournament rating: round-robin (Zeus + all Stockfish anchors play each other), joint fit via the real `bayeselo` CLI (Bradley-Terry + Bayesian prior, MM algorithm), command sequence `readpgn`→`elo`→`mm`→`exactdist`→`ratings` | `crates/zeus-bench/src/bayeselo.rs`, `crates/zeus-bench/src/pgn.rs` | `Reference/testing/BayesElo.md` (which `Reference/testing/Match_Statistics.md` itself names and links) |
| Mapping bayeselo's arbitrary-origin rating scale onto real Elo via ordinary least-squares regression against the known Stockfish `UCI_Elo` anchors | `crates/zeus-bench/src/bayeselo.rs::fit_real_elo_scale` | Not a chess-engine-algorithm decision — standard statistics (OLS line fit), applied on top of bayeselo's output since `Reference/testing/BayesElo.md`'s own example shows the tool's raw numbers aren't on a real-Elo scale by default. Noted here for completeness, not presented as CPW-sourced. |
