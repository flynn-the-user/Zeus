//! zeus-bench: round-robin tournament Elo calibration harness.
//!
//! Zeus plus several Stockfish instances pinned to different target Elo
//! values (via `UCI_LimitStrength`/`UCI_Elo`,
//! Reference/protocols/UCI_Protocol.md) all play each other — not just
//! Zeus against each anchor — at a fixed fast (bullet) time control, many
//! games in parallel. Every game is recorded as a minimal PGN
//! (`src/pgn.rs`) and the whole batch is handed to the real `bayeselo` CLI
//! (`src/bayeselo.rs`, Reference/testing/BayesElo.md) to fit ratings for
//! every participant jointly via its Bayesian Bradley-Terry model — more
//! robust than the single-pair logistic formula in `src/elo.rs` (which is
//! still printed per-pair as a quick cross-check), especially when Zeus
//! shuts out or gets shut out by an anchor.
//!
//! This is the methodology ELO_LOG.md / ROADMAP.md require every future
//! stage's before/after comparison to reuse unchanged.
mod bayeselo;
mod elo;
mod game;
mod openings;
mod pgn;
mod uci_client;

use elo::{estimate, EloEstimate, MatchResult};
use game::{play_game, Outcome, TimeControl};
use std::collections::{HashMap, VecDeque};
use std::fs::OpenOptions;
use std::io::Write;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use uci_client::UciEngine;

#[derive(Clone)]
struct Participant {
    name: String,
    cmd: String,
    options: Vec<(String, String)>,
}

struct Args {
    engine_cmd: String,
    engine_label: String,
    opponent_cmd: String,
    roster_elo: Vec<i64>,
    games_per_pair: u32,
    concurrency: usize,
    base_ms: i64,
    inc_ms: i64,
    max_plies: u32,
    pgn_out: String,
    bayeselo_path: String,
}

fn parse_args() -> Args {
    let mut engine_cmd = default_engine_path();
    let mut engine_label = "Zeus-v0.1".to_string();
    let mut opponent_cmd = "stockfish".to_string();
    let mut roster_elo = vec![1320i64, 1500, 1700, 1900];
    let mut games_per_pair = 100u32;
    let mut concurrency = (std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(2)
        / 2)
    .max(1);
    let mut base_ms = 60_000i64; // 1-minute bullet
    let mut inc_ms = 0i64;
    let mut max_plies = 240u32;
    let mut pgn_out = "results/tournament.pgn".to_string();
    let mut bayeselo_path = default_bayeselo_path();

    let argv: Vec<String> = std::env::args().skip(1).collect();
    let mut i = 0;
    while i < argv.len() {
        match argv[i].as_str() {
            "--engine" => {
                engine_cmd = argv[i + 1].clone();
                i += 2;
            }
            "--engine-label" => {
                engine_label = argv[i + 1].clone();
                i += 2;
            }
            "--opponent" => {
                opponent_cmd = argv[i + 1].clone();
                i += 2;
            }
            "--roster" => {
                roster_elo = argv[i + 1]
                    .split(',')
                    .map(|s| s.parse().expect("--roster is a comma list of integers"))
                    .collect();
                i += 2;
            }
            "--games-per-pair" => {
                games_per_pair = argv[i + 1].parse().expect("--games-per-pair is a number");
                i += 2;
            }
            "--concurrency" => {
                concurrency = argv[i + 1].parse().expect("--concurrency is a number");
                i += 2;
            }
            "--base-ms" => {
                base_ms = argv[i + 1].parse().expect("--base-ms is a number");
                i += 2;
            }
            "--inc-ms" => {
                inc_ms = argv[i + 1].parse().expect("--inc-ms is a number");
                i += 2;
            }
            "--max-plies" => {
                max_plies = argv[i + 1].parse().expect("--max-plies is a number");
                i += 2;
            }
            "--pgn-out" => {
                pgn_out = argv[i + 1].clone();
                i += 2;
            }
            "--bayeselo-path" => {
                bayeselo_path = argv[i + 1].clone();
                i += 2;
            }
            other => {
                eprintln!("unknown argument: {other}");
                std::process::exit(2);
            }
        }
    }

    Args {
        engine_cmd,
        engine_label,
        opponent_cmd,
        roster_elo,
        games_per_pair,
        concurrency,
        base_ms,
        inc_ms,
        max_plies,
        pgn_out,
        bayeselo_path,
    }
}

fn default_engine_path() -> String {
    for candidate in ["target/release/zeus-uci", "target/debug/zeus-uci"] {
        if std::path::Path::new(candidate).exists() {
            return candidate.to_string();
        }
    }
    "target/debug/zeus-uci".to_string()
}

