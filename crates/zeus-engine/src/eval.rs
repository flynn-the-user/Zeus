//! Material + piece-square table evaluation.
//!
//! Reference: `Reference/evaluation/Simplified_Evaluation_Function.md`
//! (piece values, Michniewski 1995 set, and every PST table value below)
//! and `Reference/evaluation/Point_Value.md` (confirms that value set as
//! a named, documented historical table rather than an invented
//! constant). `Reference/evaluation/Piece-Square_Tables.md` covers the
//! general technique and the deliberate scope choice (a binary
//! middlegame/endgame table switch, not a smooth "Tapered Eval").
use chess::{Board, Color, Piece, ALL_PIECES, ALL_COLORS};

/// Centipawn value per piece type, indexed by `Piece::to_index()`.
/// Order is [Pawn, Knight, Bishop, Rook, Queen, King] per the `chess` crate's
/// `ALL_PIECES` ordering.
pub const PIECE_VALUE: [i32; 6] = [100, 320, 330, 500, 900, 20_000];

fn piece_value(piece: Piece) -> i32 {
    PIECE_VALUE[piece.to_index()]
}

// Every table below is given exactly as printed in
// Reference/evaluation/Simplified_Evaluation_Function.md: index 0 = a8,
// row-major, ranks printed top (rank 8, index 0..8) to bottom (rank 1,
// index 56..64) - i.e. White's own perspective. `pst_value` below handles
// converting a real (White- or Black-side) square into this table's
// indexing, including Black's vertical mirror.
#[rustfmt::skip]
const PAWN_PST: [i32; 64] = [
     0,  0,  0,  0,  0,  0,  0,  0,
    50, 50, 50, 50, 50, 50, 50, 50,
    10, 10, 20, 30, 30, 20, 10, 10,
     5,  5, 10, 25, 25, 10,  5,  5,
     0,  0,  0, 20, 20,  0,  0,  0,
     5, -5,-10,  0,  0,-10, -5,  5,
     5, 10, 10,-20,-20, 10, 10,  5,
     0,  0,  0,  0,  0,  0,  0,  0,
];

#[rustfmt::skip]
const KNIGHT_PST: [i32; 64] = [
    -50,-40,-30,-30,-30,-30,-40,-50,
    -40,-20,  0,  0,  0,  0,-20,-40,
    -30,  0, 10, 15, 15, 10,  0,-30,
    -30,  5, 15, 20, 20, 15,  5,-30,
    -30,  0, 15, 20, 20, 15,  0,-30,
    -30,  5, 10, 15, 15, 10,  5,-30,
    -40,-20,  0,  5,  5,  0,-20,-40,
    -50,-40,-30,-30,-30,-30,-40,-50,
];

#[rustfmt::skip]
const BISHOP_PST: [i32; 64] = [
    -20,-10,-10,-10,-10,-10,-10,-20,
    -10,  0,  0,  0,  0,  0,  0,-10,
    -10,  0,  5, 10, 10,  5,  0,-10,
    -10,  5,  5, 10, 10,  5,  5,-10,
    -10,  0, 10, 10, 10, 10,  0,-10,
    -10, 10, 10, 10, 10, 10, 10,-10,
    -10,  5,  0,  0,  0,  0,  5,-10,
    -20,-10,-10,-10,-10,-10,-10,-20,
];

#[rustfmt::skip]
const ROOK_PST: [i32; 64] = [
     0,  0,  0,  0,  0,  0,  0,  0,
     5, 10, 10, 10, 10, 10, 10,  5,
    -5,  0,  0,  0,  0,  0,  0, -5,
    -5,  0,  0,  0,  0,  0,  0, -5,
    -5,  0,  0,  0,  0,  0,  0, -5,
    -5,  0,  0,  0,  0,  0,  0, -5,
    -5,  0,  0,  0,  0,  0,  0, -5,
     0,  0,  0,  5,  5,  0,  0,  0,
];

#[rustfmt::skip]
const QUEEN_PST: [i32; 64] = [
    -20,-10,-10, -5, -5,-10,-10,-20,
    -10,  0,  0,  0,  0,  0,  0,-10,
    -10,  0,  5,  5,  5,  5,  0,-10,
     -5,  0,  5,  5,  5,  5,  0, -5,
      0,  0,  5,  5,  5,  5,  0, -5,
    -10,  5,  5,  5,  5,  5,  0,-10,
    -10,  0,  5,  0,  0,  0,  0,-10,
    -20,-10,-10, -5, -5,-10,-10,-20,
];

