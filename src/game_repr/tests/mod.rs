use super::*;

// ==================== HELPER FUNCTIONS ====================

/// Helper function to create an empty board
pub fn empty_board() -> Position {
    use crate::game_repr::bitboards::Bitboards;
    Position {
        bitboards: Bitboards::empty(),
        position: [Piece::default(); 64],
        prev_moves: Vec::new(),
        castling_cond: [false; 6],
    }
}

/// Helper function to place a piece
pub fn place_piece(pos: &mut Position, idx: usize, piece: Piece) {
    pos.position[idx] = piece;
    // Also update bitboards
    if piece.piece_type != Type::None {
        pos.bitboards.add_piece(piece.color, piece.piece_type, idx);
    }
}

/// Helper function to check if a move exists in the move list
pub fn has_move(moves: &[Move], from: usize, to: usize) -> bool {
    moves.iter().any(|m| m._from() == from && m._to() == to)
}

/// Helper function to count moves of a specific type
pub fn count_move_type(moves: &[Move], move_type: MoveType) -> usize {
    moves.iter().filter(|m| {
        let mt = m.move_type();
        match (mt, move_type) {
            (MoveType::Normal, MoveType::Normal) => true,
            (MoveType::EnPassant, MoveType::EnPassant) => true,
            (MoveType::Castling, MoveType::Castling) => true,
            // Match any promotion type
            _ if mt.is_promotion() && move_type.is_promotion() => true,
            (a, b) if a == b => true,
            _ => false,
        }
    }).count()
}

// ==================== TEST MODULES ====================

mod king_movement;
mod pawn_movement;
mod piece_movement;
mod en_passant;
mod castling;
mod promotion;
mod check_detection;
mod checkmate;
mod stalemate;
mod regression;
mod fen_parsing;
mod perft;
