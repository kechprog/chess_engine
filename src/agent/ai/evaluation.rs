// Position evaluation function
// Returns score in centipawns (positive = good for side to move)

use crate::game_repr::{Position, Color, Type};
use super::piece_square_tables::get_pst_value;

// Material values in centipawns
const PAWN_VALUE: i32 = 100;
const KNIGHT_VALUE: i32 = 300;
const BISHOP_VALUE: i32 = 320;
const ROOK_VALUE: i32 = 500;
const QUEEN_VALUE: i32 = 900;

// Phase values for game phase calculation (opening=256, endgame=0)
const PAWN_PHASE: i32 = 0;
const KNIGHT_PHASE: i32 = 1;
const BISHOP_PHASE: i32 = 1;
const ROOK_PHASE: i32 = 2;
const QUEEN_PHASE: i32 = 4;
const TOTAL_PHASE: i32 = PAWN_PHASE * 16 + KNIGHT_PHASE * 4 + BISHOP_PHASE * 4 + ROOK_PHASE * 4 + QUEEN_PHASE * 2;

/// Tapered evaluation score with middlegame and endgame components
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TaperedScore {
    pub mg: i32, // Middlegame score
    pub eg: i32, // Endgame score
}

impl TaperedScore {
    #[allow(dead_code)]
    pub fn new(mg: i32, eg: i32) -> Self {
        Self { mg, eg }
    }

    /// Interpolate between middlegame and endgame scores based on game phase
    /// phase: 0 (endgame) to 256 (opening)
    pub fn interpolate(&self, phase: i32) -> i32 {
        ((self.mg * phase) + (self.eg * (256 - phase))) / 256
    }

    /// Add another tapered score
    pub fn add(&mut self, other: TaperedScore) {
        self.mg += other.mg;
        self.eg += other.eg;
    }

    /// Subtract another tapered score
    pub fn sub(&mut self, other: TaperedScore) {
        self.mg -= other.mg;
        self.eg -= other.eg;
    }
}

// Evaluation weights (tapered: mg/eg)
const DOUBLED_PAWN_PENALTY: TaperedScore = TaperedScore { mg: 15, eg: 20 };
const ISOLATED_PAWN_PENALTY: TaperedScore = TaperedScore { mg: 20, eg: 25 };
const PASSED_PAWN_BONUS: TaperedScore = TaperedScore { mg: 40, eg: 70 };
const PAWN_SHIELD_BONUS: TaperedScore = TaperedScore { mg: 15, eg: 5 };

// Mobility bonuses per move (mg/eg)
const KNIGHT_MOBILITY: TaperedScore = TaperedScore { mg: 4, eg: 4 };
const BISHOP_MOBILITY: TaperedScore = TaperedScore { mg: 5, eg: 5 };
const ROOK_MOBILITY: TaperedScore = TaperedScore { mg: 2, eg: 4 };
const QUEEN_MOBILITY: TaperedScore = TaperedScore { mg: 1, eg: 2 };
const KING_MOBILITY: TaperedScore = TaperedScore { mg: 0, eg: 3 };

// Piece coordination bonuses
const BISHOP_PAIR_BONUS: TaperedScore = TaperedScore { mg: 40, eg: 50 };
const ROOK_ON_OPEN_FILE: TaperedScore = TaperedScore { mg: 25, eg: 25 };
const ROOK_ON_SEMI_OPEN_FILE: TaperedScore = TaperedScore { mg: 12, eg: 12 };
const ROOK_ON_SEVENTH: TaperedScore = TaperedScore { mg: 18, eg: 25 };
const CONNECTED_ROOKS: TaperedScore = TaperedScore { mg: 15, eg: 15 };

/// Get material value for a piece type
fn piece_value(piece_type: Type) -> i32 {
    match piece_type {
        Type::Pawn => PAWN_VALUE,
        Type::Knight => KNIGHT_VALUE,
        Type::Bishop => BISHOP_VALUE,
        Type::Rook => ROOK_VALUE,
        Type::Queen => QUEEN_VALUE,
        Type::King => 0, // King has no material value
        Type::None => 0,
    }
}

