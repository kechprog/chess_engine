use super::*;

// ==================== EN PASSANT TESTS ====================

#[test]
fn test_en_passant_legal() {
    let mut pos = empty_board();

    // Setup: White pawn on e5, black pawn on d7
    place_piece(&mut pos, 36, Piece { color: Color::White, piece_type: Type::Pawn }); // e5
    place_piece(&mut pos, 51, Piece { color: Color::Black, piece_type: Type::Pawn }); // d7

    // Black pawn moves two squares from d7 to d5
    let black_move = Move::new(51, 35, MoveType::Normal);
    pos.mk_move(black_move);

    // Now white pawn on e5 should be able to capture en passant
    let moves = pos.legal_moves(36);

    // Check that en passant move exists
    let en_passant_moves = moves.iter().filter(|m| {
        matches!(m.move_type(), MoveType::EnPassant)
    }).count();

    assert_eq!(en_passant_moves, 1, "Should have exactly one en passant move");

    // The en passant capture should move to d6 (square 43)
    assert!(moves.iter().any(|m| {
        m._from() == 36 && m._to() == 43 && matches!(m.move_type(), MoveType::EnPassant)
    }), "En passant should capture to d6");
}

#[test]
fn test_en_passant_no_rank_wrapping() {
    let mut pos = empty_board();

    // Place white pawn on h5 (right edge)
    place_piece(&mut pos, 39, Piece { color: Color::White, piece_type: Type::Pawn });

    // Place black pawn on a7 (left edge, different rank)
    place_piece(&mut pos, 48, Piece { color: Color::Black, piece_type: Type::Pawn });

    // Black pawn moves two squares
    let black_move = Move::new(48, 32, MoveType::Normal);
    pos.mk_move(black_move);

    // White pawn should NOT be able to capture en passant (different ranks)
    let moves = pos.legal_moves(39);
    let en_passant_moves = count_move_type(&moves, MoveType::EnPassant);

    assert_eq!(en_passant_moves, 0, "En passant should not wrap across ranks");
}

#[test]
fn test_en_passant_works_on_starting_squares() {
    let mut pos = empty_board();

    // White pawn on d2 (starting square)
    place_piece(&mut pos, 11, Piece { color: Color::White, piece_type: Type::Pawn });

    // Move it two squares to d4
    let move1 = Move::new(11, 27, MoveType::Normal);
    pos.mk_move(move1);

    // Move white pawn from d4 to d5
    let move2 = Move::new(27, 35, MoveType::Normal);
    pos.mk_move(move2);

    // Black pawn on e7
    place_piece(&mut pos, 52, Piece { color: Color::Black, piece_type: Type::Pawn });

    // Black pawn moves two squares to e5 (now adjacent to white pawn on d5)
    let move3 = Move::new(52, 36, MoveType::Normal);
    pos.mk_move(move3);

    // White pawn on d5 should be able to capture en passant on e6
    let moves = pos.legal_moves(35);
    let en_passant_moves = count_move_type(&moves, MoveType::EnPassant);

    assert!(en_passant_moves > 0, "En passant should work even after moving from starting square");
}

#[test]
fn test_en_passant_execution() {
    let mut pos = empty_board();

    // Setup: White pawn on e5, black pawn on d7
    place_piece(&mut pos, 36, Piece { color: Color::White, piece_type: Type::Pawn });
    place_piece(&mut pos, 51, Piece { color: Color::Black, piece_type: Type::Pawn });

    // Black moves d7-d5
    pos.mk_move(Move::new(51, 35, MoveType::Normal));

    // White captures en passant
    let moves = pos.legal_moves(36);
    let en_passant_move = moves.iter().find(|m| {
        matches!(m.move_type(), MoveType::EnPassant)
    }).expect("Should have en passant move");

    pos.mk_move(*en_passant_move);

    // Check that white pawn is on d6
    assert_eq!(pos.position[43].piece_type, Type::Pawn);
    assert_eq!(pos.position[43].color, Color::White);

    // Check that black pawn on d5 is removed
    assert_eq!(pos.position[35].piece_type, Type::None);

    // Check that e5 is now empty
    assert_eq!(pos.position[36].piece_type, Type::None);
}

// ==================== EN PASSANT DIRECTION BUG TEST ====================

#[test]
fn test_en_passant_direction_white() {
    // White pawn at d4, black pawn moves c7->c5 (double move next to white pawn)
    // White pawn at d4 should capture to c6 (toward enemy), not somewhere else
    let mut pos = empty_board();

    // Place white pawn at d4 (index 27 = row 3, col 3)
    place_piece(&mut pos, 27, Piece { color: Color::White, piece_type: Type::Pawn });

    // Place black pawn at c4 (index 26 = row 3, col 2) - adjacent to white pawn
    place_piece(&mut pos, 26, Piece { color: Color::Black, piece_type: Type::Pawn });

    // Simulate the previous move: c6->c4 (index 42->26, distance = 16)
    pos.prev_moves = vec![Move::new(42, 26, MoveType::Normal)];

    // Place kings
    place_piece(&mut pos, 4, Piece { color: Color::White, piece_type: Type::King });
    place_piece(&mut pos, 60, Piece { color: Color::Black, piece_type: Type::King });

    let moves = pos.legal_moves(27);

    // Should be able to capture via en passant to c5 (index 34)
    // White pawn at d4 (27) captures left-forward with offset 7: 27 + 7 = 34 (c5)
    assert!(has_move(&moves, 27, 34), "White pawn at d4 should capture LEFT to c5 via en passant");
}

#[test]
fn test_en_passant_direction_black() {
    // Black pawn at d5, white pawn moves e3->e5 (double move next to black pawn)
    // Black pawn at d5 should capture to e4 (toward enemy), not somewhere else
    let mut pos = empty_board();

    // Place black pawn at d5 (index 35 = row 4, col 3)
    place_piece(&mut pos, 35, Piece { color: Color::Black, piece_type: Type::Pawn });

    // Place white pawn at e5 (index 36 = row 4, col 4) - adjacent to black pawn
    place_piece(&mut pos, 36, Piece { color: Color::White, piece_type: Type::Pawn });

    // Simulate the previous move: e3->e5 (index 20->36, distance = 16)
    pos.prev_moves = vec![Move::new(20, 36, MoveType::Normal)];

    // Place kings
    place_piece(&mut pos, 4, Piece { color: Color::White, piece_type: Type::King });
    place_piece(&mut pos, 60, Piece { color: Color::Black, piece_type: Type::King });

    let moves = pos.legal_moves(35);

    // Should be able to capture via en passant to e4 (index 28)
    // Black pawn at d5 (35) captures right-forward with offset -7: 35 + (-7) = 28 (e4)
    assert!(has_move(&moves, 35, 28), "Black pawn at d5 should capture RIGHT to e4 via en passant");
}
