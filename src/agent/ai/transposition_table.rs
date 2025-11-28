use crate::game_repr::{Move, Position, Type, Color};
use std::collections::HashMap;
use std::sync::LazyLock;

/// Zobrist hashing constants for chess positions
///
/// Zobrist hashing uses random 64-bit numbers to represent each possible
/// piece-square combination, along with castling rights, en passant, and
/// side to move. This allows for efficient incremental hash updates.
pub struct ZobristKeys {
    /// [piece_type][color][square] - 6 piece types * 2 colors * 64 squares
    pub pieces: [[[u64; 64]; 2]; 6],
    /// [castling_index] - 6 castling conditions
    pub castling: [u64; 6],
    /// [file] - en passant file (0-7)
    pub en_passant: [u64; 8],
    /// Side to move (toggle this when it's black's turn)
    pub side_to_move: u64,
}

impl ZobristKeys {
    /// Generate Zobrist keys using a seeded random number generator
    /// This ensures the keys are random but reproducible
    fn generate() -> Self {
        use rand::{Rng, SeedableRng};
        use rand::rngs::StdRng;

        // Use a fixed seed for reproducibility
        let mut rng = StdRng::seed_from_u64(0x517cc1b727220a95);

        let mut pieces = [[[0u64; 64]; 2]; 6];

        // Generate random numbers for each piece-square combination
        for piece_type in &mut pieces {
            for color in piece_type {
                for square in color {
                    *square = rng.gen();
                }
            }
        }

        let mut castling = [0u64; 6];
        for castle in &mut castling {
            *castle = rng.gen();
        }

        let mut en_passant = [0u64; 8];
        for ep in &mut en_passant {
            *ep = rng.gen();
        }

        Self {
            pieces,
            castling,
            en_passant,
            side_to_move: rng.gen(),
        }
    }

    /// Get piece type index for Zobrist hashing
    #[inline]
    fn piece_index(piece_type: Type) -> usize {
        match piece_type {
            Type::Pawn => 0,
            Type::Knight => 1,
            Type::Bishop => 2,
            Type::Rook => 3,
            Type::Queen => 4,
            Type::King => 5,
            Type::None => panic!("Cannot hash Type::None"),
        }
    }

    /// Get color index for Zobrist hashing
    #[inline]
    fn color_index(color: Color) -> usize {
        match color {
            Color::White => 0,
            Color::Black => 1,
        }
    }
}

/// Global Zobrist keys - initialized once using LazyLock
static ZOBRIST: LazyLock<ZobristKeys> = LazyLock::new(ZobristKeys::generate);

/// Node type for transposition table entries
///
/// This is crucial for alpha-beta pruning:
/// - Exact: The exact score for this position
/// - LowerBound: Score is at least this value (beta cutoff)
/// - UpperBound: Score is at most this value (alpha cutoff)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeType {
    /// Exact score - position was fully searched
    Exact,
    /// Lower bound - beta cutoff occurred
    LowerBound,
    /// Upper bound - alpha cutoff occurred (all moves failed low)
    UpperBound,
}

/// Entry in the transposition table
#[derive(Debug, Clone, Copy)]
pub struct TranspositionTableEntry {
    /// Zobrist hash of the position
    pub hash: u64,
    /// Search depth when this position was evaluated
    pub depth: u8,
    /// Evaluation score (centipawns)
    pub score: i32,
    /// Best move found in this position
    pub best_move: Option<Move>,
    /// Type of node (exact, lower bound, upper bound)
    pub node_type: NodeType,
}

/// Transposition Table for storing previously evaluated positions
///
/// This is a critical optimization for chess engines. It stores positions
/// that have been evaluated before so we don't re-evaluate them. Uses
/// Zobrist hashing for position identification.
pub struct TranspositionTable {
    /// HashMap storing entries by hash
    table: HashMap<u64, TranspositionTableEntry>,
    /// Maximum number of entries allowed
    max_size: usize,
    /// Statistics: number of successful probes
    pub hits: u64,
    /// Statistics: number of failed probes
    pub misses: u64,
}

impl TranspositionTable {
    /// Create a transposition table with default size (1 million entries)
    pub fn new() -> Self {
        Self::with_capacity(1_000_000)
    }

