// Regression tests for previously found bugs
//
// This module contains tests that verify bugs that were discovered and fixed
// during development. These tests ensure the bugs don't resurface.

use super::*;

// ==================== PAWN EDGE SQUARE BUG TEST ====================

#[test]
fn test_pawn_edge_square_capture_bug() {
    // Regression test for bug where black pawn at a3 could capture at h1
    // This was caused by using offset=0 for edge squares, which created invalid captures
    let mut pos = empty_board();

    // Place black pawn at a3 (index 16 - left edge)
    place_piece(&mut pos, 16, Piece {
        color: Color::Black,
        piece_type: Type::Pawn,
    });

    // Place white rook at h1 (index 7 - should NOT be capturable by a3 pawn!)
    place_piece(&mut pos, 7, Piece {
        color: Color::White,
        piece_type: Type::Rook,
    });

    // Place white king and black king (required for legal moves)
    place_piece(&mut pos, 4, Piece {
        color: Color::White,
        piece_type: Type::King,
    });
    place_piece(&mut pos, 60, Piece {
        color: Color::Black,
        piece_type: Type::King,
    });

    let moves = pos.legal_moves(16);

    // The pawn at a3 should only be able to move to a2 (index 8)
    // It should NOT be able to move to h1 (index 7)
    assert_eq!(moves.len(), 1, "Black pawn at a3 should only have 1 move (a2)");
    assert!(has_move(&moves, 16, 8), "Pawn should be able to move from a3 to a2");
    assert!(!has_move(&moves, 16, 7), "BUG: Pawn should NOT be able to move from a3 to h1!");
}

#[test]
fn test_pawn_edge_square_right_edge() {
    // Test right edge (h-file) pawns as well
    let mut pos = empty_board();

    // Place white pawn at h3 (index 23 - right edge)
    place_piece(&mut pos, 23, Piece {
        color: Color::White,
        piece_type: Type::Pawn,
    });

    // Place black rook at a5 (index 32 - should NOT be capturable!)
    place_piece(&mut pos, 32, Piece {
        color: Color::Black,
        piece_type: Type::Rook,
    });

    // Place kings
    place_piece(&mut pos, 4, Piece {
        color: Color::White,
        piece_type: Type::King,
    });
    place_piece(&mut pos, 60, Piece {
        color: Color::Black,
        piece_type: Type::King,
    });

    let moves = pos.legal_moves(23);

    // The pawn at h3 should only be able to move to h4 (index 31)
    // It should NOT wrap around to capture on the a-file
    assert_eq!(moves.len(), 1, "White pawn at h3 should only have 1 move (h4)");
    assert!(has_move(&moves, 23, 31), "Pawn should be able to move from h3 to h4");
    assert!(!has_move(&moves, 23, 32), "BUG: Pawn should NOT wrap around board edge!");
}
