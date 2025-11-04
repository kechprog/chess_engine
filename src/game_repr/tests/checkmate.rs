use super::*;

// ==================== CHECKMATE TESTS ====================

#[test]
fn test_fools_mate() {
    // Fool's mate is the quickest possible checkmate (2 moves)
    let mut pos = Position::default();

    // 1. f3
    pos.mk_move(Move::new(13, 21, MoveType::Normal)); // f2-f3

    // 1... e5
    pos.mk_move(Move::new(52, 36, MoveType::Normal)); // e7-e5

    // 2. g4
    pos.mk_move(Move::new(14, 30, MoveType::Normal)); // g2-g4

    // 2... Qh4# (checkmate)
    pos.mk_move(Move::new(59, 31, MoveType::Normal)); // Qd8-h4

    assert!(pos.is_checkmate(Color::White), "Should be checkmate (Fool's mate)");
    assert!(!pos.has_legal_moves(Color::White), "White should have no legal moves");
}

#[test]
fn test_scholars_mate() {
    // Scholar's mate (4-move checkmate)
    let mut pos = Position::default();

    // 1. e4 e5
    pos.mk_move(Move::new(12, 28, MoveType::Normal));
    pos.mk_move(Move::new(52, 36, MoveType::Normal));

    // 2. Bc4 Nc6
    pos.mk_move(Move::new(5, 26, MoveType::Normal));
    pos.mk_move(Move::new(57, 42, MoveType::Normal));

    // 3. Qh5 Nf6
    pos.mk_move(Move::new(3, 31, MoveType::Normal));
    pos.mk_move(Move::new(62, 45, MoveType::Normal));

    // 4. Qxf7# (checkmate)
    pos.mk_move(Move::new(31, 53, MoveType::Normal));

    assert!(pos.is_checkmate(Color::Black), "Should be checkmate (Scholar's mate)");
}

#[test]
fn test_back_rank_mate() {
    let mut pos = empty_board();

    // White king trapped on back rank by own pawns
    pos.position[6] = Piece { color: Color::White, piece_type: Type::King };  // g1
    pos.position[13] = Piece { color: Color::White, piece_type: Type::Pawn }; // f2
    pos.position[14] = Piece { color: Color::White, piece_type: Type::Pawn }; // g2
    pos.position[15] = Piece { color: Color::White, piece_type: Type::Pawn }; // h2

    // Black rook delivers checkmate on back rank (on a1, far from king)
    pos.position[0] = Piece { color: Color::Black, piece_type: Type::Rook };  // a1

    // Add protection so king can't escape to f1 or h1
    pos.position[61] = Piece { color: Color::Black, piece_type: Type::Queen }; // f8 (controls f1 and h1)

    assert!(pos.is_in_check(Color::White), "King should be in check");
    assert!(pos.is_checkmate(Color::White), "Should be back rank mate");
}

#[test]
fn test_not_checkmate_can_block() {
    let mut pos = empty_board();

    // White king on e1
    pos.position[4] = Piece { color: Color::White, piece_type: Type::King };

    // White bishop that can block
    pos.position[21] = Piece { color: Color::White, piece_type: Type::Bishop };

    // Black rook checking the king
    pos.position[60] = Piece { color: Color::Black, piece_type: Type::Rook };

    assert!(pos.is_in_check(Color::White), "King should be in check");
    assert!(!pos.is_checkmate(Color::White), "Not checkmate - can block with bishop");
    assert!(pos.has_legal_moves(Color::White), "Should have legal moves to block");
}

#[test]
fn test_not_checkmate_can_capture() {
    let mut pos = empty_board();

    // White king on e4
    pos.position[28] = Piece { color: Color::White, piece_type: Type::King };

    // White rook that can capture on e7
    pos.position[52] = Piece { color: Color::White, piece_type: Type::Rook };

    // Black queen checking the king on e1 (can be captured by rook on same file)
    pos.position[4] = Piece { color: Color::Black, piece_type: Type::Queen };

    assert!(pos.is_in_check(Color::White), "King should be in check");
    assert!(!pos.is_checkmate(Color::White), "Not checkmate - can capture attacker");
}
