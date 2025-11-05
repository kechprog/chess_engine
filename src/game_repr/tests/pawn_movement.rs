use crate::game_repr::{Color, Type, Piece};
use super::{empty_board, has_move, place_piece};

// ==================== PAWN MOVEMENT TESTS ====================

#[test]
fn test_pawn_single_forward_move() {
    let mut pos = empty_board();
    // White pawn on e3
    pos.position[20] = Piece {
        color: Color::White,
        piece_type: Type::Pawn,
    };

    let moves = pos.legal_moves(20);
    assert!(has_move(&moves, 20, 28), "White pawn should move forward one square");

    // Black pawn on e6
    pos = empty_board();
    pos.position[44] = Piece {
        color: Color::Black,
        piece_type: Type::Pawn,
    };

    let moves = pos.legal_moves(44);
    assert!(has_move(&moves, 44, 36), "Black pawn should move forward one square");
}

#[test]
fn test_pawn_double_move_from_starting_rank() {
    let mut pos = empty_board();
    // White pawn on e2 (starting rank)
    pos.position[12] = Piece {
        color: Color::White,
        piece_type: Type::Pawn,
    };

    let moves = pos.legal_moves(12);
    assert!(has_move(&moves, 12, 20), "White pawn should move one square");
    assert!(has_move(&moves, 12, 28), "White pawn should move two squares from start");

    // Black pawn on e7 (starting rank)
    pos = empty_board();
    pos.position[52] = Piece {
        color: Color::Black,
        piece_type: Type::Pawn,
    };

    let moves = pos.legal_moves(52);
    assert!(has_move(&moves, 52, 44), "Black pawn should move one square");
    assert!(has_move(&moves, 52, 36), "Black pawn should move two squares from start");
}

#[test]
fn test_pawn_blocked_by_piece() {
    let mut pos = empty_board();
    // White pawn on e2
    pos.position[12] = Piece {
        color: Color::White,
        piece_type: Type::Pawn,
    };
    // Block with another piece on e3
    pos.position[20] = Piece {
        color: Color::Black,
        piece_type: Type::Pawn,
    };

    let moves = pos.legal_moves(12);
    assert_eq!(moves.len(), 0, "Blocked pawn should have no moves");
}

#[test]
fn test_pawn_diagonal_capture() {
    let mut pos = empty_board();
    // White pawn on d4
    pos.position[27] = Piece {
        color: Color::White,
        piece_type: Type::Pawn,
    };
    // Black pieces on diagonals
    place_piece(&mut pos, 34, Piece { color: Color::Black, piece_type: Type::Pawn }); // e5
    place_piece(&mut pos, 36, Piece { color: Color::Black, piece_type: Type::Pawn }); // c5

    let moves = pos.legal_moves(27);

    // Should be able to capture on both diagonals
    assert!(has_move(&moves, 27, 34), "Pawn should capture diagonally right");
    assert!(has_move(&moves, 27, 36), "Pawn should capture diagonally left");
    // And move forward
    assert!(has_move(&moves, 27, 35), "Pawn should move forward");
}

#[test]
fn test_pawn_cannot_capture_own_pieces() {
    let mut pos = empty_board();
    // White pawn on d4
    pos.position[27] = Piece {
        color: Color::White,
        piece_type: Type::Pawn,
    };
    // White pieces on diagonals
    place_piece(&mut pos, 34, Piece { color: Color::White, piece_type: Type::Pawn });
    place_piece(&mut pos, 36, Piece { color: Color::White, piece_type: Type::Pawn });

    let moves = pos.legal_moves(27);

    // Should NOT capture own pieces
    assert!(!has_move(&moves, 27, 34));
    assert!(!has_move(&moves, 27, 36));
    // Should still move forward
    assert!(has_move(&moves, 27, 35));
}
