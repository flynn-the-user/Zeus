//! Material-only evaluation.
//!
//! Reference: `Reference/evaluation/Simplified_Evaluation_Function.md`
//! (piece values, Michniewski 1995 set) and
//! `Reference/evaluation/Point_Value.md` (confirms that set as a named,
//! documented historical value table rather than an invented constant).
//! Piece-square tables from the same page are deliberately NOT used yet
//! (see `ROADMAP.md`: PST is a separate, later-measured stage).
use chess::{Board, Color, Piece, ALL_PIECES};

/// Centipawn value per piece type, indexed by `Piece::to_index()`.
/// Order is [Pawn, Knight, Bishop, Rook, Queen, King] per the `chess` crate's
/// `ALL_PIECES` ordering.
pub const PIECE_VALUE: [i32; 6] = [100, 320, 330, 500, 900, 20_000];

fn piece_value(piece: Piece) -> i32 {
    PIECE_VALUE[piece.to_index()]
}

/// Static evaluation of `board`, in centipawns, **relative to the side to
/// move** (positive means the side to move is better).
///
/// This sign convention is mandated by `Reference/search/Negamax.md`:
/// "your Static Evaluation function must return a score relative to the
/// side being evaluated."
pub fn evaluate(board: &Board) -> i32 {
    let mut white_material = 0;
    let mut black_material = 0;
    for &piece in ALL_PIECES.iter() {
        let count_for = |color: Color| (board.pieces(piece) & board.color_combined(color)).popcnt() as i32;
        white_material += piece_value(piece) * count_for(Color::White);
        black_material += piece_value(piece) * count_for(Color::Black);
    }
    let white_relative = white_material - black_material;
    match board.side_to_move() {
        Color::White => white_relative,
        Color::Black => -white_relative,
    }
}
