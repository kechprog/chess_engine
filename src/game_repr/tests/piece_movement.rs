use crate::game_repr::{Position, Color, Type, Piece, Move, MoveType};
use super::{empty_board, has_move, place_piece};

// ==================== OTHER PIECE MOVEMENT TESTS ====================

#[test]
fn test_knight_moves() {
    let mut pos = empty_board();

    // Knight on e4
    place_piece(&mut pos, 28, Piece { color: Color::White, piece_type: Type::Knight });

    let moves = pos.legal_moves(28);

    // Knight should have 8 possible moves from center
    assert_eq!(moves.len(), 8, "Knight should have 8 moves from center");
}

#[test]
fn test_bishop_moves() {
    let mut pos = empty_board();

    // Bishop on e4
    place_piece(&mut pos, 28, Piece { color: Color::White, piece_type: Type::Bishop });

    let moves = pos.legal_moves(28);

    // Bishop should have 13 diagonal moves from e4
    assert_eq!(moves.len(), 13, "Bishop should have 13 moves from e4");
}

#[test]
fn test_rook_moves() {
    let mut pos = empty_board();

    // Rook on e4
    place_piece(&mut pos, 28, Piece { color: Color::White, piece_type: Type::Rook });

    let moves = pos.legal_moves(28);

    // Rook should have 14 moves from e4 (7 vertical + 7 horizontal)
    assert_eq!(moves.len(), 14, "Rook should have 14 moves from e4");
}

#[test]
fn test_queen_moves() {
    let mut pos = empty_board();

    // Queen on e4
    place_piece(&mut pos, 28, Piece { color: Color::White, piece_type: Type::Queen });

    let moves = pos.legal_moves(28);

    // Queen should have 27 moves from e4 (combines rook + bishop)
    assert_eq!(moves.len(), 27, "Queen should have 27 moves from e4");
}

#[test]
fn test_rook_blocked_by_own_piece() {
    let mut pos = empty_board();

    // Rook on a1
    place_piece(&mut pos, 0, Piece { color: Color::White, piece_type: Type::Rook });

    // White pawn on a3
    place_piece(&mut pos, 16, Piece { color: Color::White, piece_type: Type::Pawn });

    let moves = pos.legal_moves(0);

    // Should not be able to move past own piece
    assert!(!has_move(&moves, 0, 24), "Rook cannot jump over own piece");
    assert!(has_move(&moves, 0, 8), "Rook can move to square before own piece");
}

#[test]
fn test_bishop_captures_opponent() {
    let mut pos = empty_board();

    // White bishop on a1
    place_piece(&mut pos, 0, Piece { color: Color::White, piece_type: Type::Bishop });

    // Black pawn on c3
    place_piece(&mut pos, 18, Piece { color: Color::Black, piece_type: Type::Pawn });

    let moves = pos.legal_moves(0);

    // Should be able to capture
    assert!(has_move(&moves, 0, 18), "Bishop should capture opponent piece");

    // Should not be able to move past it
    assert!(!has_move(&moves, 0, 27), "Bishop cannot move past captured piece");
}