fn default_bayeselo_path() -> String {
    let candidate = "tools/bayeselo/bayeselo";
    if std::path::Path::new(candidate).exists() {
        candidate.to_string()
    } else {
        "bayeselo".to_string()
    }
}

struct GameSpec {
    pair: (usize, usize),
    white_idx: usize,
    black_idx: usize,
    opening_idx: usize,
}

/// Parses games already written by a previous (possibly interrupted) run of
/// this same `--pgn-out` file, so a relaunch resumes instead of starting
/// over or double-counting. Only understands our own minimal PGN shape
/// (`src/pgn.rs`: consecutive `[White]`/`[Black]`/`[Result]` tag lines).
fn read_existing_games(path: &str) -> Vec<(String, String, String)> {
    let Ok(contents) = std::fs::read_to_string(path) else {
        return Vec::new();
    };
    let mut games = Vec::new();
    let (mut white, mut black) = (None, None);
    for line in contents.lines() {
        if let Some(v) = extract_tag(line, "White") {
            white = Some(v);
        } else if let Some(v) = extract_tag(line, "Black") {
            black = Some(v);
        } else if let Some(v) = extract_tag(line, "Result") {
            if let (Some(w), Some(b)) = (white.take(), black.take()) {
                games.push((w, b, v));
            }
        }
    }
    games
}

fn extract_tag(line: &str, tag: &str) -> Option<String> {
    let prefix = format!("[{tag} \"");
    let rest = line.strip_prefix(&prefix)?;
    rest.strip_suffix("\"]").map(|s| s.to_string())
}

fn outcome_from_result_token(token: &str) -> Option<Outcome> {
    match token {
        "1-0" => Some(Outcome::WhiteWins),
        "0-1" => Some(Outcome::BlackWins),
        "1/2-1/2" => Some(Outcome::Draw),
        _ => None,
    }
}