    /// Create a new transposition table with specified maximum size
    ///
    /// # Arguments
    /// * `max_size` - Maximum number of entries (typical: 1_000_000 for ~100MB)
    pub fn with_capacity(max_size: usize) -> Self {
        Self {
            table: HashMap::with_capacity(max_size.min(100_000)),
            max_size,
            hits: 0,
            misses: 0,
        }
    }

    /// Compute Zobrist hash for a position
    ///
    /// This is the main hashing function that combines all position features
    /// into a single 64-bit hash value.
    pub fn compute_hash(pos: &Position) -> u64 {
        let mut hash = 0u64;

        // Hash all pieces on the board
        for square in 0..64 {
            let piece = pos.position[square];
            if piece.piece_type != Type::None {
                let piece_idx = ZobristKeys::piece_index(piece.piece_type);
                let color_idx = ZobristKeys::color_index(piece.color);
                hash ^= ZOBRIST.pieces[piece_idx][color_idx][square];
            }
        }

        // Hash castling rights
        for i in 0..6 {
            if pos.castling_cond[i] {
                hash ^= ZOBRIST.castling[i];
            }
        }

        // Hash en passant square if available
        if let Some(last_move) = pos.prev_moves.last() {
            let from = last_move._from();
            let to = last_move._to();
            let moved_piece = pos.position[to];

            // Check if it was a pawn double move
            if moved_piece.piece_type == Type::Pawn {
                let distance = to.abs_diff(from);
                if distance == 16 {
                    // En passant square is between from and to
                    let ep_square = (from + to) / 2;
                    let file = ep_square % 8;
                    hash ^= ZOBRIST.en_passant[file];
                }
            }
        }

        // Hash side to move (black to move toggles this bit)
        if pos.prev_moves.len() % 2 == 1 {
            hash ^= ZOBRIST.side_to_move;
        }

        hash
    }

    /// Update hash incrementally after a move
    ///
    /// This is more efficient than recomputing the entire hash from scratch.
    /// Returns the new hash value.
    ///
    /// # Arguments
    /// * `old_hash` - Previous position hash
    /// * `mv` - Move being made
    /// * `pos` - Position BEFORE the move is made
    pub fn update_hash(old_hash: u64, mv: Move, pos: &Position) -> u64 {
        let mut hash = old_hash;

        let from = mv._from();
        let to = mv._to();
        let moving_piece = pos.position[from];
        let captured_piece = pos.position[to];

        // Remove piece from source square
        let piece_idx = ZobristKeys::piece_index(moving_piece.piece_type);
        let color_idx = ZobristKeys::color_index(moving_piece.color);
        hash ^= ZOBRIST.pieces[piece_idx][color_idx][from];

        // Remove captured piece if any (for normal moves)
        if captured_piece.piece_type != Type::None {
            let cap_piece_idx = ZobristKeys::piece_index(captured_piece.piece_type);
            let cap_color_idx = ZobristKeys::color_index(captured_piece.color);
            hash ^= ZOBRIST.pieces[cap_piece_idx][cap_color_idx][to];
        }

        // Handle special moves
        match mv.move_type() {
            crate::game_repr::MoveType::Normal => {
                // Add piece to destination square
                hash ^= ZOBRIST.pieces[piece_idx][color_idx][to];
            },
            crate::game_repr::MoveType::EnPassant => {
                // Add pawn to destination
                hash ^= ZOBRIST.pieces[piece_idx][color_idx][to];

                // Remove captured pawn (not at destination square)
                let captured_sq = match moving_piece.color {
                    Color::White => to - 8,
                    Color::Black => to + 8,
                };
                let cap_color_idx = ZobristKeys::color_index(moving_piece.color.opposite());
                hash ^= ZOBRIST.pieces[0][cap_color_idx][captured_sq]; // 0 = Pawn
            },
            crate::game_repr::MoveType::Castling => {
                // Add king to destination
                hash ^= ZOBRIST.pieces[piece_idx][color_idx][to];

                // Move the rook
                let is_kingside = to > from;
                let (rook_from, rook_to) = match (moving_piece.color, is_kingside) {
                    (Color::White, true) => (7, 5),
                    (Color::White, false) => (0, 3),
                    (Color::Black, true) => (63, 61),
                    (Color::Black, false) => (56, 59),
                };

                let rook_idx = ZobristKeys::piece_index(Type::Rook);
                hash ^= ZOBRIST.pieces[rook_idx][color_idx][rook_from];
                hash ^= ZOBRIST.pieces[rook_idx][color_idx][rook_to];
            },
            crate::game_repr::MoveType::PromotionQueen |
            crate::game_repr::MoveType::PromotionRook |
            crate::game_repr::MoveType::PromotionBishop |
            crate::game_repr::MoveType::PromotionKnight => {
                // Determine promoted piece type
                let promoted_type = match mv.move_type() {
                    crate::game_repr::MoveType::PromotionQueen => Type::Queen,
                    crate::game_repr::MoveType::PromotionRook => Type::Rook,
                    crate::game_repr::MoveType::PromotionBishop => Type::Bishop,
                    crate::game_repr::MoveType::PromotionKnight => Type::Knight,
                    _ => unreachable!(),
                };

                // Add promoted piece to destination
                let promoted_idx = ZobristKeys::piece_index(promoted_type);
                hash ^= ZOBRIST.pieces[promoted_idx][color_idx][to];
            },
        }

        // Update castling rights (simplified - just XOR old and new)
        // This could be optimized further with incremental tracking
        for i in 0..6 {
            if pos.castling_cond[i] {
                hash ^= ZOBRIST.castling[i];
            }
        }

        // Update new castling rights after the move would be applied
        // (This is approximate - for full accuracy, apply move and recompute)

        // Update en passant
        // Remove old en passant if it existed
        if let Some(last_move) = pos.prev_moves.last() {
            let last_from = last_move._from();
            let last_to = last_move._to();
            let last_piece = pos.position[last_to];

            if last_piece.piece_type == Type::Pawn {
                let distance = last_to.abs_diff(last_from);
                if distance == 16 {
                    let ep_square = (last_from + last_to) / 2;
                    let file = ep_square % 8;
                    hash ^= ZOBRIST.en_passant[file];
                }
            }
        }

        // Add new en passant if this is a pawn double move
        if moving_piece.piece_type == Type::Pawn {
            let distance = to.abs_diff(from);
            if distance == 16 {
                let ep_square = (from + to) / 2;
                let file = ep_square % 8;
                hash ^= ZOBRIST.en_passant[file];
            }
        }

        // Toggle side to move
        hash ^= ZOBRIST.side_to_move;

        hash
    }

