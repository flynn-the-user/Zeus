//! Elo estimation from match results.
//!
//! Reference: `Reference/testing/Match_Statistics.md`.
//!   E = 1 / (1 + 10^(-delta/400))         (expected score from Elo delta)
//!   delta = 400 * log10(E / (1 - E))      (Elo delta from observed score)
//!   N = wins + draws + losses, score = wins + draws/2, E = score/N

#[derive(Debug, Clone, Copy)]
pub struct MatchResult {
    pub wins: u32,
    pub draws: u32,
    pub losses: u32,
}

impl MatchResult {
    pub fn n(&self) -> u32 {
        self.wins + self.draws + self.losses
    }

    pub fn score(&self) -> f64 {
        self.wins as f64 + self.draws as f64 / 2.0
    }

    pub fn expected_score(&self) -> f64 {
        self.score() / self.n() as f64
    }
}

#[derive(Debug, Clone, Copy)]
pub enum EloEstimate {
    /// Normal case: 0 < E < 1, delta is finite.
    Delta {
        delta: f64,
        delta_lo: f64,
        delta_hi: f64,
    },
    /// All games lost (E = 0): true strength is below the opponent's Elo
    /// by an unknown (unbounded by this data) amount. Reporting a finite
    /// number here would misrepresent what a 0-win sample can prove, so we
    /// surface this as its own variant rather than inventing a continuity
    /// correction that Reference/testing/Match_Statistics.md does not
    /// specify.
    AllLosses,
    /// All games won (E = 1): symmetric case.
    AllWins,
}

/// 400*log10(E/(1-E)), per Reference/testing/Match_Statistics.md.
fn delta_from_expected(e: f64) -> f64 {
    400.0 * (e / (1.0 - e)).log10()
}

/// 95% Wald confidence interval on E (standard normal approximation),
/// E +/- 1.96*sqrt(E(1-E)/N), then each bound is passed back through the
/// same delta formula. This is the explicit uncertainty band called for by
/// Reference/testing/Match_Statistics.md's emphasis on confidence
/// intervals/error bars alongside the point estimate.
pub fn estimate(result: MatchResult) -> EloEstimate {
    let n = result.n() as f64;
    let e = result.expected_score();

    if result.wins == 0 && result.draws == 0 {
        return EloEstimate::AllLosses;
    }
    if result.losses == 0 && result.draws == 0 {
        return EloEstimate::AllWins;
    }

    let se = (e * (1.0 - e) / n).sqrt();
    let e_lo = (e - 1.96 * se).clamp(1e-6, 1.0 - 1e-6);
    let e_hi = (e + 1.96 * se).clamp(1e-6, 1.0 - 1e-6);
    let e_clamped = e.clamp(1e-6, 1.0 - 1e-6);

    EloEstimate::Delta {
        delta: delta_from_expected(e_clamped),
        delta_lo: delta_from_expected(e_lo),
        delta_hi: delta_from_expected(e_hi),
    }
}
