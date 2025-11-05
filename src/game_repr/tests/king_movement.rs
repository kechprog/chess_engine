use crate::game_repr::{Color, Type, Piece};
use super::{empty_board, has_move, place_piece};

// ==================== KING MOVEMENT TESTS ====================

#[test]
fn test_king_moves_all_directions() {
    let mut pos = empty_board();
    // Place white king in center of board (d4 = 27)
    pos.position[27] = Piece {
        color: Color::White,
        piece_type: Type::King,
    };

    let moves = pos.legal_moves(27);

    // King should be able to move to all 8 adjacent squares
    assert_eq!(moves.len(), 8, "King should have 8 moves from center");

    // Check all 8 directions
    assert!(has_move(&moves, 27, 28)); // Right
    assert!(has_move(&moves, 27, 26)); // Left
    assert!(has_move(&moves, 27, 35)); // Up
    assert!(has_move(&moves, 27, 19)); // Down
    assert!(has_move(&moves, 27, 36)); // Up-Right
    assert!(has_move(&moves, 27, 34)); // Up-Left
    assert!(has_move(&moves, 27, 20)); // Down-Right
    assert!(has_move(&moves, 27, 18)); // Down-Left
}

#[test]
fn test_king_cannot_capture_own_pieces() {
    let mut pos = empty_board();
    // Place white king in center (d4 = 27)
    pos.position[27] = Piece {
        color: Color::White,
        piece_type: Type::King,
    };

    // Surround with white pawns
    place_piece(&mut pos, 28, Piece { color: Color::White, piece_type: Type::Pawn });
    place_piece(&mut pos, 26, Piece { color: Color::White, piece_type: Type::Pawn });
    place_piece(&mut pos, 35, Piece { color: Color::White, piece_type: Type::Pawn });
    place_piece(&mut pos, 19, Piece { color: Color::White, piece_type: Type::Pawn });

    let moves = pos.legal_moves(27);

    // King should only have 4 moves (diagonal directions)
    assert_eq!(moves.len(), 4, "King should not capture own pieces");

    // Should NOT be able to move to squares with own pieces
    assert!(!has_move(&moves, 27, 28));
    assert!(!has_move(&moves, 27, 26));
    assert!(!has_move(&moves, 27, 35));
    assert!(!has_move(&moves, 27, 19));
}

#[test]
fn test_king_cannot_wrap_around_board() {
    let mut pos = empty_board();
    // Place white king on right edge (h4 = 31)
    pos.position[31] = Piece {
        color: Color::White,
        piece_type: Type::King,
    };

    let moves = pos.legal_moves(31);

    // King should have 5 moves (not wrapping to left edge)
    assert_eq!(moves.len(), 5, "King should not wrap around board edges");

    // Should NOT wrap to left side of board (square 24)
    assert!(!has_move(&moves, 31, 24));

    // Place king on left edge (a4 = 24)
    pos = empty_board();
    pos.position[24] = Piece {
        color: Color::White,
        piece_type: Type::King,
    };

    let moves = pos.legal_moves(24);
    assert_eq!(moves.len(), 5, "King should not wrap around board edges");

    // Should NOT wrap to right side (square 31)
    assert!(!has_move(&moves, 24, 31));
}

#[test]
fn test_king_corner_moves() {
    let mut pos = empty_board();
    // Test all four corners

    // Bottom-left corner (a1 = 0)
    pos.position[0] = Piece {
        color: Color::White,
        piece_type: Type::King,
    };
    let moves = pos.legal_moves(0);
    assert_eq!(moves.len(), 3, "King in corner should have 3 moves");

    // Bottom-right corner (h1 = 7)
    pos = empty_board();
    pos.position[7] = Piece {
        color: Color::White,
        piece_type: Type::King,
    };
    let moves = pos.legal_moves(7);
    assert_eq!(moves.len(), 3, "King in corner should have 3 moves");

    // Top-left corner (a8 = 56)
    pos = empty_board();
    pos.position[56] = Piece {
        color: Color::White,
        piece_type: Type::King,
    };
    let moves = pos.legal_moves(56);
    assert_eq!(moves.len(), 3, "King in corner should have 3 moves");

    // Top-right corner (h8 = 63)
    pos = empty_board();
    pos.position[63] = Piece {
        color: Color::White,
        piece_type: Type::King,
    };
    let moves = pos.legal_moves(63);
    assert_eq!(moves.len(), 3, "King in corner should have 3 moves");
}
