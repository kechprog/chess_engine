use super::*;
use super::bitboards::{Bitboards, pop_lsb, bitscan_forward};
use super::bitboards::tables::*;

/*
 * MODULE IS RESPONSIBLE FOR
 * GAME REPRESENTATION AND LOGIC
 */


#[derive(Clone)]
pub struct Position {
    /// Bitboard representation for fast move generation
    pub(crate) bitboards: Bitboards,
    /// Mailbox representation for fast piece lookup (kept in sync with bitboards)
    pub position: [Piece; 64],
    pub prev_moves: Vec<Move>,
    /// 3 bits for each side
    /// TRUE - has not moved
    /// KingRook, QueenRook, King - white  |  R  |  K  |  Q  | R
    /// KingRook, QueenRook, King - black  |  R  |  Q  |  K  | R
    pub castling_cond: [bool; 6],
}

#[derive(Clone, Copy)]
pub struct UndoInfo {
    captured_piece: Piece,
    castling_cond: [bool; 6],
}

impl Default for Position {
    fn default() -> Self {
        Self::from_fen(r"rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR")
    }
}

impl Position {

    pub fn from_fen(fen_str: &str) -> Position {
        let parts: Vec<&str> = fen_str.split_whitespace().collect();

        // Parse piece placement (always present)
        let piece_placement = parts.get(0).unwrap_or(&"");
        let mut idx: usize = 56;
        let mut board = [Piece::default(); 64];

        for c in piece_placement.chars() {
            match c {
                '/' => {
                    idx = idx - 16;
                }
                '1'..='8' => {
                    idx += c.to_digit(10).unwrap() as usize;
                }
                _ => {
                    board[idx] = Piece::from_char(c);
                    idx += 1;
                }
            }
        }

        // Parse castling rights (if present)
        // castling_cond: [white_kingside_rook, white_queenside_rook, white_king, black_kingside_rook, black_queenside_rook, black_king]
        let mut castling_cond = [false; 6];
        if let Some(castling_str) = parts.get(2) {
            if *castling_str != "-" {
                for c in castling_str.chars() {
                    match c {
                        'K' => {
                            castling_cond[0] = true;  // White kingside rook
                            castling_cond[2] = true;  // White king
                        }
                        'Q' => {
                            castling_cond[1] = true;  // White queenside rook
                            castling_cond[2] = true;  // White king
                        }
                        'k' => {
                            castling_cond[3] = true;  // Black kingside rook
                            castling_cond[5] = true;  // Black king
                        }
                        'q' => {
                            castling_cond[4] = true;  // Black queenside rook
                            castling_cond[5] = true;  // Black king
                        }
                        _ => {}
                    }
                }
            }
        } else {
            // No castling rights specified, default to all true (for backward compatibility)
            castling_cond = [true; 6];
        }

        let bitboards = Bitboards::from_array(board);

        Self {
            bitboards,
            position: board,
            prev_moves: Vec::new(),
            castling_cond,
        }
    }

