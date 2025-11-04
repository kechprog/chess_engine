// FEN Parsing Tests
//
// This module contains tests for FEN (Forsyth-Edwards Notation) string parsing.
// FEN is a standard notation for describing chess positions.

use crate::game_repr::{Position, Color, Type};

// ==================== FEN PARSING TEST ====================

#[test]
fn test_default_position_from_fen() {
    let pos = Position::default();

    // Check some key pieces are in correct positions
    assert_eq!(pos.position[0].piece_type, Type::Rook);
    assert_eq!(pos.position[0].color, Color::White);

    assert_eq!(pos.position[4].piece_type, Type::King);
    assert_eq!(pos.position[4].color, Color::White);

    assert_eq!(pos.position[60].piece_type, Type::King);
    assert_eq!(pos.position[60].color, Color::Black);

    // Check pawns
    for i in 8..16 {
        assert_eq!(pos.position[i].piece_type, Type::Pawn);
        assert_eq!(pos.position[i].color, Color::White);
    }

    for i in 48..56 {
        assert_eq!(pos.position[i].piece_type, Type::Pawn);
        assert_eq!(pos.position[i].color, Color::Black);
    }
}

#[test]
fn test_custom_fen_position() {
    // Empty board except kings
    let pos = Position::from_fen("4k3/8/8/8/8/8/8/4K3");

    assert_eq!(pos.position[4].piece_type, Type::King);
    assert_eq!(pos.position[4].color, Color::White);

    assert_eq!(pos.position[60].piece_type, Type::King);
    assert_eq!(pos.position[60].color, Color::Black);

    // Check rest is empty
    for i in 0..64 {
        if i != 4 && i != 60 {
            assert_eq!(pos.position[i].piece_type, Type::None);
        }
    }
}
