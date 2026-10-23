//! zeus-bench: Elo calibration harness.
//!
//! Plays Zeus against a local Stockfish restricted to specific target Elo
//! values (via the standard `UCI_LimitStrength`/`UCI_Elo` option pair,
//! Reference/protocols/UCI_Protocol.md) at a fixed, fast time control, with
//! many games run in parallel, and reports an Elo estimate with a
//! confidence interval per Reference/testing/Match_Statistics.md (see
//! src/elo.rs). This is the methodology CITATIONS.md and ROADMAP.md
//! require every future feature's Elo delta to be measured with, so the
//! time control and opponent set must stay fixed across runs to keep
//! results comparable.
mod elo;
mod game;
mod uci_client;

use elo::{estimate, EloEstimate, MatchResult};
use game::{play_game, Outcome, TimeControl};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use uci_client::UciEngine;

struct Args {
    engine_cmd: String,
    opponent_cmd: String,
    games_per_rung: u32,
    concurrency: usize,
    base_ms: i64,
    inc_ms: i64,
    max_plies: u32,
    rungs: Option<Vec<i64>>,
}

fn parse_args() -> Args {
    let mut engine_cmd = default_engine_path();
    let mut opponent_cmd = "stockfish".to_string();
    let mut games_per_rung = 40u32;
    let mut concurrency = (std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(2)
        / 2)
    .max(1);
    let mut base_ms = 10_000i64;
    let mut inc_ms = 100i64;
    let mut max_plies = 240u32;
    let mut rungs: Option<Vec<i64>> = None;

    let argv: Vec<String> = std::env::args().skip(1).collect();
    let mut i = 0;
    while i < argv.len() {
        match argv[i].as_str() {
            "--engine" => {
                engine_cmd = argv[i + 1].clone();
                i += 2;
            }
            "--opponent" => {
                opponent_cmd = argv[i + 1].clone();
                i += 2;
            }
            "--games-per-rung" => {
                games_per_rung = argv[i + 1].parse().expect("--games-per-rung is a number");
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
            "--rungs" => {
                rungs = Some(
                    argv[i + 1]
                        .split(',')
                        .map(|s| s.parse().expect("--rungs is a comma list of integers"))
                        .collect(),
                );
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
        opponent_cmd,
        games_per_rung,
        concurrency,
        base_ms,
        inc_ms,
        max_plies,
        rungs,
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

#[derive(Clone, Copy)]
struct GameSpec {
    rung_elo: i64,
    zeus_is_white: bool,
}

#[derive(Clone, Copy)]
enum ZeusResult {
    Win,
    Draw,
    Loss,
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

    // Query Stockfish's own declared UCI_Elo range rather than hardcoding
    // an assumed min/max (Reference/protocols/UCI_Protocol.md: engines
    // declare "option name UCI_Elo type spin default <x> min <x> max <x>"
    // during the uci handshake).
    let probe = UciEngine::spawn(&args.opponent_cmd, &[])
        .unwrap_or_else(|e| panic!("failed to spawn opponent '{}': {e}", args.opponent_cmd));
    let uci_elo_opt = probe
        .options
        .get("UCI_Elo")
        .cloned()
        .unwrap_or_else(|| panic!("opponent '{}' has no UCI_Elo option", args.opponent_cmd));
    drop(probe);
    let elo_min = uci_elo_opt.min.expect("UCI_Elo option declares a min");
    let elo_max = uci_elo_opt.max.expect("UCI_Elo option declares a max");
    println!(
        "Opponent '{}' declares UCI_Elo range [{elo_min}, {elo_max}]",
        args.opponent_cmd
    );

    let rungs = args.rungs.clone().unwrap_or_else(|| {
        // Zeus v0.1 (fixed-depth negamax/alpha-beta, material-only eval,
        // no quiescence search) is expected to be very weak, so the
        // default ladder stays near the floor of the opponent's range
        // instead of wasting games on rungs it has ~0% chance against.
        let span = elo_max - elo_min;
        vec![
            elo_min,
            elo_min + (span as f64 * 0.15) as i64,
            elo_min + (span as f64 * 0.30) as i64,
        ]
    });

    println!(
        "zeus-bench: engine='{}' opponent='{}' games_per_rung={} concurrency={} tc={}+{}ms max_plies={}",
        args.engine_cmd, args.opponent_cmd, args.games_per_rung, args.concurrency,
        args.base_ms, args.inc_ms, args.max_plies
    );

    for &rung in &rungs {
        let result = run_rung(&args, rung);
        print_rung_result(rung, result);
    }
}

fn run_rung(args: &Args, rung_elo: i64) -> MatchResult {
    let mut queue: VecDeque<GameSpec> = VecDeque::new();
    for g in 0..args.games_per_rung {
        queue.push_back(GameSpec {
            rung_elo,
            zeus_is_white: g % 2 == 0,
        });
    }
    let queue = Arc::new(Mutex::new(queue));
    let results: Arc<Mutex<Vec<ZeusResult>>> = Arc::new(Mutex::new(Vec::new()));
    let done = Arc::new(Mutex::new(0u32));
    let total = args.games_per_rung;

    std::thread::scope(|scope| {
        for _ in 0..args.concurrency {
            let queue = Arc::clone(&queue);
            let results = Arc::clone(&results);
            let done = Arc::clone(&done);
            let engine_cmd = args.engine_cmd.clone();
            let opponent_cmd = args.opponent_cmd.clone();
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

                let Ok(mut zeus) = UciEngine::spawn(&engine_cmd, &[]) else {
                    continue;
                };
                let Ok(mut opponent) = UciEngine::spawn(&opponent_cmd, &[]) else {
                    continue;
                };
                let _ = opponent.set_option("UCI_LimitStrength", "true");
                let _ = opponent.set_option("UCI_Elo", &spec.rung_elo.to_string());
                let _ = opponent.set_option("Threads", "1");
                let _ = zeus.sync();
                let _ = opponent.sync();

                let record = if spec.zeus_is_white {
                    play_game(&mut zeus, &mut opponent, tc, max_plies)
                } else {
                    play_game(&mut opponent, &mut zeus, tc, max_plies)
                };
                let outcome = record.outcome;
                zeus.quit();
                opponent.quit();

                let zeus_result = match outcome {
                    Outcome::Draw => ZeusResult::Draw,
                    Outcome::WhiteWins if spec.zeus_is_white => ZeusResult::Win,
                    Outcome::BlackWins if !spec.zeus_is_white => ZeusResult::Win,
                    _ => ZeusResult::Loss,
                };
                results.lock().unwrap().push(zeus_result);

                let mut d = done.lock().unwrap();
                *d += 1;
                println!(
                    "  [elo {}] game {}/{} done ({:?}, {} plies)",
                    spec.rung_elo, *d, total, record.reason, record.plies
                );
            });
        }
    });

    let results = results.lock().unwrap();
    let wins = results.iter().filter(|o| matches!(o, ZeusResult::Win)).count() as u32;
    let losses = results.iter().filter(|o| matches!(o, ZeusResult::Loss)).count() as u32;
    let draws = results.iter().filter(|o| matches!(o, ZeusResult::Draw)).count() as u32;
    MatchResult {
        wins,
        draws,
        losses,
    }
}

fn print_rung_result(rung_elo: i64, result: MatchResult) {
    println!(
        "\n=== Stockfish UCI_Elo {rung_elo} === N={} W={} D={} L={} score={:.1}%",
        result.n(),
        result.wins,
        result.draws,
        result.losses,
        result.expected_score() * 100.0
    );
    match estimate(result) {
        EloEstimate::Delta {
            delta,
            delta_lo,
            delta_hi,
        } => {
            println!(
                "    estimated Zeus Elo: {:.0}  (95% CI: {:.0} .. {:.0})  [opponent {rung_elo} + delta]",
                rung_elo as f64 + delta,
                rung_elo as f64 + delta_lo,
                rung_elo as f64 + delta_hi
            );
        }
        EloEstimate::AllLosses => {
            println!(
                "    0 wins/draws out of {}: Zeus Elo is below {rung_elo} by an unknown margin (no finite estimate from this sample)",
                result.n()
            );
        }
        EloEstimate::AllWins => {
            println!(
                "    0 losses/draws out of {}: Zeus Elo is above {rung_elo} by an unknown margin (no finite estimate from this sample)",
                result.n()
            );
        }
    }
}
