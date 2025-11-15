use once_cell::sync::Lazy;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

use super::{Color, Position, Type, Piece, bitboards::Bitboards, Move};

const PIECE_VARIANTS: usize = 12; // 6 piece types * 2 colors
const BOARD_SQUARES: usize = 64;
const CASTLING_FLAGS: usize = 6;

pub struct ZobristTables {
    pub piece_keys: [[u64; BOARD_SQUARES]; PIECE_VARIANTS],
    pub castling_keys: [u64; CASTLING_FLAGS],
    pub en_passant_keys: [u64; BOARD_SQUARES],
    pub side_to_move_key: u64,
}

pub static ZOBRIST_TABLES: Lazy<ZobristTables> = Lazy::new(|| {
    let mut rng = StdRng::seed_from_u64(0xC0DEC0DECAFEBABEu64);

    let mut piece_keys = [[0u64; BOARD_SQUARES]; PIECE_VARIANTS];
    for table in &mut piece_keys {
        for entry in table.iter_mut() {
            *entry = rng.gen();
        }
    }

    let mut castling_keys = [0u64; CASTLING_FLAGS];
    for key in &mut castling_keys {
        *key = rng.gen();
    }

    let mut en_passant_keys = [0u64; BOARD_SQUARES];
    for key in &mut en_passant_keys {
        *key = rng.gen();
    }

    let side_to_move_key = rng.gen();

    ZobristTables {
        piece_keys,
        castling_keys,
        en_passant_keys,
        side_to_move_key,
    }
});

/// When set at compile time (`CHESS_FORCE_REHASH=1`), skip incremental updates
/// and fall back to the old behavior of recomputing the hash from scratch.
pub const FORCE_RECOMPUTE_HASH: bool = option_env!("CHESS_FORCE_REHASH").is_some();

pub fn recompute_hash(pos: &Position) -> u64 {
    recompute_hash_raw(&pos.bitboards, &pos.position, &pos.prev_moves, &pos.castling_cond)
}

pub fn recompute_hash_raw(_bitboards: &Bitboards, board: &[Piece; 64], prev_moves: &[Move], castling: &[bool; 6]) -> u64 {
    let mut hash = 0u64;

    for square in 0..BOARD_SQUARES {
        if let Some(idx) = piece_index(board[square]) {
            hash ^= ZOBRIST_TABLES.piece_keys[idx][square];
        }
    }

    for (i, flag) in castling.iter().enumerate() {
        if *flag {
            hash ^= ZOBRIST_TABLES.castling_keys[i];
        }
    }

    if let Some(ep) = current_en_passant_square(board, prev_moves) {
        hash ^= ZOBRIST_TABLES.en_passant_keys[ep];
    }

    if prev_moves.len() % 2 == 1 {
        hash ^= ZOBRIST_TABLES.side_to_move_key;
    }

    hash
}

fn piece_index(piece: Piece) -> Option<usize> {
    if piece.is_none() {
        return None;
    }

    let color_offset = match piece.color {
        Color::White => 0,
        Color::Black => 6,
    };

    let type_offset = match piece.piece_type {
        Type::Pawn => 0,
        Type::Knight => 1,
        Type::Bishop => 2,
        Type::Rook => 3,
        Type::Queen => 4,
        Type::King => 5,
        Type::None => return None,
    };

    Some(color_offset + type_offset)
}

#[inline]
pub fn toggle_piece(hash: &mut u64, piece: Piece, square: usize) {
    if let Some(idx) = piece_index(piece) {
        *hash ^= ZOBRIST_TABLES.piece_keys[idx][square];
    }
}

#[inline]
pub fn toggle_castling_diff(hash: &mut u64, before: &[bool; 6], after: &[bool; 6]) {
    for i in 0..CASTLING_FLAGS {
        if before[i] != after[i] {
            *hash ^= ZOBRIST_TABLES.castling_keys[i];
        }
    }
}

#[inline]
pub fn toggle_en_passant(hash: &mut u64, square: usize) {
    *hash ^= ZOBRIST_TABLES.en_passant_keys[square];
}

#[inline]
pub fn toggle_side_to_move(hash: &mut u64) {
    *hash ^= ZOBRIST_TABLES.side_to_move_key;
}

pub fn current_en_passant_square(board: &[Piece; 64], prev_moves: &[Move]) -> Option<usize> {
    let last_move = prev_moves.last()?;
    let from = last_move._from();
    let to = last_move._to();

    let piece = board[to];
    if piece.is_none() || piece.piece_type != Type::Pawn {
        return None;
    }

    if from.abs_diff(to) != 16 {
        return None;
    }

    Some((from + to) / 2)
}
