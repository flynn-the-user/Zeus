---
source: https://www.chessprogramming.org/Zobrist_Hashing
retrieved: 2026-10-30
---

# Zobrist Hashing

> a technique to transform a board position of arbitrary size into a
> number of a set length, with an equal distribution over all possible
> numbers

Assign one random number to each independent feature of a position, XOR
together the numbers for every feature present. Random numbers needed:

- **768** = 12 piece-types (6 piece kinds x 2 colors) x 64 squares — one
  number per (piece, color, square).
- **1** for side to move (applied when it's Black's turn).
- **4** for castling rights (white kingside, white queenside, black
  kingside, black queenside — "though 16 ... are often used for speed
  optimization"; Zeus uses 4, the exact feature count, not the
  speed-optimized 16).
- **8** for en passant, one per file.

**Total: 781 random numbers.**

## Incremental update (example given on the page)

For a White Knight b1-c3 capturing a Black Bishop:

> hash of position XOR hash[White Knight, b1] (remove) XOR hash[Black
> Bishop, c3] (remove captured) XOR hash[White Knight, c3] (place) XOR
> hash[side to move] (flip)

> the xor-operation is own inverse and can be undone by using the same
> xor-operation again

No pseudocode, and no explicit worked example for castling, en passant,
or promotion is given on this page.

## Zeus usage — deliberate simplification

Zeus does **not** implement incremental XOR updates this stage.
`crates/zeus-engine/src/zobrist.rs::hash(board)` recomputes the full hash
from scratch (iterate all 64 squares, plus castling rights, plus en
passant file, plus side to move) every time it's called. This is still
"Zobrist hashing" exactly as defined above (XOR of random numbers per
present feature) — incremental updating is this page's own suggested
*optimization* on top of the technique, not part of its definition, and
the page gives no worked pseudocode for the trickier incremental cases
(castling rights changing, en passant square appearing/disappearing,
promotion) to implement against faithfully. Recomputing from scratch is
O(64) array lookups, cheap relative to move generation, and a legitimate
"start simple" choice consistent with the rest of this project — revisit
if profiling later shows hashing is a bottleneck.

The 781 random numbers themselves are generated once at startup by a
fixed-seed splitmix64 generator (`crates/zeus-engine/src/zobrist.rs`) —
the specific PRNG algorithm and seed are Zeus's own engineering choice,
not specified by this page (which only requires "equal distribution",
not a particular generator).
