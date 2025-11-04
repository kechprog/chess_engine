use super::*;

// ==================== CHECK DETECTION TESTS ====================

#[test]
fn test_king_in_check_by_rook() {
    let mut pos = empty_board();

    // White king on e1
    place_piece(&mut pos, 4, Piece { color: Color::White, piece_type: Type::King });

    // Black rook on e8
    place_piece(&mut pos, 60, Piece { color: Color::Black, piece_type: Type::Rook });

    assert!(pos.is_in_check(Color::White), "King should be in check from rook");
}

#[test]
fn test_king_in_check_by_bishop() {
    let mut pos = empty_board();

    // White king on e1
    place_piece(&mut pos, 4, Piece { color: Color::White, piece_type: Type::King });

    // Black bishop on a5
    place_piece(&mut pos, 32, Piece { color: Color::Black, piece_type: Type::Bishop });

    assert!(pos.is_in_check(Color::White), "King should be in check from bishop");
}

#[test]
fn test_king_in_check_by_queen() {
    let mut pos = empty_board();

    // White king on e1
    place_piece(&mut pos, 4, Piece { color: Color::White, piece_type: Type::King });

    // Black queen on e8
    place_piece(&mut pos, 60, Piece { color: Color::Black, piece_type: Type::Queen });

    assert!(pos.is_in_check(Color::White), "King should be in check from queen");
}

#[test]
fn test_king_in_check_by_knight() {
    let mut pos = empty_board();

    // White king on e4
    place_piece(&mut pos, 28, Piece { color: Color::White, piece_type: Type::King });

    // Black knight on d6 (can reach e4)
    place_piece(&mut pos, 43, Piece { color: Color::Black, piece_type: Type::Knight });

    assert!(pos.is_in_check(Color::White), "King should be in check from knight");
}

#[test]
fn test_king_in_check_by_pawn() {
    let mut pos = empty_board();

    // White king on e4
    place_piece(&mut pos, 28, Piece { color: Color::White, piece_type: Type::King });

    // Black pawn on d5 (attacks e4 diagonally)
    place_piece(&mut pos, 35, Piece { color: Color::Black, piece_type: Type::Pawn });

    assert!(pos.is_in_check(Color::White), "King should be in check from pawn");
}

#[test]
fn test_cannot_move_into_check() {
    let mut pos = empty_board();

    // White king on e1
    place_piece(&mut pos, 4, Piece { color: Color::White, piece_type: Type::King });

    // Black rook on f8 (controls f-file)
    place_piece(&mut pos, 61, Piece { color: Color::Black, piece_type: Type::Rook });

    let moves = pos.legal_moves(4);

    // King should not be able to move to f1 (into check)
    assert!(!has_move(&moves, 4, 5), "King cannot move into check");
}

#[test]
fn test_must_move_out_of_check() {
    let mut pos = empty_board();

    // White king on e4
    place_piece(&mut pos, 28, Piece { color: Color::White, piece_type: Type::King });

    // Black rook on e8 (checking the king)
    place_piece(&mut pos, 60, Piece { color: Color::Black, piece_type: Type::Rook });

    let moves = pos.legal_moves(28);

    // All legal moves should get the king out of check
    for m in moves.iter() {
        let mut temp_pos = Position {
            position: pos.position,
            prev_moves: pos.prev_moves.clone(),
            castling_cond: pos.castling_cond,
        };
        temp_pos.mk_move(*m);
        assert!(!temp_pos.is_in_check(Color::White), "Move should resolve check");
    }
}

#[test]
fn test_pinned_piece_cannot_move() {
    let mut pos = empty_board();

    // White king on e1
    place_piece(&mut pos, 4, Piece { color: Color::White, piece_type: Type::King });

    // White bishop on e2 (between king and attacker)
    place_piece(&mut pos, 12, Piece { color: Color::White, piece_type: Type::Bishop });

    // Black rook on e8 (pinning the bishop)
    place_piece(&mut pos, 60, Piece { color: Color::Black, piece_type: Type::Rook });

    let moves = pos.legal_moves(12);

    // Bishop should have very limited moves (only along the e-file to block/stay in line)
    // Most importantly, it cannot move off the e-file
    for m in moves.iter() {
        // Any legal move for the bishop should not expose the king to check
        let mut temp_pos = Position {
            position: pos.position,
            prev_moves: pos.prev_moves.clone(),
            castling_cond: pos.castling_cond,
        };
        temp_pos.mk_move(*m);
        assert!(!temp_pos.is_in_check(Color::White), "Pinned piece moves must not expose king");
    }
}