/// Calculate game phase based on remaining pieces
/// Returns value from 0 (endgame) to 256 (opening)
/// Based on piece phase values: Pawn=0, Knight=1, Bishop=1, Rook=2, Queen=4
fn calculate_game_phase(pos: &Position) -> i32 {
    let mut phase = 0;

    for square in 0..64 {
        let piece = pos.position[square];
        if piece.is_none() {
            continue;
        }

        phase += match piece.piece_type {
            Type::Pawn => PAWN_PHASE,
            Type::Knight => KNIGHT_PHASE,
            Type::Bishop => BISHOP_PHASE,
            Type::Rook => ROOK_PHASE,
            Type::Queen => QUEEN_PHASE,
            Type::King => 0,
            Type::None => 0,
        };
    }

    // Scale to 0-256 range (256 = opening, 0 = endgame)
    // TOTAL_PHASE is the phase value of starting position
    phase = (phase * 256 + (TOTAL_PHASE / 2)) / TOTAL_PHASE;
    phase.clamp(0, 256)
}

/// Determine if position is in endgame phase (for backward compatibility)
#[allow(dead_code)]
fn is_endgame(pos: &Position) -> bool {
    calculate_game_phase(pos) < 128
}

/// Evaluate material balance and piece-square tables
fn evaluate_material_and_position(pos: &Position, is_endgame: bool) -> i32 {
    let mut score = 0;

    for square in 0..64 {
        let piece = pos.position[square];
        if piece.is_none() {
            continue;
        }

        let material = piece_value(piece.piece_type);
        let positional = get_pst_value(
            piece.piece_type,
            square as u8,
            piece.color == Color::White,
            is_endgame,
        );

        let piece_value = material + positional;

        match piece.color {
            Color::White => score += piece_value,
            Color::Black => score -= piece_value,
        }
    }

    score
}

/// Evaluate king safety based on pawn shield
fn evaluate_king_safety(pos: &Position, color: Color) -> TaperedScore {
    // Find king position
    let king_square = pos.position.iter()
        .position(|&p| p.piece_type == Type::King && p.color == color);

    if king_square.is_none() {
        return TaperedScore::default(); // King not found (shouldn't happen in valid position)
    }

    let king_sq = king_square.unwrap() as i32;
    let king_rank = king_sq / 8;
    let king_file = king_sq % 8;

    let mut shield_bonus = TaperedScore::default();

    // Check for pawns in front of king (pawn shield)
    let pawn_ranks = match color {
        Color::White => [king_rank + 1, king_rank + 2], // Check ranks above
        Color::Black => [king_rank - 1, king_rank - 2], // Check ranks below
    };

    for &rank in &pawn_ranks {
        if !(0..8).contains(&rank) {
            continue;
        }

        for file_offset in -1..=1 {
            let file = king_file + file_offset;
            if !(0..8).contains(&file) {
                continue;
            }

            let square = (rank * 8 + file) as usize;
            let piece = pos.position[square];

            if piece.piece_type == Type::Pawn && piece.color == color {
                shield_bonus.add(PAWN_SHIELD_BONUS);
            }
        }
    }

    shield_bonus
}

