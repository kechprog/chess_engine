// Piece-square tables for positional evaluation
// All values in centipawns (100 = 1 pawn)
// Tables are from White's perspective (rank 1 at bottom, rank 8 at top)
// For Black pieces, flip the table vertically

// Pawn position values - encourage advancement and central control
pub const PAWN_TABLE: [i32; 64] = [
     0,  0,  0,  0,  0,  0,  0,  0,  // Rank 1 (pawns shouldn't be here)
     5, 10, 10,-20,-20, 10, 10,  5,  // Rank 2
     5, -5,-10,  0,  0,-10, -5,  5,  // Rank 3
     0,  0,  0, 20, 20,  0,  0,  0,  // Rank 4
     5,  5, 10, 25, 25, 10,  5,  5,  // Rank 5
    10, 10, 20, 30, 30, 20, 10, 10,  // Rank 6
    50, 50, 50, 50, 50, 50, 50, 50,  // Rank 7 (near promotion)
     0,  0,  0,  0,  0,  0,  0,  0,  // Rank 8 (pawns shouldn't be here)
];

// Knight position values - prefer center squares
pub const KNIGHT_TABLE: [i32; 64] = [
    -50,-40,-30,-30,-30,-30,-40,-50,  // Rank 1
    -40,-20,  0,  5,  5,  0,-20,-40,  // Rank 2
    -30,  5, 10, 15, 15, 10,  5,-30,  // Rank 3
    -30,  0, 15, 20, 20, 15,  0,-30,  // Rank 4
    -30,  5, 15, 20, 20, 15,  5,-30,  // Rank 5
    -30,  0, 10, 15, 15, 10,  0,-30,  // Rank 6
    -40,-20,  0,  0,  0,  0,-20,-40,  // Rank 7
    -50,-40,-30,-30,-30,-30,-40,-50,  // Rank 8
];

// Bishop position values - prefer center and long diagonals
pub const BISHOP_TABLE: [i32; 64] = [
    -20,-10,-10,-10,-10,-10,-10,-20,  // Rank 1
    -10,  5,  0,  0,  0,  0,  5,-10,  // Rank 2
    -10, 10, 10, 10, 10, 10, 10,-10,  // Rank 3
    -10,  0, 10, 10, 10, 10,  0,-10,  // Rank 4
    -10,  5,  5, 10, 10,  5,  5,-10,  // Rank 5
    -10,  0,  5, 10, 10,  5,  0,-10,  // Rank 6
    -10,  0,  0,  0,  0,  0,  0,-10,  // Rank 7
    -20,-10,-10,-10,-10,-10,-10,-20,  // Rank 8
];

// Rook position values - prefer 7th rank and center files
pub const ROOK_TABLE: [i32; 64] = [
     0,  0,  0,  5,  5,  0,  0,  0,  // Rank 1
    -5,  0,  0,  0,  0,  0,  0, -5,  // Rank 2
    -5,  0,  0,  0,  0,  0,  0, -5,  // Rank 3
    -5,  0,  0,  0,  0,  0,  0, -5,  // Rank 4
    -5,  0,  0,  0,  0,  0,  0, -5,  // Rank 5
    -5,  0,  0,  0,  0,  0,  0, -5,  // Rank 6
     5, 10, 10, 10, 10, 10, 10,  5,  // Rank 7 (7th rank bonus)
     0,  0,  0,  0,  0,  0,  0,  0,  // Rank 8
];

// Queen position values - slight central preference
pub const QUEEN_TABLE: [i32; 64] = [
    -20,-10,-10, -5, -5,-10,-10,-20,  // Rank 1
    -10,  0,  5,  0,  0,  0,  0,-10,  // Rank 2
    -10,  5,  5,  5,  5,  5,  0,-10,  // Rank 3
      0,  0,  5,  5,  5,  5,  0, -5,  // Rank 4
     -5,  0,  5,  5,  5,  5,  0, -5,  // Rank 5
    -10,  0,  5,  5,  5,  5,  0,-10,  // Rank 6
    -10,  0,  0,  0,  0,  0,  0,-10,  // Rank 7
    -20,-10,-10, -5, -5,-10,-10,-20,  // Rank 8
];

