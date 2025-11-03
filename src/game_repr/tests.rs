#[cfg(test)]
mod tests {
    use crate::game_repr::{Position, Piece, Move, MoveType, Color, Type};

    // ==================== HELPER FUNCTIONS ====================

    /// Helper function to create an empty board
    fn empty_board() -> Position {
        Position {
            position: [Piece::default(); 64],
            prev_moves: Vec::new(),
            castling_cond: [false; 6],
        }
    }

    /// Helper function to check if a move exists in the move list
    fn has_move(moves: &[Move], from: usize, to: usize) -> bool {
        moves.iter().any(|m| m._from() == from && m._to() == to)
    }

    /// Helper function to count moves of a specific type
    fn count_move_type(moves: &[Move], move_type: MoveType) -> usize {
        moves.iter().filter(|m| {
            match (m.move_type(), move_type) {
                (MoveType::Normal, MoveType::Normal) => true,
                (MoveType::EnPassant, MoveType::EnPassant) => true,
                (MoveType::Promotion, MoveType::Promotion) => true,
                (MoveType::Castling, MoveType::Castling) => true,
                _ => false,
            }
        }).count()
    }

    // ==================== KING MOVEMENT TESTS ====================

    #[test]
    fn test_king_moves_all_directions() {
        let mut pos = empty_board();
        // Place white king in center of board (d4 = 27)
        pos.position[27] = Piece {
            color: Color::White,
            piece_type: Type::King,
        };

        let moves = pos.legal_moves(27);

        // King should be able to move to all 8 adjacent squares
        assert_eq!(moves.len(), 8, "King should have 8 moves from center");

        // Check all 8 directions
        assert!(has_move(&moves, 27, 28)); // Right
        assert!(has_move(&moves, 27, 26)); // Left
        assert!(has_move(&moves, 27, 35)); // Up
        assert!(has_move(&moves, 27, 19)); // Down
        assert!(has_move(&moves, 27, 36)); // Up-Right
        assert!(has_move(&moves, 27, 34)); // Up-Left
        assert!(has_move(&moves, 27, 20)); // Down-Right
        assert!(has_move(&moves, 27, 18)); // Down-Left
    }

    #[test]
    fn test_king_cannot_capture_own_pieces() {
        let mut pos = empty_board();
        // Place white king in center (d4 = 27)
        pos.position[27] = Piece {
            color: Color::White,
            piece_type: Type::King,
        };

        // Surround with white pawns
        pos.position[28] = Piece { color: Color::White, piece_type: Type::Pawn };
        pos.position[26] = Piece { color: Color::White, piece_type: Type::Pawn };
        pos.position[35] = Piece { color: Color::White, piece_type: Type::Pawn };
        pos.position[19] = Piece { color: Color::White, piece_type: Type::Pawn };

        let moves = pos.legal_moves(27);

        // King should only have 4 moves (diagonal directions)
        assert_eq!(moves.len(), 4, "King should not capture own pieces");

        // Should NOT be able to move to squares with own pieces
        assert!(!has_move(&moves, 27, 28));
        assert!(!has_move(&moves, 27, 26));
        assert!(!has_move(&moves, 27, 35));
        assert!(!has_move(&moves, 27, 19));
    }

    #[test]
    fn test_king_cannot_wrap_around_board() {
        let mut pos = empty_board();
        // Place white king on right edge (h4 = 31)
        pos.position[31] = Piece {
            color: Color::White,
            piece_type: Type::King,
        };

        let moves = pos.legal_moves(31);

        // King should have 5 moves (not wrapping to left edge)
        assert_eq!(moves.len(), 5, "King should not wrap around board edges");

        // Should NOT wrap to left side of board (square 24)
        assert!(!has_move(&moves, 31, 24));

        // Place king on left edge (a4 = 24)
        pos = empty_board();
        pos.position[24] = Piece {
            color: Color::White,
            piece_type: Type::King,
        };

        let moves = pos.legal_moves(24);
        assert_eq!(moves.len(), 5, "King should not wrap around board edges");

        // Should NOT wrap to right side (square 31)
        assert!(!has_move(&moves, 24, 31));
    }

