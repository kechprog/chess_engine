use super::*;
use crate::game_repr::zobrist::recompute_hash;

// ==================== HELPER FUNCTIONS ====================

/// Helper function to create an empty board
pub fn empty_board() -> Position {
    use crate::game_repr::bitboards::Bitboards;
    let mut pos = Position {
        bitboards: Bitboards::empty(),
        position: [Piece::default(); 64],
        prev_moves: Vec::new(),
        castling_cond: [false; 6],
        hash: 0,
    };
    pos.hash = recompute_hash(&pos);
    pos
}

/// Helper function to place a piece
pub fn place_piece(pos: &mut Position, idx: usize, piece: Piece) {
    pos.position[idx] = piece;
    // Also update bitboards
    if piece.piece_type != Type::None {
        pos.bitboards.add_piece(piece.color, piece.piece_type, idx);
    }
    pos.hash = recompute_hash(pos);
}

/// Convert algebraic-like square ("e4") into a board index
pub fn square(square: &str) -> usize {
    assert!(square.len() == 2, "Square must be exactly 2 characters");
    let bytes = square.as_bytes();
    let file = (bytes[0] - b'a') as usize;
    let rank = (bytes[1] - b'1') as usize;
    rank * 8 + file
}

/// Find a move by origin/destination squares in the current position
pub fn find_move(pos: &Position, from: usize, to: usize) -> Move {
    pos.all_legal_moves().into_iter()
        .find(|mv| mv._from() == from && mv._to() == to)
        .unwrap_or_else(|| panic!("Move from {} to {} not found", from, to))
}

/// Find a move with an explicit move type (useful for promotions)
pub fn find_move_with_type(pos: &Position, from: usize, to: usize, move_type: MoveType) -> Move {
    pos.all_legal_moves().into_iter()
        .find(|mv| mv._from() == from && mv._to() == to && mv.move_type() == move_type)
        .unwrap_or_else(|| panic!("Move {:?} from {} to {} not found", move_type, from, to))
}

/// Helper to ensure incremental hash matches a full recomputation
pub fn assert_hash_consistency(pos: &Position) {
    assert_eq!(
        pos.hash,
        recompute_hash(pos),
        "Incremental hash diverged from recompute"
    );
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