// King middlegame position values - prefer safety on back rank
pub const KING_MIDDLEGAME_TABLE: [i32; 64] = [
     20, 30, 10,  0,  0, 10, 30, 20,  // Rank 1 (castled position)
     20, 20,  0,  0,  0,  0, 20, 20,  // Rank 2
    -10,-20,-20,-20,-20,-20,-20,-10,  // Rank 3
    -20,-30,-30,-40,-40,-30,-30,-20,  // Rank 4
    -30,-40,-40,-50,-50,-40,-40,-30,  // Rank 5
    -30,-40,-40,-50,-50,-40,-40,-30,  // Rank 6
    -30,-40,-40,-50,-50,-40,-40,-30,  // Rank 7
    -30,-40,-40,-50,-50,-40,-40,-30,  // Rank 8
];

// King endgame position values - prefer center activity
pub const KING_ENDGAME_TABLE: [i32; 64] = [
    -50,-30,-30,-30,-30,-30,-30,-50,  // Rank 1
    -30,-30,  0,  0,  0,  0,-30,-30,  // Rank 2
    -30,-10, 20, 30, 30, 20,-10,-30,  // Rank 3
    -30,-10, 30, 40, 40, 30,-10,-30,  // Rank 4
    -30,-10, 30, 40, 40, 30,-10,-30,  // Rank 5
    -30,-10, 20, 30, 30, 20,-10,-30,  // Rank 6
    -30,-20,-10,  0,  0,-10,-20,-30,  // Rank 7
    -50,-40,-30,-20,-20,-30,-40,-50,  // Rank 8
];

/// Get piece-square table value for a given square and piece type
/// For Black pieces, the table is flipped vertically
pub fn get_pst_value(piece_type: crate::game_repr::Type, square: u8, is_white: bool, is_endgame: bool) -> i32 {
    use crate::game_repr::Type;

    // Flip square for Black pieces
    let idx = if is_white {
        square as usize
    } else {
        (63 - square) as usize
    };

    match piece_type {
        Type::Pawn => PAWN_TABLE[idx],
        Type::Knight => KNIGHT_TABLE[idx],
        Type::Bishop => BISHOP_TABLE[idx],
        Type::Rook => ROOK_TABLE[idx],
        Type::Queen => QUEEN_TABLE[idx],
        Type::King => {
            if is_endgame {
                KING_ENDGAME_TABLE[idx]
            } else {
                KING_MIDDLEGAME_TABLE[idx]
            }
        },
        Type::None => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game_repr::Type;

    #[test]
    fn test_pawn_prefers_advancement() {
        // Pawns on rank 7 should be worth more than pawns on rank 2
        let white_pawn_rank2 = get_pst_value(Type::Pawn, 11, true, false); // d2
        let white_pawn_rank7 = get_pst_value(Type::Pawn, 51, true, false); // d7
        assert!(white_pawn_rank7 > white_pawn_rank2);
    }

    #[test]
    fn test_knight_prefers_center() {
        // Knights in center should be worth more than knights on edge
        let knight_center = get_pst_value(Type::Knight, 27, true, false); // d4
        let knight_edge = get_pst_value(Type::Knight, 0, true, false);    // a1
        assert!(knight_center > knight_edge);
    }

    #[test]
    fn test_king_safety_in_middlegame() {
        // King on back rank should be safer than king in center in middlegame
        let king_back_rank = get_pst_value(Type::King, 6, true, false);  // g1
        let king_center = get_pst_value(Type::King, 27, true, false);    // d4
        assert!(king_back_rank > king_center);
    }

    #[test]
    fn test_king_activity_in_endgame() {
        // King in center should be more active in endgame
        let king_back_rank = get_pst_value(Type::King, 6, true, true);   // g1
        let king_center = get_pst_value(Type::King, 27, true, true);     // d4
        assert!(king_center > king_back_rank);
    }

    #[test]
    fn test_black_pieces_flipped() {
        // White pawn on rank 2 should have same value as Black pawn on rank 7
        let white_pawn_rank2 = get_pst_value(Type::Pawn, 11, true, false);  // d2 for White
        let black_pawn_rank7 = get_pst_value(Type::Pawn, 51, false, false); // d7 for Black (which is rank 2 from Black's view)
        assert_eq!(white_pawn_rank2, black_pawn_rank7);
    }
}