    #[test]
    fn test_king_corner_moves() {
        let mut pos = empty_board();
        // Test all four corners

        // Bottom-left corner (a1 = 0)
        pos.position[0] = Piece {
            color: Color::White,
            piece_type: Type::King,
        };
        let moves = pos.legal_moves(0);
        assert_eq!(moves.len(), 3, "King in corner should have 3 moves");

        // Bottom-right corner (h1 = 7)
        pos = empty_board();
        pos.position[7] = Piece {
            color: Color::White,
            piece_type: Type::King,
        };
        let moves = pos.legal_moves(7);
        assert_eq!(moves.len(), 3, "King in corner should have 3 moves");

        // Top-left corner (a8 = 56)
        pos = empty_board();
        pos.position[56] = Piece {
            color: Color::White,
            piece_type: Type::King,
        };
        let moves = pos.legal_moves(56);
        assert_eq!(moves.len(), 3, "King in corner should have 3 moves");

        // Top-right corner (h8 = 63)
        pos = empty_board();
        pos.position[63] = Piece {
            color: Color::White,
            piece_type: Type::King,
        };
        let moves = pos.legal_moves(63);
        assert_eq!(moves.len(), 3, "King in corner should have 3 moves");
    }

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
        pos.position[34] = Piece { color: Color::Black, piece_type: Type::Pawn }; // e5
        pos.position[36] = Piece { color: Color::Black, piece_type: Type::Pawn }; // c5

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
        pos.position[34] = Piece { color: Color::White, piece_type: Type::Pawn };
        pos.position[36] = Piece { color: Color::White, piece_type: Type::Pawn };

        let moves = pos.legal_moves(27);