    pub fn mk_move(&mut self, _move: Move) {
        let from = _move._from();
        let to = _move._to();
        let moving_piece = self.position[from];
        let captured_piece = self.position[to];

        // Update castling conditions BEFORE making the move
        // If king moves, disable castling for that color
        if moving_piece.piece_type == Type::King {
            match moving_piece.color {
                Color::White => self.castling_cond[2] = false,
                Color::Black => self.castling_cond[5] = false,
            }
        }

        // If rook moves from starting position, disable castling on that side
        if moving_piece.piece_type == Type::Rook {
            match from {
                7 => self.castling_cond[0] = false,   // White kingside rook (h1)
                0 => self.castling_cond[1] = false,   // White queenside rook (a1)
                63 => self.castling_cond[3] = false,  // Black kingside rook (h8)
                56 => self.castling_cond[4] = false,  // Black queenside rook (a8)
                _ => {}
            }
        }

        // If a rook is captured on its starting square, disable castling for that rook
        if captured_piece.piece_type == Type::Rook {
            match to {
                7 => self.castling_cond[0] = false,   // White kingside rook (h1)
                0 => self.castling_cond[1] = false,   // White queenside rook (a1)
                63 => self.castling_cond[3] = false,  // Black kingside rook (h8)
                56 => self.castling_cond[4] = false,  // Black queenside rook (a8)
                _ => {}
            }
        }

        match _move.move_type(){
            MoveType::Normal => {
                // Update bitboards: remove captured piece if any
                if captured_piece.piece_type != Type::None {
                    self.bitboards.remove_piece(captured_piece.color, captured_piece.piece_type, to);
                }
                // Move piece in bitboards
                self.bitboards.move_piece(moving_piece.color, moving_piece.piece_type, from, to);

                // Update mailbox
                self.position[to] = self.position[from];
                self.position[from] = Piece::default();
            },
            MoveType::PromotionQueen => {
                // Remove pawn from bitboards
                self.bitboards.remove_piece(moving_piece.color, Type::Pawn, from);
                // Remove captured piece if any
                if captured_piece.piece_type != Type::None {
                    self.bitboards.remove_piece(captured_piece.color, captured_piece.piece_type, to);
                }
                // Add queen to bitboards
                self.bitboards.add_piece(moving_piece.color, Type::Queen, to);

                // Update mailbox
                self.position[to] = Piece{
                    piece_type: Type::Queen,
                    color: self.position[from].color
                };
                self.position[from] = Piece::default();
            },
            MoveType::PromotionRook => {
                // Remove pawn from bitboards
                self.bitboards.remove_piece(moving_piece.color, Type::Pawn, from);
                // Remove captured piece if any
                if captured_piece.piece_type != Type::None {
                    self.bitboards.remove_piece(captured_piece.color, captured_piece.piece_type, to);
                }
                // Add rook to bitboards
                self.bitboards.add_piece(moving_piece.color, Type::Rook, to);

                // Update mailbox
                self.position[to] = Piece{
                    piece_type: Type::Rook,
                    color: self.position[from].color
                };
                self.position[from] = Piece::default();
            },
            MoveType::PromotionBishop => {
                // Remove pawn from bitboards
                self.bitboards.remove_piece(moving_piece.color, Type::Pawn, from);
                // Remove captured piece if any
                if captured_piece.piece_type != Type::None {
                    self.bitboards.remove_piece(captured_piece.color, captured_piece.piece_type, to);
                }
                // Add bishop to bitboards
                self.bitboards.add_piece(moving_piece.color, Type::Bishop, to);

                // Update mailbox
                self.position[to] = Piece{
                    piece_type: Type::Bishop,
                    color: self.position[from].color
                };
                self.position[from] = Piece::default();
            },
            MoveType::PromotionKnight => {
                // Remove pawn from bitboards
                self.bitboards.remove_piece(moving_piece.color, Type::Pawn, from);
                // Remove captured piece if any
                if captured_piece.piece_type != Type::None {
                    self.bitboards.remove_piece(captured_piece.color, captured_piece.piece_type, to);
                }
                // Add knight to bitboards
                self.bitboards.add_piece(moving_piece.color, Type::Knight, to);

                // Update mailbox
                self.position[to] = Piece{
                    piece_type: Type::Knight,
                    color: self.position[from].color
                };
                self.position[from] = Piece::default();
            },
            MoveType::EnPassant => {
                let captured_pawn_sq = match moving_piece.color {
                    Color::White => to - 8,
                    Color::Black => to + 8,
                };
                let captured_pawn = self.position[captured_pawn_sq];

                // Remove captured pawn from bitboards
                self.bitboards.remove_piece(captured_pawn.color, Type::Pawn, captured_pawn_sq);
                // Move attacking pawn in bitboards
                self.bitboards.move_piece(moving_piece.color, Type::Pawn, from, to);

                // Update mailbox
                self.position[captured_pawn_sq] = Piece::default();
                self.position[to] = self.position[from];
                self.position[from] = Piece::default();
            },
            MoveType::Castling => {
                // Determine if kingside or queenside castling
                let is_kingside = to > from;

                // Move the rook based on color and side
                let (rook_from, rook_to) = match (moving_piece.color, is_kingside) {
                    (Color::White, true) => (7, 5),   // Kingside: h1 -> f1
                    (Color::White, false) => (0, 3),  // Queenside: a1 -> d1
                    (Color::Black, true) => (63, 61), // Kingside: h8 -> f8
                    (Color::Black, false) => (56, 59),// Queenside: a8 -> d8
                };

                // Move king in bitboards
                self.bitboards.move_piece(moving_piece.color, Type::King, from, to);
                // Move rook in bitboards
                self.bitboards.move_piece(moving_piece.color, Type::Rook, rook_from, rook_to);

                // Update mailbox
                self.position[to] = self.position[from];
                self.position[from] = Piece::default();
                self.position[rook_to] = self.position[rook_from];
                self.position[rook_from] = Piece::default();
            }
        }

        self.prev_moves.push(_move);
    }