fn main() {
    let args = parse_args();

    if !std::path::Path::new(&args.engine_cmd).exists() {
        eprintln!(
            "error: engine binary not found at '{}'. Build it first: cargo build --release -p zeus-uci",
            args.engine_cmd
        );
        std::process::exit(1);
    }
    if !std::path::Path::new(&args.bayeselo_path).exists() && args.bayeselo_path.contains('/') {
        eprintln!(
            "error: bayeselo not found at '{}'. Build it first: scripts/setup_bayeselo.sh",
            args.bayeselo_path
        );
        std::process::exit(1);
    }

    let mut participants = vec![Participant {
        name: args.engine_label.clone(),
        cmd: args.engine_cmd.clone(),
        options: vec![],
    }];
    for &elo_target in &args.roster_elo {
        participants.push(Participant {
            name: format!("Stockfish-{elo_target}"),
            cmd: args.opponent_cmd.clone(),
            options: vec![
                ("UCI_LimitStrength".to_string(), "true".to_string()),
                ("UCI_Elo".to_string(), elo_target.to_string()),
                ("Threads".to_string(), "1".to_string()),
            ],
        });
    }
    let n = participants.len();
    let name_to_idx: HashMap<&str, usize> = participants
        .iter()
        .enumerate()
        .map(|(idx, p)| (p.name.as_str(), idx))
        .collect();

    // Resume: fold in games already recorded in a previous (possibly
    // interrupted) run against this same --pgn-out path, so this run only
    // schedules what's still needed per pair, and the cross-check tally /
    // final bayeselo fit include the earlier games too (the PGN file is
    // opened in append mode below, so it keeps growing, not replaced).
    let mut pair_results: HashMap<(usize, usize), (u32, u32, u32)> = HashMap::new();
    let mut already_played: HashMap<(usize, usize), u32> = HashMap::new();
    for (white_name, black_name, result) in read_existing_games(&args.pgn_out) {
        let (Some(&wi), Some(&bi)) = (
            name_to_idx.get(white_name.as_str()),
            name_to_idx.get(black_name.as_str()),
        ) else {
            continue; // game from a different roster/engine-label; ignore
        };
        let Some(outcome) = outcome_from_result_token(&result) else {
            continue;
        };
        let pair = (wi.min(bi), wi.max(bi));
        *already_played.entry(pair).or_insert(0) += 1;
        let entry = pair_results.entry(pair).or_insert((0, 0, 0));
        let lower_is_white = wi == pair.0;
        match (outcome, lower_is_white) {
            (Outcome::Draw, _) => entry.1 += 1,
            (Outcome::WhiteWins, true) | (Outcome::BlackWins, false) => entry.0 += 1,
            _ => entry.2 += 1,
        }
    }
    let resumed_games: u32 = already_played.values().sum();
    if resumed_games > 0 {
        println!("Resuming: {resumed_games} games already recorded in {}", args.pgn_out);
    }

    let mut queue: VecDeque<GameSpec> = VecDeque::new();
    for i in 0..n {
        for j in (i + 1)..n {
            let done_for_pair = *already_played.get(&(i, j)).unwrap_or(&0);
            let remaining = args.games_per_pair.saturating_sub(done_for_pair);
            for g in 0..remaining {
                // Keep alternating from wherever the previous run left off.
                // Per Reference/testing/Engine_Testing.md: each opening is
                // played twice, colors reversed, before moving to the next
                // opening in the suite.
                let seq = done_for_pair + g;
                let (white_idx, black_idx) = if seq % 2 == 0 { (i, j) } else { (j, i) };
                let opening_idx = (seq / 2) as usize % openings::SUITE.len();
                queue.push_back(GameSpec {
                    pair: (i, j),
                    white_idx,
                    black_idx,
                    opening_idx,
                });
            }
        }
    }
    let total_scheduled = queue.len();
    println!(
        "zeus-bench tournament: {n} participants, {} pairs, {} games/pair, {} games scheduled this run, tc={}+{}ms, concurrency={}",
        n * (n - 1) / 2,
        args.games_per_pair,
        total_scheduled,
        args.base_ms,
        args.inc_ms,
        args.concurrency
    );
    for p in &participants {
        println!("  participant: {}", p.name);
    }

    if let Some(parent) = std::path::Path::new(&args.pgn_out).parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let pgn_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&args.pgn_out)
        .expect("open --pgn-out for append");
    // Sidecar log with the actual move list per game (the PGN itself is
    // kept minimal for bayeselo, see src/pgn.rs) - this is what makes it
    // possible to audit that games from the same opening pair actually
    // diverge instead of replaying the same game.
    let moves_log_path = format!("{}.moves.log", args.pgn_out);
    let moves_log = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&moves_log_path)
        .expect("open moves log for append");

    let queue = Arc::new(Mutex::new(queue));
    let participants = Arc::new(participants);
    let pgn_file = Arc::new(Mutex::new(pgn_file));
    let moves_log = Arc::new(Mutex::new(moves_log));
    let pair_results = Arc::new(Mutex::new(pair_results));
    let done = Arc::new(AtomicUsize::new(0));
    let round_counter = Arc::new(AtomicUsize::new(1));

    std::thread::scope(|scope| {
        for _ in 0..args.concurrency {
            let queue = Arc::clone(&queue);
            let participants = Arc::clone(&participants);
            let pgn_file = Arc::clone(&pgn_file);
            let moves_log = Arc::clone(&moves_log);
            let pair_results = Arc::clone(&pair_results);
            let done = Arc::clone(&done);
            let round_counter = Arc::clone(&round_counter);
            let tc = TimeControl {
                base_ms: args.base_ms,
                inc_ms: args.inc_ms,
            };
            let max_plies = args.max_plies;

            scope.spawn(move || loop {
                let spec = {
                    let mut q = queue.lock().unwrap();
                    q.pop_front()
                };
                let Some(spec) = spec else { break };

                let white_p = &participants[spec.white_idx];
                let black_p = &participants[spec.black_idx];

                let Ok(mut white) = UciEngine::spawn(&white_p.cmd, &[]) else {
                    continue;
                };
                let Ok(mut black) = UciEngine::spawn(&black_p.cmd, &[]) else {
                    continue;
                };
                for (k, v) in &white_p.options {
                    let _ = white.set_option(k, v);
                }
                for (k, v) in &black_p.options {
                    let _ = black.set_option(k, v);
                }
                let _ = white.sync();
                let _ = black.sync();

                let opening = &openings::SUITE[spec.opening_idx];
                let opening_moves: Vec<String> = opening.moves.iter().map(|s| s.to_string()).collect();
                let record = play_game(&mut white, &mut black, &opening_moves, tc, max_plies);
                white.quit();
                black.quit();

                let round = round_counter.fetch_add(1, Ordering::Relaxed);
                let pgn_text = pgn::format_game(
                    "Zeus tournament",
                    round,
                    &white_p.name,
                    &black_p.name,
                    opening.name,
                    record.outcome,
                );
                // Written and flushed immediately, one game at a time, so
                // an interrupted/killed run never loses completed games -
                // only the games still in flight at the moment of
                // interruption, and a resumed run (see
                // read_existing_games) picks up from here.
                {
                    let mut f = pgn_file.lock().unwrap();
                    let _ = f.write_all(pgn_text.as_bytes());
                    let _ = f.write_all(b"\n");
                    let _ = f.flush();
                }
                {
                    let mut f = moves_log.lock().unwrap();
                    let _ = writeln!(
                        f,
                        "round={round} white={} black={} opening=\"{}\" result={} plies={} moves={}",
                        white_p.name,
                        black_p.name,
                        opening.name,
                        pgn::result_token(record.outcome),
                        record.plies,
                        record.moves.join(" ")
                    );
                    let _ = f.flush();
                }

                {
                    let mut pr = pair_results.lock().unwrap();
                    let entry = pr.entry(spec.pair).or_insert((0, 0, 0));
                    // Tally from the perspective of the lower-index
                    // participant in the pair (spec.pair.0).
                    let lower_is_white = spec.white_idx == spec.pair.0;
                    match (record.outcome, lower_is_white) {
                        (Outcome::Draw, _) => entry.1 += 1,
                        (Outcome::WhiteWins, true) | (Outcome::BlackWins, false) => entry.0 += 1,
                        _ => entry.2 += 1,
                    }
                }

                let d = done.fetch_add(1, Ordering::Relaxed) + 1;
                if d % 10 == 0 || d == total_scheduled {
                    println!(
                        "  {d}/{total_scheduled} games done this run ({:?}, {} plies)",
                        record.reason, record.plies
                    );
                }
            });
        }
    });

    println!("\n--- per-pair quick cross-check (src/elo.rs pairwise formula) ---");
    let pair_results = pair_results.lock().unwrap();
    for i in 0..n {
        for j in (i + 1)..n {
            if let Some(&(w, d, l)) = pair_results.get(&(i, j)) {
                let result = MatchResult {
                    wins: w,
                    draws: d,
                    losses: l,
                };
                print!(
                    "  {} vs {}: {}W {}D {}L ({:.1}%) ",
                    participants[i].name,
                    participants[j].name,
                    w,
                    d,
                    l,
                    result.expected_score() * 100.0
                );
                match estimate(result) {
                    EloEstimate::Delta {
                        delta,
                        delta_lo,
                        delta_hi,
                    } => println!("delta={delta:+.0} (95% CI {delta_lo:+.0}..{delta_hi:+.0})"),
                    EloEstimate::AllLosses => println!("(shutout loss for {})", participants[i].name),
                    EloEstimate::AllWins => println!("(shutout win for {})", participants[i].name),
                }
            }
        }
    }

    println!(
        "\nAll games written incrementally to {} (ratings input) and {} (full move lists, for audit) as they finished.",
        args.pgn_out, moves_log_path
    );

    println!("\n--- bayeselo ratings (authoritative, joint fit over all participants) ---");
    match bayeselo::run(&args.bayeselo_path, &args.pgn_out) {
        Ok(ratings) => {
            let known_anchors: Vec<(String, i64)> = args
                .roster_elo
                .iter()
                .map(|&e| (format!("Stockfish-{e}"), e))
                .collect();
            let scale = bayeselo::fit_real_elo_scale(&ratings, &known_anchors);
            if let Some((a, b)) = scale {
                println!(
                    "(raw bayeselo ratings are on an arbitrary scale; calibrated to real Elo via least-squares fit against the {} known Stockfish UCI_Elo anchors: real_elo = {a:.1} + {b:.4}*raw_elo)",
                    known_anchors.len()
                );
            } else {
                println!("(could not calibrate to real Elo: need >=2 Stockfish anchors present in the ratings)");
            }
            println!(
                "{:<20} {:>6} {:>10} {:>5} {:>5} {:>6} {:>7} {:>7} {:>7}",
                "Name", "RawElo", "RealElo", "+", "-", "Games", "Score%", "OppElo", "Draws%"
            );
            for r in &ratings {
                let real_elo = scale.map(|(a, b)| format!("{:.0}", a + b * r.elo as f64)).unwrap_or_else(|| "n/a".to_string());
                println!(
                    "{:<20} {:>6} {:>10} {:>5} {:>5} {:>6} {:>6.0}% {:>7} {:>6.0}%",
                    r.name, r.elo, real_elo, r.plus, r.minus, r.games, r.score_pct, r.opponent_avg_elo, r.draws_pct
                );
            }
        }
        Err(e) => eprintln!("bayeselo failed: {e}"),
    }
}