        // Should NOT capture own pieces
        assert!(!has_move(&moves, 27, 34));
        assert!(!has_move(&moves, 27, 36));
        // Should still move forward
        assert!(has_move(&moves, 27, 35));
    }

    // ==================== EN PASSANT TESTS ====================

    #[test]
    fn test_en_passant_legal() {
        let mut pos = empty_board();

        // Setup: White pawn on e5, black pawn on d7
        pos.position[36] = Piece { color: Color::White, piece_type: Type::Pawn }; // e5
        pos.position[51] = Piece { color: Color::Black, piece_type: Type::Pawn }; // d7

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
        pos.position[39] = Piece { color: Color::White, piece_type: Type::Pawn };

        // Place black pawn on a7 (left edge, different rank)
        pos.position[48] = Piece { color: Color::Black, piece_type: Type::Pawn };

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
        pos.position[11] = Piece { color: Color::White, piece_type: Type::Pawn };

        // Move it two squares to d4
        let move1 = Move::new(11, 27, MoveType::Normal);
        pos.mk_move(move1);

        // Move white pawn from d4 to d5
        let move2 = Move::new(27, 35, MoveType::Normal);
        pos.mk_move(move2);

        // Black pawn on e7
        pos.position[52] = Piece { color: Color::Black, piece_type: Type::Pawn };

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
        pos.position[36] = Piece { color: Color::White, piece_type: Type::Pawn };
        pos.position[51] = Piece { color: Color::Black, piece_type: Type::Pawn };

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

    // ==================== CASTLING TESTS ====================

    #[test]
    fn test_white_kingside_castling_legal() {
        let mut pos = empty_board();

        // Set up white king and rook in starting positions
        pos.position[4] = Piece { color: Color::White, piece_type: Type::King };  // e1
        pos.position[7] = Piece { color: Color::White, piece_type: Type::Rook };  // h1

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
        pos.position[4] = Piece { color: Color::White, piece_type: Type::King };  // e1
        pos.position[0] = Piece { color: Color::White, piece_type: Type::Rook };  // a1

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
        pos.position[60] = Piece { color: Color::Black, piece_type: Type::King };  // e8
        pos.position[63] = Piece { color: Color::Black, piece_type: Type::Rook };  // h8

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
        pos.position[60] = Piece { color: Color::Black, piece_type: Type::King };  // e8
        pos.position[56] = Piece { color: Color::Black, piece_type: Type::Rook };  // a8

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

        pos.position[4] = Piece { color: Color::White, piece_type: Type::King };
        pos.position[7] = Piece { color: Color::White, piece_type: Type::Rook };
        pos.position[5] = Piece { color: Color::White, piece_type: Type::Knight }; // Block f1

        pos.castling_cond[0] = true;
        pos.castling_cond[2] = true;

        let moves = pos.legal_moves(4);
        let castling_moves = count_move_type(&moves, MoveType::Castling);

        assert_eq!(castling_moves, 0, "Cannot castle when pieces block the path");
    }

    #[test]
    fn test_castling_prevented_when_king_moved() {
        let mut pos = empty_board();

        pos.position[4] = Piece { color: Color::White, piece_type: Type::King };
        pos.position[7] = Piece { color: Color::White, piece_type: Type::Rook };

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

        pos.position[4] = Piece { color: Color::White, piece_type: Type::King };
        pos.position[7] = Piece { color: Color::White, piece_type: Type::Rook };

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
        pos.position[4] = Piece { color: Color::White, piece_type: Type::King };
        pos.position[7] = Piece { color: Color::White, piece_type: Type::Rook };

        // Black rook attacking the king
        pos.position[60] = Piece { color: Color::Black, piece_type: Type::Rook };

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
        pos.position[4] = Piece { color: Color::White, piece_type: Type::King };
        pos.position[7] = Piece { color: Color::White, piece_type: Type::Rook };

        // Black rook attacking f1 (square king passes through)
        pos.position[61] = Piece { color: Color::Black, piece_type: Type::Rook };

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
        pos.position[4] = Piece { color: Color::White, piece_type: Type::King };
        pos.position[7] = Piece { color: Color::White, piece_type: Type::Rook };

        // Black rook attacking g1 (square king lands on)
        pos.position[62] = Piece { color: Color::Black, piece_type: Type::Rook };

        pos.castling_cond[0] = true;
        pos.castling_cond[2] = true;

        let moves = pos.legal_moves(4);
        let castling_moves = count_move_type(&moves, MoveType::Castling);

        assert_eq!(castling_moves, 0, "Cannot castle into check");
    }

    #[test]
    fn test_castling_execution_kingside() {
        let mut pos = empty_board();

        pos.position[4] = Piece { color: Color::White, piece_type: Type::King };
        pos.position[7] = Piece { color: Color::White, piece_type: Type::Rook };

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

        pos.position[4] = Piece { color: Color::White, piece_type: Type::King };
        pos.position[0] = Piece { color: Color::White, piece_type: Type::Rook };

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

        pos.position[4] = Piece { color: Color::White, piece_type: Type::King };
        pos.position[7] = Piece { color: Color::White, piece_type: Type::Rook };
        pos.position[15] = Piece { color: Color::Black, piece_type: Type::Rook };

        pos.castling_cond[0] = true;
        pos.castling_cond[2] = true;

        // Black rook captures white rook on h1
        pos.mk_move(Move::new(15, 7, MoveType::Normal));

        // Kingside castling should be disabled
        assert_eq!(pos.castling_cond[0], false, "Castling rights should be revoked when rook captured");
    }

    // ==================== PROMOTION TESTS ====================

    #[test]
    fn test_pawn_promotion_on_back_rank() {
        let mut pos = empty_board();

        // White pawn on 7th rank
        pos.position[54] = Piece { color: Color::White, piece_type: Type::Pawn }; // g7

        let moves = pos.legal_moves(54);

        // Check that move to 8th rank is a promotion
        let promotion_moves = count_move_type(&moves, MoveType::Promotion);
        assert!(promotion_moves > 0, "Pawn should promote on reaching back rank");
    }

    #[test]
    fn test_pawn_promotion_execution() {
        let mut pos = empty_board();

        // White pawn on 7th rank
        pos.position[54] = Piece { color: Color::White, piece_type: Type::Pawn };

        // Move to 8th rank (should auto-promote to queen)
        pos.mk_move(Move::new(54, 62, MoveType::Promotion));

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
            m._from() == 54 && m._to() == 63 && matches!(m.move_type(), MoveType::Promotion)
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
        let promotion_moves = count_move_type(&moves, MoveType::Promotion);
        assert!(promotion_moves > 0, "Black pawn should promote on reaching 1st rank");

        // Execute promotion
        pos.mk_move(Move::new(9, 1, MoveType::Promotion));

        assert_eq!(pos.position[1].piece_type, Type::Queen);
        assert_eq!(pos.position[1].color, Color::Black);
    }

    // ==================== CHECK DETECTION TESTS ====================

    #[test]
    fn test_king_in_check_by_rook() {
        let mut pos = empty_board();

        // White king on e1
        pos.position[4] = Piece { color: Color::White, piece_type: Type::King };

        // Black rook on e8
        pos.position[60] = Piece { color: Color::Black, piece_type: Type::Rook };

        assert!(pos.is_in_check(Color::White), "King should be in check from rook");
    }

    #[test]
    fn test_king_in_check_by_bishop() {
        let mut pos = empty_board();

        // White king on e1
        pos.position[4] = Piece { color: Color::White, piece_type: Type::King };

        // Black bishop on a5
        pos.position[32] = Piece { color: Color::Black, piece_type: Type::Bishop };

        assert!(pos.is_in_check(Color::White), "King should be in check from bishop");
    }

    #[test]
    fn test_king_in_check_by_queen() {
        let mut pos = empty_board();

        // White king on e1
        pos.position[4] = Piece { color: Color::White, piece_type: Type::King };

        // Black queen on e8
        pos.position[60] = Piece { color: Color::Black, piece_type: Type::Queen };

        assert!(pos.is_in_check(Color::White), "King should be in check from queen");
    }

    #[test]
    fn test_king_in_check_by_knight() {
        let mut pos = empty_board();

        // White king on e4
        pos.position[28] = Piece { color: Color::White, piece_type: Type::King };

        // Black knight on d6 (can reach e4)
        pos.position[43] = Piece { color: Color::Black, piece_type: Type::Knight };

        assert!(pos.is_in_check(Color::White), "King should be in check from knight");
    }

    #[test]
    fn test_king_in_check_by_pawn() {
        let mut pos = empty_board();

        // White king on e4
        pos.position[28] = Piece { color: Color::White, piece_type: Type::King };

        // Black pawn on d5 (attacks e4 diagonally)
        pos.position[35] = Piece { color: Color::Black, piece_type: Type::Pawn };

        assert!(pos.is_in_check(Color::White), "King should be in check from pawn");
    }

    #[test]
    fn test_cannot_move_into_check() {
        let mut pos = empty_board();

        // White king on e1
        pos.position[4] = Piece { color: Color::White, piece_type: Type::King };

        // Black rook on f8 (controls f-file)
        pos.position[61] = Piece { color: Color::Black, piece_type: Type::Rook };

        let moves = pos.legal_moves(4);

        // King should not be able to move to f1 (into check)
        assert!(!has_move(&moves, 4, 5), "King cannot move into check");
    }

    #[test]
    fn test_must_move_out_of_check() {
        let mut pos = empty_board();

        // White king on e4
        pos.position[28] = Piece { color: Color::White, piece_type: Type::King };

        // Black rook on e8 (checking the king)
        pos.position[60] = Piece { color: Color::Black, piece_type: Type::Rook };

        let moves = pos.legal_moves(28);

        // All legal moves should get the king out of check
        for m in moves.iter() {
            let mut temp_pos = Position {
                position: pos.position.clone(),
                prev_moves: pos.prev_moves.clone(),
                castling_cond: pos.castling_cond.clone(),
            };
            temp_pos.mk_move(*m);
            assert!(!temp_pos.is_in_check(Color::White), "Move should resolve check");
        }
    }

    #[test]
    fn test_pinned_piece_cannot_move() {
        let mut pos = empty_board();

        // White king on e1
        pos.position[4] = Piece { color: Color::White, piece_type: Type::King };

        // White bishop on e2 (between king and attacker)
        pos.position[12] = Piece { color: Color::White, piece_type: Type::Bishop };

        // Black rook on e8 (pinning the bishop)
        pos.position[60] = Piece { color: Color::Black, piece_type: Type::Rook };

        let moves = pos.legal_moves(12);

        // Bishop should have very limited moves (only along the e-file to block/stay in line)
        // Most importantly, it cannot move off the e-file
        for m in moves.iter() {
            // Any legal move for the bishop should not expose the king to check
            let mut temp_pos = Position {
                position: pos.position.clone(),
                prev_moves: pos.prev_moves.clone(),
                castling_cond: pos.castling_cond.clone(),
            };
            temp_pos.mk_move(*m);
            assert!(!temp_pos.is_in_check(Color::White), "Pinned piece moves must not expose king");
        }
    }

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

    // ==================== STALEMATE TESTS ====================

    #[test]
    fn test_basic_stalemate() {
        let mut pos = empty_board();

        // White king trapped in corner
        pos.position[0] = Piece { color: Color::White, piece_type: Type::King };  // a1

        // Black queen controlling escape squares (but not checking the king)
        pos.position[10] = Piece { color: Color::Black, piece_type: Type::Queen }; // c2

        // Black king (needed for valid position)
        pos.position[63] = Piece { color: Color::Black, piece_type: Type::King };

        assert!(!pos.is_in_check(Color::White), "King should not be in check");
        assert!(!pos.has_legal_moves(Color::White), "Should have no legal moves");
        assert!(pos.is_stalemate(Color::White), "Should be stalemate");
    }

    #[test]
    fn test_not_stalemate_when_in_check() {
        let mut pos = empty_board();

        // White king
        pos.position[0] = Piece { color: Color::White, piece_type: Type::King };

        // Black rook checking the king
        pos.position[56] = Piece { color: Color::Black, piece_type: Type::Rook };

        assert!(pos.is_in_check(Color::White), "King should be in check");
        assert!(!pos.is_stalemate(Color::White), "Not stalemate when in check (it's checkmate)");
    }

    #[test]
    fn test_not_stalemate_has_pawn_move() {
        let mut pos = empty_board();

        // White king trapped
        pos.position[0] = Piece { color: Color::White, piece_type: Type::King };

        // White pawn that can move
        pos.position[8] = Piece { color: Color::White, piece_type: Type::Pawn };

        // Black queen controlling king's squares
        pos.position[10] = Piece { color: Color::Black, piece_type: Type::Queen };

        assert!(!pos.is_stalemate(Color::White), "Not stalemate - pawn can move");
        assert!(pos.has_legal_moves(Color::White), "Should have legal pawn move");
    }

    // ==================== OTHER PIECE MOVEMENT TESTS ====================

    #[test]
    fn test_knight_moves() {
        let mut pos = empty_board();

        // Knight on e4
        pos.position[28] = Piece { color: Color::White, piece_type: Type::Knight };

        let moves = pos.legal_moves(28);

        // Knight should have 8 possible moves from center
        assert_eq!(moves.len(), 8, "Knight should have 8 moves from center");
    }

    #[test]
    fn test_bishop_moves() {
        let mut pos = empty_board();

        // Bishop on e4
        pos.position[28] = Piece { color: Color::White, piece_type: Type::Bishop };

        let moves = pos.legal_moves(28);

        // Bishop should have 13 diagonal moves from e4
        assert_eq!(moves.len(), 13, "Bishop should have 13 moves from e4");
    }

    #[test]
    fn test_rook_moves() {
        let mut pos = empty_board();

        // Rook on e4
        pos.position[28] = Piece { color: Color::White, piece_type: Type::Rook };

        let moves = pos.legal_moves(28);

        // Rook should have 14 moves from e4 (7 vertical + 7 horizontal)
        assert_eq!(moves.len(), 14, "Rook should have 14 moves from e4");
    }

    #[test]
    fn test_queen_moves() {
        let mut pos = empty_board();

        // Queen on e4
        pos.position[28] = Piece { color: Color::White, piece_type: Type::Queen };

        let moves = pos.legal_moves(28);

        // Queen should have 27 moves from e4 (combines rook + bishop)
        assert_eq!(moves.len(), 27, "Queen should have 27 moves from e4");
    }

    #[test]
    fn test_rook_blocked_by_own_piece() {
        let mut pos = empty_board();

        // Rook on a1
        pos.position[0] = Piece { color: Color::White, piece_type: Type::Rook };

        // White pawn on a3
        pos.position[16] = Piece { color: Color::White, piece_type: Type::Pawn };

        let moves = pos.legal_moves(0);

        // Should not be able to move past own piece
        assert!(!has_move(&moves, 0, 24), "Rook cannot jump over own piece");
        assert!(has_move(&moves, 0, 8), "Rook can move to square before own piece");
    }

    #[test]
    fn test_bishop_captures_opponent() {
        let mut pos = empty_board();

        // White bishop on a1
        pos.position[0] = Piece { color: Color::White, piece_type: Type::Bishop };

        // Black pawn on c3
        pos.position[18] = Piece { color: Color::Black, piece_type: Type::Pawn };

        let moves = pos.legal_moves(0);

        // Should be able to capture
        assert!(has_move(&moves, 0, 18), "Bishop should capture opponent piece");

        // Should not be able to move past it
        assert!(!has_move(&moves, 0, 27), "Bishop cannot move past captured piece");
    }

    // ==================== EN PASSANT DIRECTION BUG TEST ====================

    #[test]
    fn test_en_passant_direction_white() {
        // White pawn at d4, black pawn moves c7->c5 (double move next to white pawn)
        // White pawn at d4 should capture to c6 (toward enemy), not somewhere else
        let mut pos = empty_board();

        // Place white pawn at d4 (index 27 = row 3, col 3)
        pos.position[27] = Piece { color: Color::White, piece_type: Type::Pawn };

        // Place black pawn at c4 (index 26 = row 3, col 2) - adjacent to white pawn
        pos.position[26] = Piece { color: Color::Black, piece_type: Type::Pawn };

        // Simulate the previous move: c6->c4 (index 42->26, distance = 16)
        pos.prev_moves = vec![Move::new(42, 26, MoveType::Normal)];

        // Place kings
        pos.position[4] = Piece { color: Color::White, piece_type: Type::King };
        pos.position[60] = Piece { color: Color::Black, piece_type: Type::King };

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
        pos.position[35] = Piece { color: Color::Black, piece_type: Type::Pawn };

        // Place white pawn at e5 (index 36 = row 4, col 4) - adjacent to black pawn
        pos.position[36] = Piece { color: Color::White, piece_type: Type::Pawn };

        // Simulate the previous move: e3->e5 (index 20->36, distance = 16)
        pos.prev_moves = vec![Move::new(20, 36, MoveType::Normal)];

        // Place kings
        pos.position[4] = Piece { color: Color::White, piece_type: Type::King };
        pos.position[60] = Piece { color: Color::Black, piece_type: Type::King };

        let moves = pos.legal_moves(35);

        // Should be able to capture via en passant to e4 (index 28)
        // Black pawn at d5 (35) captures right-forward with offset -7: 35 + (-7) = 28 (e4)
        assert!(has_move(&moves, 35, 28), "Black pawn at d5 should capture RIGHT to e4 via en passant");
    }

    // ==================== PAWN EDGE SQUARE BUG TEST ====================

    #[test]
    fn test_pawn_edge_square_capture_bug() {
        // Regression test for bug where black pawn at a3 could capture at h1
        // This was caused by using offset=0 for edge squares, which created invalid captures
        let mut pos = empty_board();

        // Place black pawn at a3 (index 16 - left edge)
        pos.position[16] = Piece {
            color: Color::Black,
            piece_type: Type::Pawn,
        };

        // Place white rook at h1 (index 7 - should NOT be capturable by a3 pawn!)
        pos.position[7] = Piece {
            color: Color::White,
            piece_type: Type::Rook,
        };

        // Place white king and black king (required for legal moves)
        pos.position[4] = Piece {
            color: Color::White,
            piece_type: Type::King,
        };
        pos.position[60] = Piece {
            color: Color::Black,
            piece_type: Type::King,
        };

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
        pos.position[23] = Piece {
            color: Color::White,
            piece_type: Type::Pawn,
        };

        // Place black rook at a5 (index 32 - should NOT be capturable!)
        pos.position[32] = Piece {
            color: Color::Black,
            piece_type: Type::Rook,
        };

        // Place kings
        pos.position[4] = Piece {
            color: Color::White,
            piece_type: Type::King,
        };
        pos.position[60] = Piece {
            color: Color::Black,
            piece_type: Type::King,
        };

        let moves = pos.legal_moves(23);

        // The pawn at h3 should only be able to move to h4 (index 31)
        // It should NOT wrap around to capture on the a-file
        assert_eq!(moves.len(), 1, "White pawn at h3 should only have 1 move (h4)");
        assert!(has_move(&moves, 23, 31), "Pawn should be able to move from h3 to h4");
        assert!(!has_move(&moves, 23, 32), "BUG: Pawn should NOT wrap around board edge!");
    }

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
}