/// Evaluate pawn structure (doubled, isolated, passed pawns)
fn evaluate_pawn_structure(pos: &Position, color: Color) -> TaperedScore {
    let mut score = TaperedScore::default();

    // Track pawns on each file
    let mut file_pawn_counts = [0; 8];
    let mut pawn_positions: Vec<(usize, usize)> = Vec::new(); // (square, file)

    // Count pawns on each file and record positions
    for square in 0..64 {
        let piece = pos.position[square];
        if piece.piece_type == Type::Pawn && piece.color == color {
            let file = square % 8;
            file_pawn_counts[file] += 1;
            pawn_positions.push((square, file));
        }
    }

    // Evaluate each pawn
    for (square, file) in pawn_positions {
        // Doubled pawn penalty
        if file_pawn_counts[file] > 1 {
            score.sub(DOUBLED_PAWN_PENALTY);
        }

        // Isolated pawn penalty (no friendly pawns on adjacent files)
        let has_left_neighbor = file > 0 && file_pawn_counts[file - 1] > 0;
        let has_right_neighbor = file < 7 && file_pawn_counts[file + 1] > 0;
        if !has_left_neighbor && !has_right_neighbor {
            score.sub(ISOLATED_PAWN_PENALTY);
        }

        // Passed pawn bonus (no enemy pawns blocking or attacking)
        if is_passed_pawn(pos, square, file, color) {
            score.add(PASSED_PAWN_BONUS);
        }
    }

    score
}

/// Check if a pawn is passed (no enemy pawns can stop it)
fn is_passed_pawn(pos: &Position, square: usize, file: usize, color: Color) -> bool {
    let rank = square / 8;
    let enemy_color = color.opposite();

    // Define the area to check (in front of the pawn and adjacent files)
    let check_ranks = match color {
        Color::White => (rank + 1)..8,  // Check ranks above
        Color::Black => 0..rank,        // Check ranks below
    };

    for check_rank in check_ranks {
        // Check same file and adjacent files
        for file_offset in -1..=1 {
            let check_file = file as i32 + file_offset;
            if !(0..8).contains(&check_file) {
                continue;
            }

            let check_square = check_rank * 8 + check_file as usize;
            let piece = pos.position[check_square];

            if piece.piece_type == Type::Pawn && piece.color == enemy_color {
                return false; // Enemy pawn blocks this pawn
            }
        }
    }

    true // No enemy pawns can stop this pawn
}

/// Count pseudo-legal moves for a piece at a given square (for mobility evaluation)
/// This is a simplified version that counts attacked squares without full legality checking
fn count_piece_mobility(pos: &Position, square: usize, piece_type: Type, color: Color) -> i32 {
    let from = square;
    let rank = (from / 8) as i32;
    let file = (from % 8) as i32;
    let mut move_count = 0;

    match piece_type {
        Type::Knight => {
            // Knight moves: L-shaped
            let knight_moves = [
                (rank + 2, file + 1), (rank + 2, file - 1),
                (rank - 2, file + 1), (rank - 2, file - 1),
                (rank + 1, file + 2), (rank + 1, file - 2),
                (rank - 1, file + 2), (rank - 1, file - 2),
            ];
            for (r, f) in knight_moves {
                if (0..8).contains(&r) && (0..8).contains(&f) {
                    let to = (r * 8 + f) as usize;
                    let target_piece = pos.position[to];
                    // Count if square is empty or occupied by enemy
                    if target_piece.is_none() || target_piece.color != color {
                        move_count += 1;
                    }
                }
            }
        }
        Type::Bishop => {
            // Bishop moves: diagonals
            let directions = [(1, 1), (1, -1), (-1, 1), (-1, -1)];
            for (dr, df) in directions {
                let mut r = rank + dr;
                let mut f = file + df;
                while (0..8).contains(&r) && (0..8).contains(&f) {
                    let to = (r * 8 + f) as usize;
                    let target_piece = pos.position[to];
                    if target_piece.is_none() {
                        move_count += 1;
                    } else {
                        if target_piece.color != color {
                            move_count += 1; // Can capture
                        }
                        break; // Blocked
                    }
                    r += dr;
                    f += df;
                }
            }
        }
        Type::Rook => {
            // Rook moves: ranks and files
            let directions = [(1, 0), (-1, 0), (0, 1), (0, -1)];
            for (dr, df) in directions {
                let mut r = rank + dr;
                let mut f = file + df;
                while (0..8).contains(&r) && (0..8).contains(&f) {
                    let to = (r * 8 + f) as usize;
                    let target_piece = pos.position[to];
                    if target_piece.is_none() {
                        move_count += 1;
                    } else {
                        if target_piece.color != color {
                            move_count += 1; // Can capture
                        }
                        break; // Blocked
                    }
                    r += dr;
                    f += df;
                }
            }
        }
        Type::Queen => {
            // Queen moves: combination of rook and bishop
            let directions = [
                (1, 0), (-1, 0), (0, 1), (0, -1),  // Rook directions
                (1, 1), (1, -1), (-1, 1), (-1, -1) // Bishop directions
            ];
            for (dr, df) in directions {
                let mut r = rank + dr;
                let mut f = file + df;
                while (0..8).contains(&r) && (0..8).contains(&f) {
                    let to = (r * 8 + f) as usize;
                    let target_piece = pos.position[to];
                    if target_piece.is_none() {
                        move_count += 1;
                    } else {
                        if target_piece.color != color {
                            move_count += 1; // Can capture
                        }
                        break; // Blocked
                    }
                    r += dr;
                    f += df;
                }
            }
        }
        Type::King => {
            // King moves: one square in any direction
            for dr in -1..=1 {
                for df in -1..=1 {
                    if dr == 0 && df == 0 {
                        continue;
                    }
                    let r = rank + dr;
                    let f = file + df;
                    if (0..8).contains(&r) && (0..8).contains(&f) {
                        let to = (r * 8 + f) as usize;
                        let target_piece = pos.position[to];
                        if target_piece.is_none() || target_piece.color != color {
                            move_count += 1;
                        }
                    }
                }
            }
        }
        _ => {} // Pawns not evaluated for mobility (handled by PST)
    }

    move_count
}