    /// Detects which pieces are pinned to the king and returns pin information
    /// Returns (pinned_pieces_bitboard, pin_rays_array)
    /// pin_rays_array[square] contains a bitboard of valid squares the pinned piece can move to
    fn detect_pins(&self, king_color: Color) -> (u64, [u64; 64]) {
        let mut pinned_pieces = 0u64;
        let mut pin_rays = [0u64; 64];

        // Find the king position
        let king_bb = self.bitboards.pieces_of_type(king_color, Type::King);
        if king_bb == 0 {
            return (pinned_pieces, pin_rays);
        }
        let king_square = bitscan_forward(king_bb);

        let occupied = self.bitboards.all_occupied();

        // Check all 8 directions for potential pins
        for &direction in &[NORTH, NORTH_EAST, EAST, SOUTH_EAST, SOUTH, SOUTH_WEST, WEST, NORTH_WEST] {
            let ray = RAYS[direction][king_square];
            let blockers_on_ray = ray & occupied;

            if blockers_on_ray == 0 {
                continue;  // No pieces on this ray
            }

            // Find first two pieces on this ray from king outward
            let first_blocker_sq = if direction == NORTH || direction == NORTH_EAST || direction == NORTH_WEST || direction == EAST {
                bitscan_forward(blockers_on_ray)
            } else {
                63 - blockers_on_ray.leading_zeros() as usize
            };

            let first_piece = self.position[first_blocker_sq];

            // Only interested if first blocker is our own piece
            if first_piece.color != king_color {
                continue;
            }

            // Remove first blocker and check for second blocker
            let remaining_blockers = blockers_on_ray & !(1u64 << first_blocker_sq);
            if remaining_blockers == 0 {
                continue;  // No second piece
            }

            let second_blocker_sq = if direction == NORTH || direction == NORTH_EAST || direction == NORTH_WEST || direction == EAST {
                bitscan_forward(remaining_blockers)
            } else {
                63 - remaining_blockers.leading_zeros() as usize
            };

            let second_piece = self.position[second_blocker_sq];

            // Pin exists if second piece is enemy slider of correct type
            if second_piece.color == king_color.opposite() {
                let is_diagonal = direction == NORTH_EAST || direction == NORTH_WEST ||
                                 direction == SOUTH_EAST || direction == SOUTH_WEST;
                let is_orthogonal = direction == NORTH || direction == SOUTH ||
                                   direction == EAST || direction == WEST;

                let is_pinner = if is_diagonal {
                    second_piece.piece_type == Type::Bishop || second_piece.piece_type == Type::Queen
                } else if is_orthogonal {
                    second_piece.piece_type == Type::Rook || second_piece.piece_type == Type::Queen
                } else {
                    false
                };

                if is_pinner {
                    // Mark this piece as pinned
                    pinned_pieces |= 1u64 << first_blocker_sq;

                    // Calculate the pin ray: the pinned piece can move along the line from king to pinner
                    // This includes:
                    // 1. Squares between king and pinned piece (towards king)
                    // 2. Squares between pinned piece and pinner (towards pinner)
                    // 3. The pinner square itself (capture)

                    let ray_from_king = RAYS[direction][king_square];
                    let ray_from_pinner = RAYS[direction][second_blocker_sq];

                    // Everything on the ray from king that is NOT beyond the pinner
                    let pin_ray = ray_from_king & !(ray_from_pinner);

                    pin_rays[first_blocker_sq] = pin_ray;
                }
            }
        }

        (pinned_pieces, pin_rays)
    }

