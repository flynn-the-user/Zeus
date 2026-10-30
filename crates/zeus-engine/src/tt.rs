//! Transposition table, per `Reference/search/Transposition_Table.md`.
use chess::ChessMove;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Bound {
    /// PV-node: the true value.
    Exact,
    /// Cut-node: a beta cutoff occurred, the true value is at least this.
    Lower,
    /// All-node: no move improved alpha, the true value is at most this.
    Upper,
}

#[derive(Clone, Copy)]
pub struct TtEntry {
    pub key: u64,
    pub depth: u32,
    pub score: i32,
    pub bound: Bound,
    pub best_move: Option<ChessMove>,
}

/// Single fixed-size, always-replace table (both choices documented as
/// Zeus's own in `Reference/search/Transposition_Table.md` — that page
/// names several replacement schemes without recommending one). Indexed
/// by the low bits of the Zobrist key; the full key is also stored per
/// entry to detect collisions between positions sharing those low bits.
pub struct TranspositionTable {
    slots: Vec<Option<TtEntry>>,
    mask: usize,
}

impl TranspositionTable {
    /// `size_mb` per `Reference/protocols/UCI_Protocol.md`'s standard
    /// `Hash` option. Slot count is the largest power of two (for a cheap
    /// `& mask` index) that fits in `size_mb`, with a sane floor so a
    /// tiny/zero setting still works.
    pub fn new(size_mb: usize) -> Self {
        let entry_bytes = std::mem::size_of::<Option<TtEntry>>().max(1);
        let budget_bytes = size_mb.max(1) * 1024 * 1024;
        let slots = (budget_bytes / entry_bytes).max(1).next_power_of_two();
        let slots = slots.max(1 << 16); // floor: ~65k slots even for a tiny Hash setting
        TranspositionTable {
            slots: vec![None; slots],
            mask: slots - 1,
        }
    }

    pub fn clear(&mut self) {
        for s in self.slots.iter_mut() {
            *s = None;
        }
    }

    pub fn probe(&self, key: u64) -> Option<TtEntry> {
        match &self.slots[key as usize & self.mask] {
            Some(e) if e.key == key => Some(*e),
            _ => None,
        }
    }

    pub fn store(&mut self, entry: TtEntry) {
        self.slots[entry.key as usize & self.mask] = Some(entry);
    }
}