/// Evaluate piece mobility (count of pseudo-legal moves)
fn evaluate_mobility(pos: &Position, color: Color) -> TaperedScore {
    let mut mobility = TaperedScore::default();

    for square in 0..64 {
        let piece = pos.position[square];
        if piece.is_none() || piece.color != color {
            continue;
        }

        let move_count = count_piece_mobility(pos, square, piece.piece_type, color);

        let bonus = match piece.piece_type {
            Type::Knight => TaperedScore {
                mg: KNIGHT_MOBILITY.mg * move_count,
                eg: KNIGHT_MOBILITY.eg * move_count,
            },
            Type::Bishop => TaperedScore {
                mg: BISHOP_MOBILITY.mg * move_count,
                eg: BISHOP_MOBILITY.eg * move_count,
            },
            Type::Rook => TaperedScore {
                mg: ROOK_MOBILITY.mg * move_count,
                eg: ROOK_MOBILITY.eg * move_count,
            },
            Type::Queen => TaperedScore {
                mg: QUEEN_MOBILITY.mg * move_count,
                eg: QUEEN_MOBILITY.eg * move_count,
            },
            Type::King => TaperedScore {
                mg: KING_MOBILITY.mg * move_count,
                eg: KING_MOBILITY.eg * move_count,
            },
            _ => TaperedScore::default(),
        };

        mobility.add(bonus);
    }

    mobility
}

/// Evaluate bishop pair bonus
fn evaluate_bishop_pair(pos: &Position, color: Color) -> TaperedScore {
    let mut bishop_count = 0;

    for square in 0..64 {
        let piece = pos.position[square];
        if piece.piece_type == Type::Bishop && piece.color == color {
            bishop_count += 1;
        }
    }

    if bishop_count >= 2 {
        BISHOP_PAIR_BONUS
    } else {
        TaperedScore::default()
    }
}