    /// Generate legal moves for a piece into a provided buffer
    /// The buffer is NOT cleared before adding moves
    pub fn legal_moves_into(&self, idx: usize, moves: &mut Vec<Move>) {
        let initial_len = moves.len();

        // Generate pseudo-legal moves into the buffer
        match self.position[idx] {
            Piece { piece_type: Type::Pawn,   ..} => self.pawn_moves_into(idx, moves),
            Piece { piece_type: Type::Rook,   ..} => self.rook_moves_into(idx, false, moves),
            Piece { piece_type: Type::Knight, ..} => self.knight_moves_into(idx, moves),
            Piece { piece_type: Type::Bishop, ..} => self.bishop_moves_into(idx, false, moves),
            Piece { piece_type: Type::Queen,  ..} => self.queen_moves_into(idx, moves),
            Piece { piece_type: Type::King,   ..} => self.king_moves_into(idx, moves),
            Piece { piece_type: Type::None,   ..} => return,
        };

        // Filter out moves that would leave the king in check
        // We iterate backwards and remove illegal moves using swap_remove for efficiency
        let mut i = moves.len();
        while i > initial_len {
            i -= 1;
            if !self.is_move_legal(moves[i]) {
                moves.swap_remove(i);
            }
        }
    }

    /// Generate legal moves for a piece (backward-compatible wrapper)
    pub fn legal_moves(&self, idx: usize) -> Vec<Move> {
        let mut moves = Vec::with_capacity(40);
        self.legal_moves_into(idx, &mut moves);
        moves
    }

    /// Checks if a square is under attack by any piece of the given color
    pub fn is_square_attacked(&self, square: usize, by_color: Color) -> bool {
        let occupied = self.bitboards.all_occupied();

        // Check for pawn attacks using reverse lookup
        // We need to find where pawns of by_color would need to be to attack this square
        // White pawns attack NE/NW (from lower squares), Black pawns attack SE/SW (from higher squares)
        // So to check if square X is attacked by white pawns, we look at squares that white pawns attack from
        // that would include X - which is the opposite color's attack pattern from X
        let opposite_color_idx = match by_color {
            Color::White => 1,  // Use black's attack pattern (from higher to lower)
            Color::Black => 0,  // Use white's attack pattern (from lower to higher)
        };
        let pawn_attacker_squares = PAWN_ATTACKS[opposite_color_idx][square];
        let enemy_pawns = self.bitboards.pieces_of_type(by_color, Type::Pawn);
        if (pawn_attacker_squares & enemy_pawns) != 0 {
            return true;
        }

        // Check for knight attacks
        let knight_attackers = KNIGHT_ATTACKS[square];
        let enemy_knights = self.bitboards.pieces_of_type(by_color, Type::Knight);
        if (knight_attackers & enemy_knights) != 0 {
            return true;
        }

        // Check for king attacks
        let king_attackers = KING_ATTACKS[square];
        let enemy_kings = self.bitboards.pieces_of_type(by_color, Type::King);
        if (king_attackers & enemy_kings) != 0 {
            return true;
        }

        // Check for sliding piece attacks (rook, bishop, queen)
        // Check diagonal attacks (bishop and queen)
        for &direction in &[NORTH_EAST, NORTH_WEST, SOUTH_EAST, SOUTH_WEST] {
            let ray = RAYS[direction][square];
            let blockers = ray & occupied;

            if blockers != 0 {
                // Find first blocker in this direction
                let blocker_sq = if direction == NORTH_EAST || direction == NORTH_WEST {
                    bitscan_forward(blockers)
                } else {
                    63 - blockers.leading_zeros() as usize
                };

                let blocker_piece = self.position[blocker_sq];
                if blocker_piece.color == by_color
                    && (blocker_piece.piece_type == Type::Bishop || blocker_piece.piece_type == Type::Queen)
                {
                    return true;
                }
            }
        }

        // Check rank/file attacks (rook and queen)
        for &direction in &[NORTH, SOUTH, EAST, WEST] {
            let ray = RAYS[direction][square];
            let blockers = ray & occupied;

            if blockers != 0 {
                // Find first blocker in this direction
                let blocker_sq = if direction == NORTH || direction == EAST {
                    bitscan_forward(blockers)
                } else {
                    63 - blockers.leading_zeros() as usize
                };

                let blocker_piece = self.position[blocker_sq];
                if blocker_piece.color == by_color
                    && (blocker_piece.piece_type == Type::Rook || blocker_piece.piece_type == Type::Queen)
                {
                    return true;
                }
            }
        }

        false
    }

