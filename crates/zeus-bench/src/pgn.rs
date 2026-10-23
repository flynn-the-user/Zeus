//! Minimal PGN game records for bayeselo's `readpgn`.
//!
//! Reference: `Reference/testing/BayesElo.md` — bayeselo's rating
//! computation only uses the `[White]`/`[Black]`/`[Result]` tags, so each
//! game is written with just those headers plus the result token as a
//! one-line movetext body (verified against the real `bayeselo` binary
//! during development: `readpgn` accepted this minimal form without
//! complaint). Participant names must not contain whitespace, since the
//! `ratings` table bayeselo prints is whitespace-delimited and
//! `crates/zeus-bench/src/bayeselo.rs` parses it that way.
use crate::game::Outcome;

pub fn result_token(outcome: Outcome) -> &'static str {
    match outcome {
        Outcome::WhiteWins => "1-0",
        Outcome::BlackWins => "0-1",
        Outcome::Draw => "1/2-1/2",
    }
}

pub fn format_game(event: &str, round: usize, white: &str, black: &str, outcome: Outcome) -> String {
    let result = result_token(outcome);
    format!(
        "[Event \"{event}\"]\n[Round \"{round}\"]\n[White \"{white}\"]\n[Black \"{black}\"]\n[Result \"{result}\"]\n\n{result}\n"
    )
}
