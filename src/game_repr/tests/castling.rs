use super::*;

// ==================== CASTLING TESTS ====================

#[test]
fn test_white_kingside_castling_legal() {
    let mut pos = empty_board();

    // Set up white king and rook in starting positions
    place_piece(&mut pos, 4, Piece { color: Color::White, piece_type: Type::King });  // e1
    place_piece(&mut pos, 7, Piece { color: Color::White, piece_type: Type::Rook });  // h1

    // Enable castling rights
    pos.castling_cond[0] = true; // White kingside rook
    pos.castling_cond[2] = true; // White king

    let moves = pos.legal_moves(4);
    let castling_moves = count_move_type(&moves, MoveType::Castling);

    assert!(castling_moves > 0, "White should be able to castle kingside");
    assert!(has_move(&moves, 4, 6), "King should move to g1");
}

#[test]
fn test_white_queenside_castling_legal() {
    let mut pos = empty_board();

    // Set up white king and rook
    place_piece(&mut pos, 4, Piece { color: Color::White, piece_type: Type::King });  // e1
    place_piece(&mut pos, 0, Piece { color: Color::White, piece_type: Type::Rook });  // a1

    pos.castling_cond[1] = true; // White queenside rook
    pos.castling_cond[2] = true; // White king

    let moves = pos.legal_moves(4);
    let castling_moves = count_move_type(&moves, MoveType::Castling);

    assert!(castling_moves > 0, "White should be able to castle queenside");
    assert!(has_move(&moves, 4, 2), "King should move to c1");
}

#[test]
fn test_black_kingside_castling_legal() {
    let mut pos = empty_board();

    // Set up black king and rook
    place_piece(&mut pos, 60, Piece { color: Color::Black, piece_type: Type::King });  // e8
    place_piece(&mut pos, 63, Piece { color: Color::Black, piece_type: Type::Rook });  // h8

    pos.castling_cond[3] = true; // Black kingside rook
    pos.castling_cond[5] = true; // Black king

    let moves = pos.legal_moves(60);
    let castling_moves = count_move_type(&moves, MoveType::Castling);

    assert!(castling_moves > 0, "Black should be able to castle kingside");
    assert!(has_move(&moves, 60, 62), "King should move to g8");
}

#[test]
fn test_black_queenside_castling_legal() {
    let mut pos = empty_board();

    // Set up black king and rook
    place_piece(&mut pos, 60, Piece { color: Color::Black, piece_type: Type::King });  // e8
    place_piece(&mut pos, 56, Piece { color: Color::Black, piece_type: Type::Rook });  // a8

    pos.castling_cond[4] = true; // Black queenside rook
    pos.castling_cond[5] = true; // Black king

    let moves = pos.legal_moves(60);
    let castling_moves = count_move_type(&moves, MoveType::Castling);

    assert!(castling_moves > 0, "Black should be able to castle queenside");
    assert!(has_move(&moves, 60, 58), "King should move to c8");
}

#[test]
fn test_castling_blocked_by_pieces() {
    let mut pos = empty_board();

    place_piece(&mut pos, 4, Piece { color: Color::White, piece_type: Type::King });
    place_piece(&mut pos, 7, Piece { color: Color::White, piece_type: Type::Rook });
    place_piece(&mut pos, 5, Piece { color: Color::White, piece_type: Type::Knight }); // Block f1

    pos.castling_cond[0] = true;
    pos.castling_cond[2] = true;

    let moves = pos.legal_moves(4);
    let castling_moves = count_move_type(&moves, MoveType::Castling);

    assert_eq!(castling_moves, 0, "Cannot castle when pieces block the path");
}

#[test]
fn test_castling_prevented_when_king_moved() {
    let mut pos = empty_board();

    place_piece(&mut pos, 4, Piece { color: Color::White, piece_type: Type::King });
    place_piece(&mut pos, 7, Piece { color: Color::White, piece_type: Type::Rook });

    pos.castling_cond[0] = true;
    pos.castling_cond[2] = true;

    // Move king and move it back
    pos.mk_move(Move::new(4, 5, MoveType::Normal));
    pos.mk_move(Move::new(5, 4, MoveType::Normal));

    // Castling should be disabled
    assert_eq!(pos.castling_cond[2], false, "King castling right should be disabled");

    let moves = pos.legal_moves(4);
    let castling_moves = count_move_type(&moves, MoveType::Castling);

    assert_eq!(castling_moves, 0, "Cannot castle after king has moved");
}

#[test]
fn test_castling_prevented_when_rook_moved() {
    let mut pos = empty_board();

    place_piece(&mut pos, 4, Piece { color: Color::White, piece_type: Type::King });
    place_piece(&mut pos, 7, Piece { color: Color::White, piece_type: Type::Rook });

    pos.castling_cond[0] = true;
    pos.castling_cond[2] = true;

    // Move rook and move it back
    pos.mk_move(Move::new(7, 6, MoveType::Normal));
    pos.mk_move(Move::new(6, 7, MoveType::Normal));

    // Kingside castling should be disabled
    assert_eq!(pos.castling_cond[0], false, "Rook castling right should be disabled");

    let moves = pos.legal_moves(4);
    let castling_moves = count_move_type(&moves, MoveType::Castling);

    assert_eq!(castling_moves, 0, "Cannot castle after rook has moved");
}

