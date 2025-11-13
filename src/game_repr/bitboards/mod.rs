use super::{Color, Type, piece::Piece};

pub mod tables;
pub use tables::*;

/// Bitboard representation using 12 u64 values (6 piece types × 2 colors)
/// Each bit represents presence/absence of a piece on that square (0-63)
#[derive(Clone, Copy, Debug)]
pub struct Bitboards {
    /// 12 piece-specific bitboards indexed by [color * 6 + piece_type]
    /// White: 0=Pawn, 1=Knight, 2=Bishop, 3=Rook, 4=Queen, 5=King
    /// Black: 6=Pawn, 7=Knight, 8=Bishop, 9=Rook, 10=Queen, 11=King
    pieces: [u64; 12],
}

// Indexing constants for pieces array
#[allow(dead_code)]
const WHITE_PAWNS: usize = 0;
#[allow(dead_code)]
const WHITE_KNIGHTS: usize = 1;
#[allow(dead_code)]
const WHITE_BISHOPS: usize = 2;
#[allow(dead_code)]
const WHITE_ROOKS: usize = 3;
#[allow(dead_code)]
const WHITE_QUEENS: usize = 4;
#[allow(dead_code)]
const WHITE_KINGS: usize = 5;
#[allow(dead_code)]
const BLACK_PAWNS: usize = 6;
#[allow(dead_code)]
const BLACK_KNIGHTS: usize = 7;
#[allow(dead_code)]
const BLACK_BISHOPS: usize = 8;
#[allow(dead_code)]
const BLACK_ROOKS: usize = 9;
#[allow(dead_code)]
const BLACK_QUEENS: usize = 10;
#[allow(dead_code)]
const BLACK_KINGS: usize = 11;

impl Bitboards {
    /// Create empty bitboards
    pub fn empty() -> Self {
        Self {
            pieces: [0; 12],
        }
    }

    /// Convert from 64-element piece array to bitboards
    pub fn from_array(pieces: [Piece; 64]) -> Self {
        let mut bitboards = Self::empty();

        for (idx, piece) in pieces.iter().enumerate() {
            if piece.piece_type == Type::None {
                continue;
            }

            let bb_idx = piece_type_to_index(piece.color, piece.piece_type);
            bitboards.set_bit(bb_idx, idx);
        }

        bitboards
    }

    /// Convert bitboards to 64-element piece array
    pub fn to_array(&self) -> [Piece; 64] {
        let mut pieces = [Piece::default(); 64];

        for color in [Color::White, Color::Black] {
            for piece_type in [Type::Pawn, Type::Knight, Type::Bishop,
                               Type::Rook, Type::Queen, Type::King] {
                let mut bb = self.pieces_of_type(color, piece_type);
                while bb != 0 {
                    let sq = pop_lsb(&mut bb);
                    pieces[sq] = Piece { color, piece_type };
                }
            }
        }

        pieces
    }

    /// Set a bit at the given square for the specified piece type
    #[inline]
    pub fn set_bit(&mut self, piece_idx: usize, square: usize) {
        self.pieces[piece_idx] |= 1u64 << square;
    }

    /// Clear a bit at the given square for the specified piece type
    #[inline]
    pub fn clear_bit(&mut self, piece_idx: usize, square: usize) {
        self.pieces[piece_idx] &= !(1u64 << square);
    }

    /// Test if a bit is set at the given square for the specified piece type
    #[inline]
    pub fn test_bit(&self, piece_idx: usize, square: usize) -> bool {
        (self.pieces[piece_idx] & (1u64 << square)) != 0
    }

    /// Get bitboard for a specific piece type and color
    #[inline(always)]
    pub fn pieces_of_type(&self, color: Color, piece_type: Type) -> u64 {
        let idx = piece_type_to_index(color, piece_type);
        self.pieces[idx]
    }

    /// Get bitboard for all pieces of a color
    #[inline(always)]
    pub fn occupied_by_color(&self, color: Color) -> u64 {
        let base = match color {
            Color::White => 0,
            Color::Black => 6,
        };
        self.pieces[base] | self.pieces[base + 1] | self.pieces[base + 2] |
        self.pieces[base + 3] | self.pieces[base + 4] | self.pieces[base + 5]
    }

    /// Get bitboard for all occupied squares
    #[inline(always)]
    pub fn all_occupied(&self) -> u64 {
        self.occupied_by_color(Color::White) | self.occupied_by_color(Color::Black)
    }

    /// Get the piece at a specific square (if any)
    pub fn piece_at(&self, square: usize) -> Piece {
        for color in [Color::White, Color::Black] {
            for piece_type in [Type::Pawn, Type::Knight, Type::Bishop,
                               Type::Rook, Type::Queen, Type::King] {
                if self.test_bit(piece_type_to_index(color, piece_type), square) {
                    return Piece { color, piece_type };
                }
            }
        }
        Piece::default()
    }

    /// Update a piece position (move from one square to another)
    pub fn move_piece(&mut self, color: Color, piece_type: Type, from: usize, to: usize) {
        let idx = piece_type_to_index(color, piece_type);
        self.clear_bit(idx, from);
        self.set_bit(idx, to);
    }

    /// Remove a piece from a square
    pub fn remove_piece(&mut self, color: Color, piece_type: Type, square: usize) {
        let idx = piece_type_to_index(color, piece_type);
        self.clear_bit(idx, square);
    }

