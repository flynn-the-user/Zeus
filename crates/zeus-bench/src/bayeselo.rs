//! Drives the real `bayeselo` CLI (Rémi Coulom's tool; see
//! `Reference/testing/BayesElo.md`) over a PGN file and parses its
//! `ratings` table. The exact command sequence
//! (`readpgn` -> `elo` -> `mm` -> `exactdist` -> `ratings` -> `x`) is the
//! one documented/demonstrated for this tool, not invented here.
use std::io::Write;
use std::process::{Command, Stdio};

#[derive(Debug, Clone)]
pub struct Rating {
    pub name: String,
    pub elo: i64,
    pub plus: i64,
    pub minus: i64,
    pub games: u32,
    pub score_pct: f64,
    pub opponent_avg_elo: i64,
    pub draws_pct: f64,
}

pub fn run(bayeselo_path: &str, pgn_path: &str) -> std::io::Result<Vec<Rating>> {
    let mut child = Command::new(bayeselo_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()?;

    {
        let stdin = child.stdin.as_mut().expect("piped stdin");
        writeln!(stdin, "readpgn {pgn_path}")?;
        writeln!(stdin, "elo")?;
        writeln!(stdin, "mm")?;
        writeln!(stdin, "exactdist")?;
        writeln!(stdin, "ratings")?;
        writeln!(stdin, "x")?;
    }

    let output = child.wait_with_output()?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(parse_ratings(&stdout))
}

/// bayeselo's ratings are on an arbitrary scale (no fixed zero point) -
/// Reference/testing/BayesElo.md's own example output shows a weak amateur
/// engine rated "2", not anything near a real-world Elo. We have several
/// participants whose real-world strength is already known (the Stockfish
/// anchors, pinned via UCI_Elo), so instead of anchoring to just one of
/// them (noisy, and Reference/testing/BayesElo.md's own data shows the
/// anchors aren't perfectly linearly spaced at this time control), fit a
/// least-squares line `real_elo = a + b*raw_elo` over all of them and use
/// it to map every participant's raw rating onto the real Elo scale. This
/// is a standard least-squares fit, not a bayeselo feature - it's applied
/// on top of bayeselo's output by `crates/zeus-bench/src/main.rs`.
pub fn fit_real_elo_scale(ratings: &[Rating], known_anchors: &[(String, i64)]) -> Option<(f64, f64)> {
    let points: Vec<(f64, f64)> = known_anchors
        .iter()
        .filter_map(|(name, real_elo)| {
            ratings
                .iter()
                .find(|r| &r.name == name)
                .map(|r| (r.elo as f64, *real_elo as f64))
        })
        .collect();
    if points.len() < 2 {
        return None;
    }
    let n = points.len() as f64;
    let mean_x = points.iter().map(|(x, _)| x).sum::<f64>() / n;
    let mean_y = points.iter().map(|(_, y)| y).sum::<f64>() / n;
    let cov: f64 = points.iter().map(|(x, y)| (x - mean_x) * (y - mean_y)).sum();
    let var_x: f64 = points.iter().map(|(x, _)| (x - mean_x).powi(2)).sum();
    if var_x == 0.0 {
        return None;
    }
    let b = cov / var_x;
    let a = mean_y - b * mean_x;
    Some((a, b))
}

fn parse_ratings(output: &str) -> Vec<Rating> {
    let mut ratings = Vec::new();
    let mut in_table = false;
    for line in output.lines() {
        let trimmed = line.trim();
        // The REPL prompt (e.g. "ResultSet-EloRating>") is written with no
        // trailing newline, so it ends up glued directly onto the start of
        // this header line rather than on its own line - match on
        // "contains", not "starts_with".
        if trimmed.contains("Rank") && trimmed.contains("Name") && trimmed.contains("Elo") {
            in_table = true;
            continue;
        }
        if !in_table {
            continue;
        }
        let tokens: Vec<&str> = trimmed.split_whitespace().collect();
        if tokens.len() != 9 || tokens[0].parse::<u32>().is_err() {
            break; // end of table (blank line / next prompt)
        }
        let parse_pct = |s: &str| s.trim_end_matches('%').parse::<f64>().unwrap_or(f64::NAN);
        ratings.push(Rating {
            name: tokens[1].to_string(),
            elo: tokens[2].parse().unwrap_or(0),
            plus: tokens[3].parse().unwrap_or(0),
            minus: tokens[4].parse().unwrap_or(0),
            games: tokens[5].parse().unwrap_or(0),
            score_pct: parse_pct(tokens[6]),
            opponent_avg_elo: tokens[7].parse().unwrap_or(0),
            draws_pct: parse_pct(tokens[8]),
        });
    }
    ratings
}