    /// Checks if the king of the given color is currently in check
    pub fn is_in_check(&self, color: Color) -> bool {
        // Find the king's square using bitboards
        let king_bb = self.bitboards.pieces_of_type(color, Type::King);

        if king_bb == 0 {
            return false; // No king found (shouldn't happen in a valid position)
        }

        let king_square = bitscan_forward(king_bb);
        self.is_square_attacked(king_square, color.opposite())
    }

    /// Checks if a move is legal (doesn't leave/put the king in check)
    pub fn is_move_legal(&self, mv: Move) -> bool {
        // Create a minimal temporary position without cloning prev_moves
        let mut temp_position = Position {
            bitboards: self.bitboards,  // Copy bitboards (fast)
            position: self.position,     // Copy array (stack-allocated, fast)
            prev_moves: Vec::new(),      // Don't clone the move history
            castling_cond: self.castling_cond,  // Copy array
        };

        // Get the color of the piece being moved
        let moving_color = temp_position.position[mv._from()].color;

        // Execute the move on the temporary position
        temp_position.mk_move(mv);

        // Check if the king is in check after the move
        !temp_position.is_in_check(moving_color)
    }

    /// Checks if the given color has ANY legal moves available
    pub fn has_legal_moves(&self, color: Color) -> bool {
        // Iterate through each piece type using bitboards
        for piece_type in [Type::Pawn, Type::Knight, Type::Bishop, Type::Rook, Type::Queen, Type::King] {
            let mut pieces_bb = self.bitboards.pieces_of_type(color, piece_type);

            while pieces_bb != 0 {
                let square = pop_lsb(&mut pieces_bb);
                // Get legal moves for this piece
                let moves = self.legal_moves(square);
                // If any piece has at least one legal move, return true
                if !moves.is_empty() {
                    return true;
                }
            }
        }
        // No legal moves found
        false
    }

    /// Returns true if the given color is in checkmate
    /// (in check AND has no legal moves)
    pub fn is_checkmate(&self, color: Color) -> bool {
        self.is_in_check(color) && !self.has_legal_moves(color)
    }

    /// Returns true if the given color is in stalemate
    /// (NOT in check AND has no legal moves)
    pub fn is_stalemate(&self, color: Color) -> bool {
        !self.is_in_check(color) && !self.has_legal_moves(color)
    }

