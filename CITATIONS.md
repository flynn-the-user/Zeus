# Citations Ledger

Every non-trivial algorithmic decision in Zeus must have a row here pointing
at the exact file in `Reference/` it came from. If it isn't in this table,
it isn't allowed in the engine. "Non-trivial" excludes plain Rust/software
engineering (error handling, CLI parsing, process spawning) — it means
anything that affects *how the engine thinks* (search, evaluation, time
management, move ordering) or *how strength is measured*.

| Decision | File(s) | Reference |
|---|---|---|
| Recursive search, side-to-move-relative score sign convention | `crates/zeus-engine/src/search.rs` | `Reference/search/Negamax.md` |
| Alpha-beta pruning, fail-soft negamax form | `crates/zeus-engine/src/search.rs` | `Reference/search/Alpha-Beta.md` |
| Iterative deepening (depth 1,2,3,... until time/stop), discard an aborted iteration and keep the last fully-completed one | `crates/zeus-engine/src/search.rs::search_iterative` | `Reference/search/Iterative_Deepening.md` |
| Depth-1 / first-root-move fallback so an abort can never leave zero candidate moves (Zeus's own addition, noted as such in the reference file) | `crates/zeus-engine/src/search.rs::search_root_once` | `Reference/search/Iterative_Deepening.md` (explicitly marked as not from this page) |
| Soft bound = `time_left/20 + increment/2`; soft checked between ID iterations, hard checked every 2048 nodes during search | `crates/zeus-uci/src/time.rs`, `crates/zeus-engine/src/search.rs::SearchContext` | `Reference/search/Time_Management.md` |
| Hard-bound formula (`min(soft*4, time_left/2)`) and the 2048-node check interval | `crates/zeus-uci/src/time.rs`, `crates/zeus-engine/src/search.rs` | Not from the reference — `Reference/search/Time_Management.md` explicitly gives no formula for the hard bound, only that it exists; documented as Zeus's own engineering choice in both files. |
| Async `stop`: search runs on a background thread, `stop` sets a shared `AtomicBool` checked periodically by the search | `crates/zeus-uci/src/main.rs` | `Reference/protocols/UCI_Protocol.md` (`stop`, and `isready` must answer immediately even while searching) |
| Quiescence search at the main search's leaf (captures-only, stand-pat lower bound, alpha-beta cutoffs) | `crates/zeus-engine/src/search.rs::quiesce` | `Reference/search/Quiescence_Search.md` |
| In check: stand-pat disallowed, search all legal evasions instead of captures-only | `crates/zeus-engine/src/search.rs::quiesce` | `Reference/search/Quiescence_Search.md` (explicit base-algorithm requirement on that page, not an enhancement) |
| No MVV-LVA/move ordering in quiescence yet, unordered captures bounded only by the existing time-deadline node check | `crates/zeus-engine/src/search.rs::quiesce` | `Reference/search/Quiescence_Search.md` flags this as a risk and recommends MVV-LVA first; documented here as a deliberate scope decision (move ordering is the next roadmap stage) rather than an oversight. |
| En passant captures not generated in quiescence's captures-only mask (`set_iterator_mask` against opponent-occupied squares misses the empty destination square of an e.p. capture) | `crates/zeus-engine/src/search.rs::quiesce` | Not from the reference — a known simplification of the `chess` crate's masking API, noted as such in both the reference file and here. |
| Material-only evaluation, piece values 100/320/330/500/900 | `crates/zeus-engine/src/eval.rs` | `Reference/evaluation/Simplified_Evaluation_Function.md`, `Reference/evaluation/Point_Value.md` (Michniewski 1995 set) |
| UCI command set implemented (`uci`, `isready`, `position`, `go`, `stop`, `ucinewgame`, `quit`, `id`, `uciok`, `readyok`, `info`, `bestmove`) | `crates/zeus-uci/src/main.rs` | `Reference/protocols/UCI_Protocol.md` |
| Pairwise Elo estimate (quick cross-check only): `Δ = 400·log10(E/(1-E))`, `engine_elo = opponent_elo + Δ` | `crates/zeus-bench/src/elo.rs` | `Reference/testing/Match_Statistics.md` |
| Calibration opponent strength set via `UCI_LimitStrength`+`UCI_Elo` | `crates/zeus-bench/src/uci_client.rs` | `Reference/protocols/UCI_Protocol.md` (option table) |
| Authoritative tournament rating: round-robin (Zeus + all Stockfish anchors play each other), joint fit via the real `bayeselo` CLI (Bradley-Terry + Bayesian prior, MM algorithm), command sequence `readpgn`→`elo`→`mm`→`exactdist`→`ratings` | `crates/zeus-bench/src/bayeselo.rs`, `crates/zeus-bench/src/pgn.rs` | `Reference/testing/BayesElo.md` (which `Reference/testing/Match_Statistics.md` itself names and links) |
| Mapping bayeselo's arbitrary-origin rating scale onto real Elo via ordinary least-squares regression against the known Stockfish `UCI_Elo` anchors | `crates/zeus-bench/src/bayeselo.rs::fit_real_elo_scale` | Not a chess-engine-algorithm decision — standard statistics (OLS line fit), applied on top of bayeselo's output since `Reference/testing/BayesElo.md`'s own example shows the tool's raw numbers aren't on a real-Elo scale by default. Noted here for completeness, not presented as CPW-sourced. |
| Fixed opening suite (not always startpos), each opening played twice per pair with colors reversed before moving to the next | `crates/zeus-bench/src/openings.rs`, `crates/zeus-bench/src/main.rs` game-scheduling loop | `Reference/testing/Engine_Testing.md` ("Opening" subsection). The 12 named openings themselves are standard chess theory, not an engine-algorithm decision — same footing as the draw rules in `src/game.rs`, noted as such in the reference file. |
