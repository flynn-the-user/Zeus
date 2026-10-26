//! Converts a UCI `go` command's clock info into the soft/hard deadlines
//! `zeus_engine::search::search_iterative` is abortable against.
//! Reference: `Reference/search/Time_Management.md`.
use std::time::{Duration, Instant};

#[derive(Default, Clone, Copy)]
pub struct GoClock {
    pub wtime: Option<i64>,
    pub btime: Option<i64>,
    pub winc: Option<i64>,
    pub binc: Option<i64>,
    pub movetime: Option<i64>,
    pub infinite: bool,
}

/// Returns (soft_deadline, hard_deadline). `soft_deadline` is `None` when
/// there is no clock-derived budget at all (e.g. `infinite`, or `go` with
/// no time fields) - in that case a depth cap is what actually bounds the
/// search, not time.
pub fn compute_deadlines(clock: &GoClock, white_to_move: bool, now: Instant) -> (Option<Instant>, Instant) {
    // No reference gives a concrete "unbounded" constant; this is just
    // long enough to never fire before a depth cap or an explicit `stop`
    // would, for `infinite` or clockless `go`.
    let effectively_unbounded = now + Duration::from_secs(3600);

    if clock.infinite {
        return (None, effectively_unbounded);
    }
    if let Some(mt) = clock.movetime {
        // UCI: "search exactly x mseconds" - a single explicit deadline,
        // soft and hard are the same per Reference/search/Time_Management.md.
        let d = Duration::from_millis(mt.max(0) as u64);
        return (Some(now + d), now + d);
    }

    let (time_left, inc) = if white_to_move {
        (clock.wtime, clock.winc.unwrap_or(0))
    } else {
        (clock.btime, clock.binc.unwrap_or(0))
    };
    let Some(time_left) = time_left else {
        return (None, effectively_unbounded);
    };

    // Reference/search/Time_Management.md's basic formula, applied
    // directly: "base / 20 + increment / 2".
    let soft_ms = (time_left as f64 / 20.0 + inc as f64 / 2.0).max(1.0);
    // Hard bound: Zeus's own choice (the reference explicitly gives no
    // formula for this, only that it exists and is checked more often) -
    // allow up to 4x the soft target, but never more than half the actual
    // remaining time, so a single move can't come close to flagging.
    let hard_ms = (soft_ms * 4.0).min(time_left as f64 / 2.0).max(soft_ms);

    (
        Some(now + Duration::from_millis(soft_ms as u64)),
        now + Duration::from_millis(hard_ms as u64),
    )
}