/// Evaluate rook on open/semi-open files and 7th rank
fn evaluate_rook_features(pos: &Position, color: Color) -> TaperedScore {
    let mut score = TaperedScore::default();
    let mut rook_squares: Vec<usize> = Vec::new();

    // Find all rooks and collect their positions
    for square in 0..64 {
        let piece = pos.position[square];
        if piece.piece_type == Type::Rook && piece.color == color {
            rook_squares.push(square);
        }
    }

    // Track which files have pawns
    let mut own_pawns_on_file = [false; 8];
    let mut enemy_pawns_on_file = [false; 8];

    for square in 0..64 {
        let piece = pos.position[square];
        if piece.piece_type == Type::Pawn {
            let file = square % 8;
            if piece.color == color {
                own_pawns_on_file[file] = true;
            } else {
                enemy_pawns_on_file[file] = true;
            }
        }
    }

    // Evaluate each rook
    for &square in &rook_squares {
        let file = square % 8;
        let rank = square / 8;

        // Open file (no pawns of either color)
        if !own_pawns_on_file[file] && !enemy_pawns_on_file[file] {
            score.add(ROOK_ON_OPEN_FILE);
        }
        // Semi-open file (no own pawns, but enemy pawns present)
        else if !own_pawns_on_file[file] && enemy_pawns_on_file[file] {
            score.add(ROOK_ON_SEMI_OPEN_FILE);
        }

        // Rook on 7th rank (rank 6 for White, rank 1 for Black)
        let is_seventh = match color {
            Color::White => rank == 6,
            Color::Black => rank == 1,
        };
        if is_seventh {
            score.add(ROOK_ON_SEVENTH);
        }
    }

    // Connected rooks (on same rank or file with clear path)
    if rook_squares.len() >= 2 {
        for i in 0..rook_squares.len() {
            for j in (i + 1)..rook_squares.len() {
                let sq1 = rook_squares[i];
                let sq2 = rook_squares[j];
                let rank1 = sq1 / 8;
                let file1 = sq1 % 8;
                let rank2 = sq2 / 8;
                let file2 = sq2 % 8;

                // Check if on same rank or file
                if rank1 == rank2 || file1 == file2 {
                    // Check for clear path between them
                    let mut clear = true;
                    if rank1 == rank2 {
                        // Same rank, check files
                        let min_file = file1.min(file2);
                        let max_file = file1.max(file2);
                        for f in (min_file + 1)..max_file {
                            let check_sq = rank1 * 8 + f;
                            if !pos.position[check_sq].is_none() {
                                clear = false;
                                break;
                            }
                        }
                    } else {
                        // Same file, check ranks
                        let min_rank = rank1.min(rank2);
                        let max_rank = rank1.max(rank2);
                        for r in (min_rank + 1)..max_rank {
                            let check_sq = r * 8 + file1;
                            if !pos.position[check_sq].is_none() {
                                clear = false;
                                break;
                            }
                        }
                    }

                    if clear {
                        score.add(CONNECTED_ROOKS);
                        // Only count once per pair
                        break;
                    }
                }
            }
        }
    }

    score
}

/// Main evaluation function
/// Returns score in centipawns from the perspective of the side to move
/// Positive score = good for side to move
pub fn evaluate(pos: &Position, side_to_move: Color) -> i32 {
    let phase = calculate_game_phase(pos);
    let is_endgame = phase < 128;

    // Material and positional evaluation (still uses old PST, will be converted later)
    let material_score = evaluate_material_and_position(pos, is_endgame);

    // Tapered evaluation components
    let mut white_score = TaperedScore::default();
    let mut black_score = TaperedScore::default();

    // King safety
    white_score.add(evaluate_king_safety(pos, Color::White));
    black_score.add(evaluate_king_safety(pos, Color::Black));

    // Pawn structure
    white_score.add(evaluate_pawn_structure(pos, Color::White));
    black_score.add(evaluate_pawn_structure(pos, Color::Black));

    // Piece mobility
    white_score.add(evaluate_mobility(pos, Color::White));
    black_score.add(evaluate_mobility(pos, Color::Black));

    // Bishop pair bonus
    white_score.add(evaluate_bishop_pair(pos, Color::White));
    black_score.add(evaluate_bishop_pair(pos, Color::Black));

    // Rook features (open files, 7th rank, connected rooks)
    white_score.add(evaluate_rook_features(pos, Color::White));
    black_score.add(evaluate_rook_features(pos, Color::Black));

    // Interpolate tapered scores based on game phase
    let white_tapered = white_score.interpolate(phase);
    let black_tapered = black_score.interpolate(phase);

    // Combine material (non-tapered) with tapered positional features
    let total_score = material_score + white_tapered - black_tapered;

    // Return from perspective of side to move
    match side_to_move {
        Color::White => total_score,
        Color::Black => -total_score,
    }
}

