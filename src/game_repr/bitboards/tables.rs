
/// Precomputed knight attack tables
/// KNIGHT_ATTACKS[square] returns a bitboard of all squares a knight can attack from that square
pub static KNIGHT_ATTACKS: [u64; 64] = generate_knight_attacks();

/// Precomputed king attack tables
/// KING_ATTACKS[square] returns a bitboard of all squares a king can attack from that square
pub static KING_ATTACKS: [u64; 64] = generate_king_attacks();

/// Precomputed pawn attack tables
/// PAWN_ATTACKS[color][square] returns a bitboard of squares a pawn can attack from that square
pub static PAWN_ATTACKS: [[u64; 64]; 2] = generate_pawn_attacks();

/// Generate knight attack table at compile time
const fn generate_knight_attacks() -> [u64; 64] {
    let mut attacks = [0u64; 64];
    let mut sq = 0;

    while sq < 64 {
        let rank = sq / 8;
        let file = sq % 8;
        let mut attack = 0u64;

        // All 8 possible knight moves (L-shaped: 2 squares in one direction, 1 in perpendicular)
        let moves: [(i8, i8); 8] = [
            (2, 1),   // NNE
            (2, -1),  // NNW
            (1, 2),   // NEE
            (1, -2),  // NWW
            (-1, 2),  // SEE
            (-1, -2), // SWW
            (-2, 1),  // SSE
            (-2, -1), // SSW
        ];

        let mut i = 0;
        while i < 8 {
            let (dr, df) = moves[i];
            let new_rank = rank as i8 + dr;
            let new_file = file as i8 + df;

            if new_rank >= 0 && new_rank < 8 && new_file >= 0 && new_file < 8 {
                let target_sq = (new_rank * 8 + new_file) as u64;
                attack |= 1u64 << target_sq;
            }

            i += 1;
        }

        attacks[sq] = attack;
        sq += 1;
    }

    attacks
}

/// Generate king attack table at compile time
const fn generate_king_attacks() -> [u64; 64] {
    let mut attacks = [0u64; 64];
    let mut sq = 0;

    while sq < 64 {
        let rank = sq / 8;
        let file = sq % 8;
        let mut attack = 0u64;

        // All 8 possible king moves (one square in any direction)
        let moves: [(i8, i8); 8] = [
            (1, 0),   // N
            (1, 1),   // NE
            (0, 1),   // E
            (-1, 1),  // SE
            (-1, 0),  // S
            (-1, -1), // SW
            (0, -1),  // W
            (1, -1),  // NW
        ];

        let mut i = 0;
        while i < 8 {
            let (dr, df) = moves[i];
            let new_rank = rank as i8 + dr;
            let new_file = file as i8 + df;

            if new_rank >= 0 && new_rank < 8 && new_file >= 0 && new_file < 8 {
                let target_sq = (new_rank * 8 + new_file) as u64;
                attack |= 1u64 << target_sq;
            }

            i += 1;
        }

        attacks[sq] = attack;
        sq += 1;
    }

    attacks
}

/// Generate pawn attack tables at compile time
/// Index 0 = White, Index 1 = Black
const fn generate_pawn_attacks() -> [[u64; 64]; 2] {
    let mut attacks = [[0u64; 64]; 2];
    let mut sq = 0;

    while sq < 64 {
        let rank = sq / 8;
        let file = sq % 8;

        // White pawn attacks (north-east and north-west)
        let mut white_attack = 0u64;
        if rank < 7 {  // Not on 8th rank
            if file > 0 {  // Can attack north-west
                white_attack |= 1u64 << (sq + 7);
            }
            if file < 7 {  // Can attack north-east
                white_attack |= 1u64 << (sq + 9);
            }
        }
        attacks[0][sq] = white_attack;

        // Black pawn attacks (south-east and south-west)
        let mut black_attack = 0u64;
        if rank > 0 {  // Not on 1st rank
            if file > 0 {  // Can attack south-west
                black_attack |= 1u64 << (sq - 9);
            }
            if file < 7 {  // Can attack south-east
                black_attack |= 1u64 << (sq - 7);
            }
        }
        attacks[1][sq] = black_attack;

        sq += 1;
    }

    attacks
}

/// Ray tables for sliding pieces
/// RAYS[direction][square] returns a bitboard of all squares in that direction from the square
pub static RAYS: [[u64; 64]; 8] = generate_rays();

// Direction indices
pub const NORTH: usize = 0;
pub const NORTH_EAST: usize = 1;
pub const EAST: usize = 2;
pub const SOUTH_EAST: usize = 3;
pub const SOUTH: usize = 4;
pub const SOUTH_WEST: usize = 5;
pub const WEST: usize = 6;
pub const NORTH_WEST: usize = 7;