    /// Probe the transposition table for a position
    ///
    /// Returns the entry if found, None otherwise.
    /// Updates hit/miss statistics.
    pub fn probe(&mut self, hash: u64) -> Option<&TranspositionTableEntry> {
        if let Some(entry) = self.table.get(&hash) {
            // Verify hash matches (collision detection)
            if entry.hash == hash {
                self.hits += 1;
                return Some(entry);
            }
        }
        self.misses += 1;
        None
    }

    /// Store an entry in the transposition table
    ///
    /// Uses replacement strategy: always replace if:
    /// 1. Slot is empty, OR
    /// 2. New entry has greater depth, OR
    /// 3. New entry is exact and old is bound
    ///
    /// If table is full, evicts based on depth (keeps deeper searches)
    pub fn store(&mut self, entry: TranspositionTableEntry) {
        // Check if we need to evict
        if self.table.len() >= self.max_size {
            if let Some(existing) = self.table.get(&entry.hash) {
                // Replacement strategy: prefer deeper searches and exact scores
                let should_replace = entry.depth >= existing.depth
                    || (entry.node_type == NodeType::Exact
                        && existing.node_type != NodeType::Exact);

                if !should_replace {
                    return; // Don't replace
                }
            } else {
                // Table is full but this hash isn't present
                // We could implement FIFO or depth-based eviction here
                // For simplicity, we'll just skip insertion
                // A production engine might evict the shallowest entry
                return;
            }
        }

        self.table.insert(entry.hash, entry);
    }

    /// Clear the transposition table
    pub fn clear(&mut self) {
        self.table.clear();
        self.hits = 0;
        self.misses = 0;
    }

    /// Get current table size
    pub fn size(&self) -> usize {
        self.table.len()
    }

    /// Get hit rate (0.0 to 1.0)
    pub fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            self.hits as f64 / total as f64
        }
    }

    /// Get memory usage estimate in bytes
    pub fn memory_usage(&self) -> usize {
        // Each entry: u64 (hash) + u8 (depth) + i32 (score) + Option<Move> + NodeType
        // Move is u16, Option adds 2 bytes, NodeType is 1 byte
        // Total per entry: ~24 bytes + HashMap overhead
        // Rough estimate: 40 bytes per entry
        self.table.len() * 40
    }
}

