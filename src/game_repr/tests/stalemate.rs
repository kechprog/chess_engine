use super::*;

// ==================== STALEMATE TESTS ====================

#[test]
fn test_basic_stalemate() {
    let mut pos = empty_board();

    // White king trapped in corner
    place_piece(&mut pos, 0, Piece { color: Color::White, piece_type: Type::King });  // a1

    // Black queen controlling escape squares (but not checking the king)
    place_piece(&mut pos, 10, Piece { color: Color::Black, piece_type: Type::Queen }); // c2

    // Black king (needed for valid position)
    place_piece(&mut pos, 63, Piece { color: Color::Black, piece_type: Type::King });

    assert!(!pos.is_in_check(Color::White), "King should not be in check");
    assert!(!pos.has_legal_moves(Color::White), "Should have no legal moves");
    assert!(pos.is_stalemate(Color::White), "Should be stalemate");
}

#[test]
fn test_not_stalemate_when_in_check() {
    let mut pos = empty_board();

    // White king
    place_piece(&mut pos, 0, Piece { color: Color::White, piece_type: Type::King });

    // Black rook checking the king
    place_piece(&mut pos, 56, Piece { color: Color::Black, piece_type: Type::Rook });

    assert!(pos.is_in_check(Color::White), "King should be in check");
    assert!(!pos.is_stalemate(Color::White), "Not stalemate when in check (it's checkmate)");
}

#[test]
fn test_not_stalemate_has_pawn_move() {
    let mut pos = empty_board();

    // White king trapped
    place_piece(&mut pos, 0, Piece { color: Color::White, piece_type: Type::King });

    // White pawn that can move
    place_piece(&mut pos, 8, Piece { color: Color::White, piece_type: Type::Pawn });

    // Black queen controlling king's squares
    place_piece(&mut pos, 10, Piece { color: Color::Black, piece_type: Type::Queen });

    assert!(!pos.is_stalemate(Color::White), "Not stalemate - pawn can move");
    assert!(pos.has_legal_moves(Color::White), "Should have legal pawn move");
}
