//! Fixed opening suite for tournament games.
//!
//! Reference: `Reference/testing/Engine_Testing.md` — "use opening
//! suites, a set of quiet test positions ... the same opening suite would
//! be used for each tournament conducted and furthermore each position is
//! played a second time with colors reversed." These are standard,
//! well-known named openings (chess theory, not engine algorithm
//! decisions), given as UCI long-algebraic move lists from the start
//! position, applied before the timed part of a game begins
//! (`src/game.rs`).
pub struct Opening {
    pub name: &'static str,
    pub moves: &'static [&'static str],
}

pub const SUITE: &[Opening] = &[
    Opening { name: "Ruy Lopez", moves: &["e2e4", "e7e5", "g1f3", "b8c6", "f1b5"] },
    Opening { name: "Italian Game", moves: &["e2e4", "e7e5", "g1f3", "b8c6", "f1c4"] },
    Opening { name: "Open Sicilian", moves: &["e2e4", "c7c5", "g1f3", "d7d6", "d2d4", "c5d4", "f3d4", "g8f6", "b1c3"] },
    Opening { name: "Sicilian Kan", moves: &["e2e4", "c7c5", "g1f3", "e7e6"] },
    Opening { name: "French Defense", moves: &["e2e4", "e7e6", "d2d4", "d7d5"] },
    Opening { name: "Caro-Kann Defense", moves: &["e2e4", "c7c6", "d2d4", "d7d5"] },
    Opening { name: "Queen's Gambit Declined", moves: &["d2d4", "d7d5", "c2c4", "e7e6"] },
    Opening { name: "Slav Defense", moves: &["d2d4", "d7d5", "c2c4", "c7c6"] },
    Opening { name: "King's Indian Defense", moves: &["d2d4", "g8f6", "c2c4", "g7g6", "b1c3", "f8g7"] },
    Opening { name: "Nimzo-Indian Defense", moves: &["d2d4", "g8f6", "c2c4", "e7e6", "b1c3", "f8b4"] },
    Opening { name: "English Opening", moves: &["c2c4", "e7e5"] },
    Opening { name: "Scandinavian Defense", moves: &["e2e4", "d7d5"] },
];