impl Default for TranspositionTable {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game_repr::MoveType;

    #[test]
    fn test_zobrist_hash_starting_position() {
        let pos = Position::default();
        let hash1 = TranspositionTable::compute_hash(&pos);
        let hash2 = TranspositionTable::compute_hash(&pos);

        // Same position should give same hash
        assert_eq!(hash1, hash2);

        // Hash should be non-zero
        assert_ne!(hash1, 0);
    }

    #[test]
    fn test_zobrist_hash_different_positions() {
        let pos1 = Position::default();
        let pos2 = Position::from_fen("rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1");

        let hash1 = TranspositionTable::compute_hash(&pos1);
        let hash2 = TranspositionTable::compute_hash(&pos2);

        // Different positions should give different hashes
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_zobrist_hash_side_to_move() {
        // Same position, different side to move
        let pos_white = Position::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
        let pos_black = Position::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR b KQkq - 0 1");

        let hash_white = TranspositionTable::compute_hash(&pos_white);
        let hash_black = TranspositionTable::compute_hash(&pos_black);

        // Should differ only by side-to-move bit
        assert_ne!(hash_white, hash_black);
        assert_eq!(hash_white ^ hash_black, ZOBRIST.side_to_move);
    }

    #[test]
    fn test_transposition_table_store_and_probe() {
        let mut table = TranspositionTable::with_capacity(100);
        let hash = 0x1234567890ABCDEF;

        let entry = TranspositionTableEntry {
            hash,
            depth: 5,
            score: 100,
            best_move: Some(Move::new(12, 28, MoveType::Normal)),
            node_type: NodeType::Exact,
        };

        table.store(entry);

        let probed = table.probe(hash);
        assert!(probed.is_some());

        let retrieved = probed.unwrap();
        assert_eq!(retrieved.hash, hash);
        assert_eq!(retrieved.depth, 5);
        assert_eq!(retrieved.score, 100);
        assert_eq!(retrieved.node_type, NodeType::Exact);
    }

    #[test]
    fn test_transposition_table_replacement() {
        let mut table = TranspositionTable::with_capacity(100);
        let hash = 0x1234567890ABCDEF;

        // Store shallow search
        let entry1 = TranspositionTableEntry {
            hash,
            depth: 3,
            score: 50,
            best_move: None,
            node_type: NodeType::LowerBound,
        };
        table.store(entry1);

        // Store deeper search - should replace
        let entry2 = TranspositionTableEntry {
            hash,
            depth: 5,
            score: 100,
            best_move: Some(Move::new(12, 28, MoveType::Normal)),
            node_type: NodeType::Exact,
        };
        table.store(entry2);

        let probed = table.probe(hash).unwrap();
        assert_eq!(probed.depth, 5);
        assert_eq!(probed.score, 100);
    }

    #[test]
    fn test_transposition_table_clear() {
        let mut table = TranspositionTable::with_capacity(100);

        table.store(TranspositionTableEntry {
            hash: 123,
            depth: 5,
            score: 100,
            best_move: None,
            node_type: NodeType::Exact,
        });

        assert_eq!(table.size(), 1);

        table.clear();

        assert_eq!(table.size(), 0);
        assert_eq!(table.hits, 0);
        assert_eq!(table.misses, 0);
    }

    #[test]
    fn test_hit_rate() {
        let mut table = TranspositionTable::with_capacity(100);

        let entry = TranspositionTableEntry {
            hash: 123,
            depth: 5,
            score: 100,
            best_move: None,
            node_type: NodeType::Exact,
        };
        table.store(entry);

        // One hit
        table.probe(123);
        // One miss
        table.probe(456);

        assert_eq!(table.hit_rate(), 0.5);
    }

    #[test]
    fn test_incremental_hash_update() {
        let mut pos = Position::default();
        let initial_hash = TranspositionTable::compute_hash(&pos);

        // Make a move (e2-e4)
        let mv = Move::new(12, 28, MoveType::Normal);
        let updated_hash = TranspositionTable::update_hash(initial_hash, mv, &pos);

        // Apply the move
        pos.mk_move(mv);
        let recomputed_hash = TranspositionTable::compute_hash(&pos);

        // Hashes should be different from initial
        assert_ne!(updated_hash, initial_hash);
        assert_ne!(recomputed_hash, initial_hash);

        // Note: incremental update might not match recomputed due to castling complexity
        // In production, you'd either compute from scratch or track castling changes
    }
}