#[test]
fn test_castling_prevented_when_in_check() {
    let mut pos = empty_board();

    // White king and rook
    place_piece(&mut pos, 4, Piece { color: Color::White, piece_type: Type::King });
    place_piece(&mut pos, 7, Piece { color: Color::White, piece_type: Type::Rook });

    // Black rook attacking the king
    place_piece(&mut pos, 60, Piece { color: Color::Black, piece_type: Type::Rook });

    pos.castling_cond[0] = true;
    pos.castling_cond[2] = true;

    let moves = pos.legal_moves(4);
    let castling_moves = count_move_type(&moves, MoveType::Castling);

    assert_eq!(castling_moves, 0, "Cannot castle while in check");
}

#[test]
fn test_castling_prevented_when_passing_through_check() {
    let mut pos = empty_board();

    // White king and rook
    place_piece(&mut pos, 4, Piece { color: Color::White, piece_type: Type::King });
    place_piece(&mut pos, 7, Piece { color: Color::White, piece_type: Type::Rook });

    // Black rook attacking f1 (square king passes through)
    place_piece(&mut pos, 61, Piece { color: Color::Black, piece_type: Type::Rook });

    pos.castling_cond[0] = true;
    pos.castling_cond[2] = true;

    let moves = pos.legal_moves(4);
    let castling_moves = count_move_type(&moves, MoveType::Castling);

    assert_eq!(castling_moves, 0, "Cannot castle through check");
}

#[test]
fn test_castling_prevented_when_landing_in_check() {
    let mut pos = empty_board();

    // White king and rook
    place_piece(&mut pos, 4, Piece { color: Color::White, piece_type: Type::King });
    place_piece(&mut pos, 7, Piece { color: Color::White, piece_type: Type::Rook });

    // Black rook attacking g1 (square king lands on)
    place_piece(&mut pos, 62, Piece { color: Color::Black, piece_type: Type::Rook });

    pos.castling_cond[0] = true;
    pos.castling_cond[2] = true;

    let moves = pos.legal_moves(4);
    let castling_moves = count_move_type(&moves, MoveType::Castling);

    assert_eq!(castling_moves, 0, "Cannot castle into check");
}

#[test]
fn test_castling_execution_kingside() {
    let mut pos = empty_board();

    place_piece(&mut pos, 4, Piece { color: Color::White, piece_type: Type::King });
    place_piece(&mut pos, 7, Piece { color: Color::White, piece_type: Type::Rook });

    pos.castling_cond[0] = true;
    pos.castling_cond[2] = true;

    // Execute castling
    pos.mk_move(Move::new(4, 6, MoveType::Castling));

    // Check king is on g1
    assert_eq!(pos.position[6].piece_type, Type::King);
    assert_eq!(pos.position[6].color, Color::White);

    // Check rook is on f1
    assert_eq!(pos.position[5].piece_type, Type::Rook);
    assert_eq!(pos.position[5].color, Color::White);

    // Check e1 and h1 are empty
    assert_eq!(pos.position[4].piece_type, Type::None);
    assert_eq!(pos.position[7].piece_type, Type::None);
}

#[test]
fn test_castling_execution_queenside() {
    let mut pos = empty_board();

    place_piece(&mut pos, 4, Piece { color: Color::White, piece_type: Type::King });
    place_piece(&mut pos, 0, Piece { color: Color::White, piece_type: Type::Rook });

    pos.castling_cond[1] = true;
    pos.castling_cond[2] = true;

    // Execute castling
    pos.mk_move(Move::new(4, 2, MoveType::Castling));

    // Check king is on c1
    assert_eq!(pos.position[2].piece_type, Type::King);
    assert_eq!(pos.position[2].color, Color::White);

    // Check rook is on d1
    assert_eq!(pos.position[3].piece_type, Type::Rook);
    assert_eq!(pos.position[3].color, Color::White);

    // Check e1 and a1 are empty
    assert_eq!(pos.position[4].piece_type, Type::None);
    assert_eq!(pos.position[0].piece_type, Type::None);
}

#[test]
fn test_castling_rights_updated_when_rook_captured() {
    let mut pos = empty_board();

    place_piece(&mut pos, 4, Piece { color: Color::White, piece_type: Type::King });
    place_piece(&mut pos, 7, Piece { color: Color::White, piece_type: Type::Rook });
    place_piece(&mut pos, 15, Piece { color: Color::Black, piece_type: Type::Rook });

    pos.castling_cond[0] = true;
    pos.castling_cond[2] = true;

    // Black rook captures white rook on h1
    pos.mk_move(Move::new(15, 7, MoveType::Normal));

    // Kingside castling should be disabled
    assert_eq!(pos.castling_cond[0], false, "Castling rights should be revoked when rook captured");
}