    /// Converts the current position to FEN notation
    pub fn to_fen(&self) -> String {
        let mut fen = String::new();

        // Piece placement (starting from rank 8 down to rank 1)
        for rank in (0..8).rev() {
            let mut empty_count = 0;
            for file in 0..8 {
                let idx = rank * 8 + file;
                let piece = self.position[idx];

                if piece.piece_type == Type::None {
                    empty_count += 1;
                } else {
                    if empty_count > 0 {
                        fen.push_str(&empty_count.to_string());
                        empty_count = 0;
                    }

                    let piece_char = match (piece.piece_type, piece.color) {
                        (Type::King, Color::White) => 'K',
                        (Type::Queen, Color::White) => 'Q',
                        (Type::Rook, Color::White) => 'R',
                        (Type::Bishop, Color::White) => 'B',
                        (Type::Knight, Color::White) => 'N',
                        (Type::Pawn, Color::White) => 'P',
                        (Type::King, Color::Black) => 'k',
                        (Type::Queen, Color::Black) => 'q',
                        (Type::Rook, Color::Black) => 'r',
                        (Type::Bishop, Color::Black) => 'b',
                        (Type::Knight, Color::Black) => 'n',
                        (Type::Pawn, Color::Black) => 'p',
                        (Type::None, _) => unreachable!(),
                    };
                    fen.push(piece_char);
                }
            }

            if empty_count > 0 {
                fen.push_str(&empty_count.to_string());
            }

            if rank > 0 {
                fen.push('/');
            }
        }

        // Side to move (even number of moves = white, odd = black)
        let side_to_move = if self.prev_moves.len() % 2 == 0 { "w" } else { "b" };
        fen.push_str(&format!(" {}", side_to_move));

        // Castling availability
        let mut castling = String::new();
        if self.castling_cond[2] {  // White king hasn't moved
            if self.castling_cond[0] {  // White kingside rook
                castling.push('K');
            }
            if self.castling_cond[1] {  // White queenside rook
                castling.push('Q');
            }
        }
        if self.castling_cond[5] {  // Black king hasn't moved
            if self.castling_cond[3] {  // Black kingside rook
                castling.push('k');
            }
            if self.castling_cond[4] {  // Black queenside rook
                castling.push('q');
            }
        }
        if castling.is_empty() {
            castling.push('-');
        }
        fen.push_str(&format!(" {}", castling));

        // En passant square
        let en_passant = if let Some(last_move) = self.prev_moves.last() {
            let from = last_move._from();
            let to = last_move._to();
            let moved_piece = self.position[to];

            // Check if it was a pawn double move
            if moved_piece.piece_type == Type::Pawn {
                let distance = if to > from { to - from } else { from - to };
                if distance == 16 {
                    // En passant square is between from and to
                    let ep_square = (from + to) / 2;
                    let file = (ep_square % 8) as u8;
                    let rank = (ep_square / 8) as u8;
                    let file_char = (b'a' + file) as char;
                    let rank_char = (b'1' + rank) as char;
                    format!("{}{}", file_char, rank_char)
                } else {
                    "-".to_string()
                }
            } else {
                "-".to_string()
            }
        } else {
            "-".to_string()
        };
        fen.push_str(&format!(" {}", en_passant));

        // Halfmove clock and fullmove number (simplified)
        fen.push_str(" 0 1");

        fen
    }