#[rustfmt::skip]
const KING_MG_PST: [i32; 64] = [
    -30,-40,-40,-50,-50,-40,-40,-30,
    -30,-40,-40,-50,-50,-40,-40,-30,
    -30,-40,-40,-50,-50,-40,-40,-30,
    -30,-40,-40,-50,-50,-40,-40,-30,
    -20,-30,-30,-40,-40,-30,-30,-20,
    -10,-20,-20,-20,-20,-20,-20,-10,
     20, 20,  0,  0,  0,  0, 20, 20,
     20, 30, 10,  0,  0, 10, 30, 20,
];

#[rustfmt::skip]
const KING_EG_PST: [i32; 64] = [
    -50,-40,-30,-20,-20,-30,-40,-50,
    -30,-20,-10,  0,  0,-10,-20,-30,
    -30,-10, 20, 30, 30, 20,-10,-30,
    -30,-10, 30, 40, 40, 30,-10,-30,
    -30,-10, 30, 40, 40, 30,-10,-30,
    -30,-10, 20, 30, 30, 20,-10,-30,
    -30,-30,  0,  0,  0,  0,-30,-30,
    -50,-30,-30,-30,-30,-30,-30,-50,
];

/// "both sides have no queens, or every side that still has a queen has
/// no other piece or at most one minor piece" — the exact endgame-begins
/// condition from Reference/evaluation/Simplified_Evaluation_Function.md.
fn is_endgame(board: &Board) -> bool {
    if board.pieces(Piece::Queen).popcnt() == 0 {
        return true;
    }
    for &color in ALL_COLORS.iter() {
        let own = board.color_combined(color);
        let has_queen = (*board.pieces(Piece::Queen) & *own).popcnt() > 0;
        if has_queen {
            let rooks = (*board.pieces(Piece::Rook) & *own).popcnt();
            let minors = ((*board.pieces(Piece::Bishop) | *board.pieces(Piece::Knight)) & *own).popcnt();
            if rooks > 0 || minors > 1 {
                return false;
            }
        }
    }
    true
}

fn pst_value(piece: Piece, color: Color, square: chess::Square, endgame: bool) -> i32 {
    let table = match piece {
        Piece::Pawn => &PAWN_PST,
        Piece::Knight => &KNIGHT_PST,
        Piece::Bishop => &BISHOP_PST,
        Piece::Rook => &ROOK_PST,
        Piece::Queen => &QUEEN_PST,
        Piece::King => {
            if endgame {
                &KING_EG_PST
            } else {
                &KING_MG_PST
            }
        }
    };
    let file = square.get_file().to_index();
    let rank = square.get_rank().to_index();
    // White: table row 0 is rank 8, so White's rank r (0=rank1) maps to
    // printed row (7-r). Black: mirrored, maps directly to row r - e.g. a
    // White pawn on rank 7 and a Black pawn on rank 2 (both "one step
    // from promoting") hit the same table row.
    let index = match color {
        Color::White => (7 - rank) * 8 + file,
        Color::Black => rank * 8 + file,
    };
    table[index]
}

/// Static evaluation of `board`, in centipawns, **relative to the side to
/// move** (positive means the side to move is better).
///
/// This sign convention is mandated by `Reference/search/Negamax.md`:
/// "your Static Evaluation function must return a score relative to the
/// side being evaluated."
pub fn evaluate(board: &Board) -> i32 {
    let endgame = is_endgame(board);
    let mut white_score = 0;
    let mut black_score = 0;
    for &piece in ALL_PIECES.iter() {
        for &color in ALL_COLORS.iter() {
            let bb = *board.pieces(piece) & *board.color_combined(color);
            for square in bb {
                let value = piece_value(piece) + pst_value(piece, color, square, endgame);
                match color {
                    Color::White => white_score += value,
                    Color::Black => black_score += value,
                }
            }
        }
    }
    let white_relative = white_score - black_score;
    match board.side_to_move() {
        Color::White => white_relative,
        Color::Black => -white_relative,
    }
}
