use super::*;

// ==================== PROMOTION TESTS ====================

#[test]
fn test_pawn_promotion_on_back_rank() {
    let mut pos = empty_board();

    // White pawn on 7th rank
    pos.position[54] = Piece { color: Color::White, piece_type: Type::Pawn }; // g7

    let moves = pos.legal_moves(54);

    // Check that move to 8th rank is a promotion
    let promotion_moves = count_move_type(&moves, MoveType::PromotionQueen);
    assert!(promotion_moves > 0, "Pawn should promote on reaching back rank");
}

#[test]
fn test_pawn_promotion_execution() {
    let mut pos = empty_board();

    // White pawn on 7th rank
    pos.position[54] = Piece { color: Color::White, piece_type: Type::Pawn };

    // Move to 8th rank (should auto-promote to queen)
    pos.mk_move(Move::new(54, 62, MoveType::PromotionQueen));

    // Check it's a queen
    assert_eq!(pos.position[62].piece_type, Type::Queen);
    assert_eq!(pos.position[62].color, Color::White);

    // Check original square is empty
    assert_eq!(pos.position[54].piece_type, Type::None);
}

#[test]
fn test_pawn_promotion_on_capture() {
    let mut pos = empty_board();

    // White pawn on 7th rank
    pos.position[54] = Piece { color: Color::White, piece_type: Type::Pawn }; // g7

    // Black piece on h8
    pos.position[63] = Piece { color: Color::Black, piece_type: Type::Rook };

    let moves = pos.legal_moves(54);

    // Should have promotion move for capture
    let has_promotion_capture = moves.iter().any(|m| {
        m._from() == 54 && m._to() == 63 && m.move_type().is_promotion()
    });

    assert!(has_promotion_capture, "Pawn should promote when capturing on back rank");
}

#[test]
fn test_black_pawn_promotion() {
    let mut pos = empty_board();

    // Black pawn on 2nd rank
    pos.position[9] = Piece { color: Color::Black, piece_type: Type::Pawn }; // b2

    let moves = pos.legal_moves(9);

    // Check for promotion move
    let promotion_moves = count_move_type(&moves, MoveType::PromotionQueen);
    assert!(promotion_moves > 0, "Black pawn should promote on reaching 1st rank");

    // Execute promotion
    pos.mk_move(Move::new(9, 1, MoveType::PromotionQueen));

    assert_eq!(pos.position[1].piece_type, Type::Queen);
    assert_eq!(pos.position[1].color, Color::Black);
}