    /// Generate all legal moves for the current side into a provided buffer
    /// The buffer is cleared before adding moves
    pub fn all_legal_moves_into(&self, moves: &mut Vec<Move>) {
        moves.clear();

        let current_side = if self.prev_moves.len() % 2 == 0 {
            Color::White
        } else {
            Color::Black
        };

        // Detect pins once for the entire position
        let in_check = self.is_in_check(current_side);
        let (pinned_pieces, pin_rays) = self.detect_pins(current_side);

        // Iterate through each piece type using bitboards
        for piece_type in [Type::Pawn, Type::Knight, Type::Bishop, Type::Rook, Type::Queen, Type::King] {
            let mut pieces_bb = self.bitboards.pieces_of_type(current_side, piece_type);

            while pieces_bb != 0 {
                let square = pop_lsb(&mut pieces_bb);
                let initial_len = moves.len();

                // Generate pseudo-legal moves
                match piece_type {
                    Type::Pawn   => self.pawn_moves_into(square, moves),
                    Type::Rook   => self.rook_moves_into(square, false, moves),
                    Type::Knight => self.knight_moves_into(square, moves),
                    Type::Bishop => self.bishop_moves_into(square, false, moves),
                    Type::Queen  => self.queen_moves_into(square, moves),
                    Type::King   => self.king_moves_into(square, moves),
                    Type::None   => continue,
                }

                // Filter moves based on pin status and check status
                let is_pinned = (pinned_pieces & (1u64 << square)) != 0;
                let is_king = piece_type == Type::King;

                if is_king {
                    // King moves always need validation (can't rely on pins)
                    let mut i = moves.len();
                    while i > initial_len {
                        i -= 1;
                        if !self.is_move_legal(moves[i]) {
                            moves.swap_remove(i);
                        }
                    }
                } else if in_check {
                    // When in check, all moves need validation to ensure they block/capture
                    let mut i = moves.len();
                    while i > initial_len {
                        i -= 1;
                        if !self.is_move_legal(moves[i]) {
                            moves.swap_remove(i);
                        }
                    }
                } else if is_pinned {
                    // Pinned pieces: only allow moves along the pin ray
                    let pin_ray = pin_rays[square];
                    let mut i = moves.len();
                    while i > initial_len {
                        i -= 1;
                        let to = moves[i]._to();
                        if (pin_ray & (1u64 << to)) == 0 {
                            moves.swap_remove(i);
                        }
                    }
                } else if piece_type == Type::Pawn {
                    // Pawns: validate only en passant moves (can expose king on rank)
                    // Forward moves and normal captures are safe if not pinned
                    let mut i = moves.len();
                    while i > initial_len {
                        i -= 1;
                        if moves[i].move_type() == MoveType::EnPassant {
                            if !self.is_move_legal(moves[i]) {
                                moves.swap_remove(i);
                            }
                        }
                    }
                }
                // else: Not pinned, not king, not in check, not pawn - all pseudo-legal moves are legal!
            }
        }
    }

    /// Returns all legal moves for the current side to move (backward-compatible wrapper)
    pub fn all_legal_moves(&self) -> Vec<Move> {
        let mut all_moves = Vec::with_capacity(40);  // Typical position has 30-40 legal moves
        self.all_legal_moves_into(&mut all_moves);
        all_moves
    }

    /// Makes a move and returns undo information
    /// This is more efficient than cloning the position
    pub fn make_move_undoable(&mut self, mv: Move) -> UndoInfo {
        let to = mv._to() as usize;
        let from = mv._from() as usize;

        // For en passant, the captured piece is not at the 'to' square
        let captured_piece = match mv.move_type() {
            MoveType::EnPassant => {
                match self.position[from].color {
                    Color::White => self.position[to - 8],
                    Color::Black => self.position[to + 8],
                }
            },
            _ => self.position[to],
        };

        let undo = UndoInfo {
            captured_piece,
            castling_cond: self.castling_cond,
        };

        self.mk_move(mv);

        undo
    }