/// Quick evaluation for move ordering (just material + PST)
/// Faster than full evaluation, good enough for ordering moves
#[allow(dead_code)]
pub fn quick_evaluate(pos: &Position, side_to_move: Color) -> i32 {
    let is_endgame = is_endgame(pos);
    let score = evaluate_material_and_position(pos, is_endgame);

    match side_to_move {
        Color::White => score,
        Color::Black => -score,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_starting_position_is_equal() {
        let pos = Position::default();
        let score = evaluate(&pos, Color::White);
        // Starting position should be roughly equal (within small margin)
        assert!(score.abs() < 100, "Starting position score: {}", score);
    }

    #[test]
    fn test_material_advantage() {
        // Position with White having extra queen (removed one Black knight)
        let pos = Position::from_fen("rnbqkb1r/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq -");
        let score = evaluate(&pos, Color::White);
        // White should be significantly ahead (up a knight = ~300 centipawns)
        assert!(score > 250, "White with extra knight score: {}", score);
    }

    #[test]
    fn test_perspective_matters() {
        let pos = Position::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNRQ");
        let score_white = evaluate(&pos, Color::White);
        let score_black = evaluate(&pos, Color::Black);
        // Scores should be opposite
        assert_eq!(score_white, -score_black);
    }

    #[test]
    fn test_endgame_detection() {
        // Position with only kings and pawns (endgame)
        let pos = Position::from_fen("4k3/8/8/8/8/8/4P3/4K3");
        assert!(is_endgame(&pos));

        // Starting position (not endgame)
        let pos = Position::default();
        assert!(!is_endgame(&pos));
    }

    #[test]
    fn test_doubled_pawns_penalty() {
        // Position with doubled White pawns on e-file
        let pos_doubled = Position::from_fen("4k3/8/8/4p3/4P3/4P3/8/4K3");
        let score_doubled = evaluate_pawn_structure(&pos_doubled, Color::White);

        // Position without doubled pawns
        let pos_normal = Position::from_fen("4k3/8/8/8/3P4/4P3/8/4K3");
        let score_normal = evaluate_pawn_structure(&pos_normal, Color::White);

        // Compare using interpolated values at middlegame phase
        assert!(score_doubled.interpolate(256) < score_normal.interpolate(256), "Doubled pawns should be penalized");
    }

    #[test]
    fn test_passed_pawn_bonus() {
        // White pawn on e4 with no Black pawns to stop it
        let pos = Position::from_fen("4k3/8/8/8/4P3/8/8/4K3");
        let score = evaluate_pawn_structure(&pos, Color::White);
        // Passed pawn should give positive bonus in both mg and eg
        assert!(score.mg > 0 && score.eg > 0, "Passed pawn should give bonus");
    }

    #[test]
    fn test_isolated_pawn_penalty() {
        // Isolated pawn on a-file vs pawns with neighbors
        let pos_isolated = Position::from_fen("4k3/8/8/8/P7/8/8/4K3 w - -");
        let pos_connected = Position::from_fen("4k3/8/8/8/PP6/8/8/4K3 w - -");

        let score_isolated = evaluate_pawn_structure(&pos_isolated, Color::White);
        let score_connected = evaluate_pawn_structure(&pos_connected, Color::White);

        // Compare using interpolated values
        assert!(score_isolated.interpolate(256) < score_connected.interpolate(256), "Isolated pawn should score worse than connected pawns");
    }
}