    /// Add a piece to a square
    pub fn add_piece(&mut self, color: Color, piece_type: Type, square: usize) {
        let idx = piece_type_to_index(color, piece_type);
        self.set_bit(idx, square);
    }
}

/// Convert color and piece type to bitboard index
#[inline(always)]
fn piece_type_to_index(color: Color, piece_type: Type) -> usize {
    let base = match color {
        Color::White => 0,
        Color::Black => 6,
    };

    let offset = match piece_type {
        Type::Pawn => 0,
        Type::Knight => 1,
        Type::Bishop => 2,
        Type::Rook => 3,
        Type::Queen => 4,
        Type::King => 5,
        Type::None => panic!("Cannot get index for None piece type"),
    };

    base + offset
}

/// Pop the least significant bit from a bitboard and return its index
#[inline(always)]
pub fn pop_lsb(bb: &mut u64) -> usize {
    let sq = bb.trailing_zeros() as usize;
    *bb &= *bb - 1;  // Clear the LSB
    sq
}

/// Find the index of the least significant bit (without modifying the bitboard)
#[inline(always)]
pub fn bitscan_forward(bb: u64) -> usize {
    bb.trailing_zeros() as usize
}

/// Find the index of the most significant bit
#[inline]
pub fn bitscan_reverse(bb: u64) -> usize {
    63 - bb.leading_zeros() as usize
}

/// Count the number of set bits in a bitboard
#[inline]
pub fn popcount(bb: u64) -> u32 {
    bb.count_ones()
}

// File and rank masks
pub const FILE_A: u64 = 0x0101010101010101;
pub const FILE_B: u64 = 0x0202020202020202;
pub const FILE_C: u64 = 0x0404040404040404;
pub const FILE_D: u64 = 0x0808080808080808;
pub const FILE_E: u64 = 0x1010101010101010;
pub const FILE_F: u64 = 0x2020202020202020;
pub const FILE_G: u64 = 0x4040404040404040;
pub const FILE_H: u64 = 0x8080808080808080;

pub const RANK_1: u64 = 0x00000000000000FF;
pub const RANK_2: u64 = 0x000000000000FF00;
pub const RANK_3: u64 = 0x0000000000FF0000;
pub const RANK_4: u64 = 0x00000000FF000000;
pub const RANK_5: u64 = 0x000000FF00000000;
pub const RANK_6: u64 = 0x0000FF0000000000;
pub const RANK_7: u64 = 0x00FF000000000000;
pub const RANK_8: u64 = 0xFF00000000000000;

// Shift operations with edge handling
#[inline]
pub fn north(bb: u64) -> u64 {
    bb << 8
}

#[inline]
pub fn south(bb: u64) -> u64 {
    bb >> 8
}

#[inline]
pub fn east(bb: u64) -> u64 {
    (bb << 1) & !FILE_A
}

#[inline]
pub fn west(bb: u64) -> u64 {
    (bb >> 1) & !FILE_H
}

#[inline]
pub fn north_east(bb: u64) -> u64 {
    (bb << 9) & !FILE_A
}

#[inline]
pub fn north_west(bb: u64) -> u64 {
    (bb << 7) & !FILE_H
}

#[inline]
pub fn south_east(bb: u64) -> u64 {
    (bb >> 7) & !FILE_A
}

#[inline]
pub fn south_west(bb: u64) -> u64 {
    (bb >> 9) & !FILE_H
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_bitboards() {
        let bb = Bitboards::empty();
        assert_eq!(bb.all_occupied(), 0);
    }

    #[test]
    fn test_set_clear_bit() {
        let mut bb = Bitboards::empty();
        bb.set_bit(WHITE_PAWNS, 8);
        assert!(bb.test_bit(WHITE_PAWNS, 8));
        bb.clear_bit(WHITE_PAWNS, 8);
        assert!(!bb.test_bit(WHITE_PAWNS, 8));
    }

    #[test]
    fn test_occupied_by_color() {
        let mut bb = Bitboards::empty();
        bb.set_bit(WHITE_PAWNS, 8);
        bb.set_bit(WHITE_KNIGHTS, 16);
        bb.set_bit(BLACK_PAWNS, 48);

        let white_occ = bb.occupied_by_color(Color::White);
        assert_eq!(white_occ, (1u64 << 8) | (1u64 << 16));

        let black_occ = bb.occupied_by_color(Color::Black);
        assert_eq!(black_occ, 1u64 << 48);
    }

    #[test]
    fn test_pop_lsb() {
        let mut bb = 0b1010u64;
        assert_eq!(pop_lsb(&mut bb), 1);
        assert_eq!(bb, 0b1000);
        assert_eq!(pop_lsb(&mut bb), 3);
        assert_eq!(bb, 0);
    }

    #[test]
    fn test_shift_operations() {
        let bb = 1u64 << 27;  // d4

        // Test north (should go to d5)
        assert_eq!(north(bb), 1u64 << 35);

        // Test south (should go to d3)
        assert_eq!(south(bb), 1u64 << 19);

        // Test east (should go to e4)
        assert_eq!(east(bb), 1u64 << 28);

        // Test west (should go to c4)
        assert_eq!(west(bb), 1u64 << 26);
    }

    #[test]
    fn test_edge_wrapping() {
        // Test that shifts don't wrap around board edges
        let a1 = 1u64 << 0;  // a1 (left edge)
        assert_eq!(west(a1), 0);  // Can't go west from a-file

        let h1 = 1u64 << 7;  // h1 (right edge)
        assert_eq!(east(h1), 0);  // Can't go east from h-file
    }
}