    /// Unmakes a move using undo information
    pub fn unmake_move(&mut self, mv: Move, undo: UndoInfo) {
        let from = mv._from() as usize;
        let to = mv._to() as usize;

        // Restore castling conditions
        self.castling_cond = undo.castling_cond;

        // Remove the move from history
        self.prev_moves.pop();

        // Reverse the move based on type
        match mv.move_type() {
            MoveType::Normal => {
                let moved_piece = self.position[to];

                // Remove piece from destination in bitboards
                self.bitboards.remove_piece(moved_piece.color, moved_piece.piece_type, to);
                // Add piece back to source in bitboards
                self.bitboards.add_piece(moved_piece.color, moved_piece.piece_type, from);
                // Restore captured piece if any
                if undo.captured_piece.piece_type != Type::None {
                    self.bitboards.add_piece(undo.captured_piece.color, undo.captured_piece.piece_type, to);
                }

                // Update mailbox
                self.position[from] = self.position[to];
                self.position[to] = undo.captured_piece;
            },
            MoveType::EnPassant => {
                let moved_piece = self.position[to];
                let captured_sq = match moved_piece.color {
                    Color::White => to - 8,
                    Color::Black => to + 8,
                };

                // Remove pawn from destination in bitboards
                self.bitboards.remove_piece(moved_piece.color, Type::Pawn, to);
                // Add pawn back to source in bitboards
                self.bitboards.add_piece(moved_piece.color, Type::Pawn, from);
                // Restore captured pawn in bitboards
                self.bitboards.add_piece(undo.captured_piece.color, Type::Pawn, captured_sq);

                // Update mailbox
                self.position[from] = self.position[to];
                self.position[to] = Piece::default();
                self.position[captured_sq] = undo.captured_piece;
            },
            MoveType::Castling => {
                let king = self.position[to];
                let is_kingside = to > from;
                let (rook_from, rook_to) = match (king.color, is_kingside) {
                    (Color::White, true) => (7, 5),
                    (Color::White, false) => (0, 3),
                    (Color::Black, true) => (63, 61),
                    (Color::Black, false) => (56, 59),
                };

                // Reverse king move in bitboards
                self.bitboards.move_piece(king.color, Type::King, to, from);
                // Reverse rook move in bitboards
                self.bitboards.move_piece(king.color, Type::Rook, rook_to, rook_from);

                // Update mailbox
                self.position[from] = self.position[to];
                self.position[to] = Piece::default();
                self.position[rook_from] = self.position[rook_to];
                self.position[rook_to] = Piece::default();
            },
            MoveType::PromotionQueen | MoveType::PromotionRook |
            MoveType::PromotionBishop | MoveType::PromotionKnight => {
                let promoted_piece = self.position[to];
                let original_color = promoted_piece.color;

                // Remove promoted piece from destination in bitboards
                self.bitboards.remove_piece(promoted_piece.color, promoted_piece.piece_type, to);
                // Add pawn back to source in bitboards
                self.bitboards.add_piece(original_color, Type::Pawn, from);
                // Restore captured piece if any
                if undo.captured_piece.piece_type != Type::None {
                    self.bitboards.add_piece(undo.captured_piece.color, undo.captured_piece.piece_type, to);
                }

                // Update mailbox
                self.position[from] = Piece {
                    piece_type: Type::Pawn,
                    color: original_color
                };
                self.position[to] = undo.captured_piece;
            },
        }
    }

    /// Perft (Performance Test) - counts nodes at a given depth
    /// Used to validate move generation correctness
    pub fn perft(&self, depth: u32) -> u64 {
        if depth == 0 {
            return 1;
        }

        // Use a reusable buffer for move generation
        let mut moves = Vec::with_capacity(128);
        self.all_legal_moves_into(&mut moves);

        // Bulk counting optimization for depth 1
        if depth == 1 {
            return moves.len() as u64;
        }

        // Bulk counting optimization for depth 2
        if depth == 2 {
            let mut count = 0;
            let mut pos = self.clone();
            let mut child_moves = Vec::with_capacity(128);

            for mv in moves {
                let undo = pos.make_move_undoable(mv);
                pos.all_legal_moves_into(&mut child_moves);
                count += child_moves.len() as u64;
                pos.unmake_move(mv, undo);
            }
            return count;
        }

        let mut nodes = 0;
        let mut pos = self.clone();  // Clone once at this level

        for mv in moves {
            let undo = pos.make_move_undoable(mv);
            nodes += pos.perft(depth - 1);
            pos.unmake_move(mv, undo);
        }

        nodes
    }

    /// Divide - shows perft count for each first-level move (debugging tool)
    pub fn divide(&self, depth: u32) -> u64 {
        let moves = self.all_legal_moves();
        let mut total = 0;
        let mut pos = self.clone();  // Clone once

        for mv in moves {
            let from = mv._from();
            let to = mv._to();

            // Convert indices to algebraic notation
            let from_file = (b'a' + (from % 8) as u8) as char;
            let from_rank = (b'1' + (from / 8) as u8) as char;
            let to_file = (b'a' + (to % 8) as u8) as char;
            let to_rank = (b'1' + (to / 8) as u8) as char;

            let undo = pos.make_move_undoable(mv);
            let count = pos.perft(depth - 1);
            pos.unmake_move(mv, undo);

            println!("{}{}{}{}: {}", from_file, from_rank, to_file, to_rank, count);
            total += count;
        }

        println!("\nTotal: {}", total);
        total
    }
}