/// Generate ray tables at compile time
const fn generate_rays() -> [[u64; 64]; 8] {
    let mut rays = [[0u64; 64]; 8];
    let mut sq = 0;

    while sq < 64 {
        let rank = sq / 8;
        let file = sq % 8;

        // North
        let mut r = rank + 1;
        while r < 8 {
            rays[NORTH][sq] |= 1u64 << (r * 8 + file);
            r += 1;
        }

        // North-East
        let mut r = rank + 1;
        let mut f = file + 1;
        while r < 8 && f < 8 {
            rays[NORTH_EAST][sq] |= 1u64 << (r * 8 + f);
            r += 1;
            f += 1;
        }

        // East
        let mut f = file + 1;
        while f < 8 {
            rays[EAST][sq] |= 1u64 << (rank * 8 + f);
            f += 1;
        }

        // South-East
        let mut r = rank as i8 - 1;
        let mut f = file + 1;
        while r >= 0 && f < 8 {
            rays[SOUTH_EAST][sq] |= 1u64 << (r * 8 + f as i8);
            r -= 1;
            f += 1;
        }

        // South
        let mut r = rank as i8 - 1;
        while r >= 0 {
            rays[SOUTH][sq] |= 1u64 << (r * 8 + file as i8);
            r -= 1;
        }

        // South-West
        let mut r = rank as i8 - 1;
        let mut f = file as i8 - 1;
        while r >= 0 && f >= 0 {
            rays[SOUTH_WEST][sq] |= 1u64 << (r * 8 + f);
            r -= 1;
            f -= 1;
        }

        // West
        let mut f = file as i8 - 1;
        while f >= 0 {
            rays[WEST][sq] |= 1u64 << (rank as i8 * 8 + f);
            f -= 1;
        }

        // North-West
        let mut r = rank + 1;
        let mut f = file as i8 - 1;
        while r < 8 && f >= 0 {
            rays[NORTH_WEST][sq] |= 1u64 << (r as i8 * 8 + f);
            r += 1;
            f -= 1;
        }

        sq += 1;
    }

    rays
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_knight_attacks() {
        // Test knight on d4 (square 27)
        let attacks = KNIGHT_ATTACKS[27];

        // Knight on d4 should attack: b3, b5, c2, c6, e2, e6, f3, f5
        // Square indices: 17, 33, 10, 42, 12, 44, 21, 37
        let expected = (1u64 << 17) | (1u64 << 33) | (1u64 << 10) | (1u64 << 42) |
                       (1u64 << 12) | (1u64 << 44) | (1u64 << 21) | (1u64 << 37);

        assert_eq!(attacks, expected);
    }

    #[test]
    fn test_king_attacks() {
        // Test king on e4 (square 28)
        let attacks = KING_ATTACKS[28];

        // King on e4 should attack: d3, d4, d5, e3, e5, f3, f4, f5
        // Square indices: 19, 27, 35, 20, 36, 21, 29, 37
        let expected = (1u64 << 19) | (1u64 << 27) | (1u64 << 35) | (1u64 << 20) |
                       (1u64 << 36) | (1u64 << 21) | (1u64 << 29) | (1u64 << 37);

        assert_eq!(attacks, expected);
    }

    #[test]
    fn test_pawn_attacks_white() {
        // Test white pawn on e4 (square 28)
        let attacks = PAWN_ATTACKS[0][28];

        // White pawn on e4 should attack: d5, f5
        // Square indices: 35, 37
        let expected = (1u64 << 35) | (1u64 << 37);

        assert_eq!(attacks, expected);
    }

    #[test]
    fn test_pawn_attacks_black() {
        // Test black pawn on e5 (square 36)
        let attacks = PAWN_ATTACKS[1][36];

        // Black pawn on e5 should attack: d4, f4
        // Square indices: 27, 29
        let expected = (1u64 << 27) | (1u64 << 29);

        assert_eq!(attacks, expected);
    }

    #[test]
    fn test_rays_north() {
        // Test north ray from a1 (square 0)
        let ray = RAYS[NORTH][0];

        // Should include a2, a3, a4, a5, a6, a7, a8
        let expected = (1u64 << 8) | (1u64 << 16) | (1u64 << 24) | (1u64 << 32) |
                       (1u64 << 40) | (1u64 << 48) | (1u64 << 56);

        assert_eq!(ray, expected);
    }

    #[test]
    fn test_rays_diagonal() {
        // Test north-east ray from a1 (square 0)
        let ray = RAYS[NORTH_EAST][0];

        // Should include b2, c3, d4, e5, f6, g7, h8
        let expected = (1u64 << 9) | (1u64 << 18) | (1u64 << 27) | (1u64 << 36) |
                       (1u64 << 45) | (1u64 << 54) | (1u64 << 63);

        assert_eq!(ray, expected);
    }
}
