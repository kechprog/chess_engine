use super::*;

/*
 * MODULE IS RESPONSIBLE FOR 
 * GAME REPRESENTATION AND LOGIC
 */


pub struct Position {
    pub position: [Piece; 64],
    pub prev_moves: Vec<Move>,
    /// 3 bits for each side
    /// TRUE - has not moved
    /// KingRook, QueenRook, King - white  |  R  |  K  |  Q  | R
    /// KingRook, QueenRook, King - black  |  R  |  Q  |  K  | R
    pub castling_cond: [bool; 6],
}

impl Default for Position {
    fn default() -> Self {
        Self::from_fen(r"rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR")
    }
}

impl Position {

    // TODO add notaion for castling
    pub fn from_fen(fen_str: &str) -> Position {
        let mut idx: usize = 56;
        let mut board = [Piece::default(); 64];

        for c in fen_str.chars() {
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

        Self {
            position: board,
            prev_moves: Vec::new(),
            castling_cond: [true; 6],
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
                self.position[to] = self.position[from];
                self.position[from] = Piece::default();
            },
            MoveType::Promotion => {
                self.position[to] = Piece{
                    piece_type: Type::Queen,
                    color: self.position[from].color
                };
                self.position[from] = Piece::default();
            },
            MoveType::EnPassant => {
                match self.position[from].color{
                    Color::White => {
                        self.position[to - 8] = Piece::default();
                        self.position[to] = self.position[from];
                        self.position[from] = Piece::default();
                    },
                    Color::Black => {
                        self.position[to + 8] = Piece::default();
                        self.position[to] = self.position[from];
                        self.position[from] = Piece::default();
                    }
                }
            },
            MoveType::Castling => {
                // Move the king
                self.position[to] = self.position[from];
                self.position[from] = Piece::default();

                // Determine if kingside or queenside castling
                let is_kingside = to > from;

                // Move the rook based on color and side
                let (rook_from, rook_to) = match (moving_piece.color, is_kingside) {
                    (Color::White, true) => (7, 5),   // Kingside: h1 -> f1
                    (Color::White, false) => (0, 3),  // Queenside: a1 -> d1
                    (Color::Black, true) => (63, 61), // Kingside: h8 -> f8
                    (Color::Black, false) => (56, 59),// Queenside: a8 -> d8
                };

                self.position[rook_to] = self.position[rook_from];
                self.position[rook_from] = Piece::default();
            }
        }

        self.prev_moves.push(_move);
    }

    pub fn legal_moves(&self, idx: usize) -> Vec<Move> {
        let moves = match self.position[idx] {
            Piece { piece_type: Type::Pawn,   ..} => self.pawn_moves(idx),
            Piece { piece_type: Type::Rook,   ..} => self.rook_moves(idx, false),
            Piece { piece_type: Type::Knight, ..} => self.knight_moves(idx),
            Piece { piece_type: Type::Bishop, ..} => self.bishop_moves(idx, false),
            Piece { piece_type: Type::Queen,  ..} => self.queen_moves(idx),
            Piece { piece_type: Type::King,   ..} => self.king_moves(idx),
            Piece { piece_type: Type::None,   ..} => vec![],
        };

        // Filter out moves that would leave the king in check
        moves.into_iter().filter(|&m| self.is_move_legal(m)).collect()
    }

    /// Checks if a square is under attack by any piece of the given color
    pub fn is_square_attacked(&self, square: usize, by_color: Color) -> bool {
        // Check for pawn attacks
        let pawn_attacks = match by_color {
            Color::White => {
                // White pawns attack diagonally upward (from lower rank to higher)
                let mut attackers = vec![];
                // Check if there's a white pawn on the square that would attack diagonally down-left
                if square >= 9 && square % 8 != 0 {
                    attackers.push(square - 9);
                }
                // Check if there's a white pawn on the square that would attack diagonally down-right
                if square >= 7 && square % 8 != 7 {
                    attackers.push(square - 7);
                }
                attackers
            },
            Color::Black => {
                // Black pawns attack diagonally downward (from higher rank to lower)
                let mut attackers = vec![];
                // Check if there's a black pawn on the square that would attack diagonally up-left
                if square < 56 && square % 8 != 0 {
                    attackers.push(square + 7);
                }
                // Check if there's a black pawn on the square that would attack diagonally up-right
                if square < 55 && square % 8 != 7 {
                    attackers.push(square + 9);
                }
                attackers
            }
        };

        for attacker_square in pawn_attacks {
            let piece = self.position[attacker_square];
            if piece.piece_type == Type::Pawn && piece.color == by_color {
                return true;
            }
        }

        // Check for knight attacks (all 8 possible knight positions)
        let knight_offsets = [
            (2, 1), (2, -1), (-2, 1), (-2, -1),
            (1, 2), (1, -2), (-1, 2), (-1, -2)
        ];

        let sq_x = (square % 8) as i32;
        let sq_y = (square / 8) as i32;

        for (dx, dy) in knight_offsets.iter() {
            let new_x = sq_x + dx;
            let new_y = sq_y + dy;

            if new_x >= 0 && new_x < 8 && new_y >= 0 && new_y < 8 {
                let attacker_square = (new_y * 8 + new_x) as usize;
                let piece = self.position[attacker_square];
                if piece.piece_type == Type::Knight && piece.color == by_color {
                    return true;
                }
            }
        }

        // Check for king attacks (all 8 adjacent squares)
        let king_offsets = [
            (1, 0), (-1, 0), (0, 1), (0, -1),
            (1, 1), (1, -1), (-1, 1), (-1, -1)
        ];

        for (dx, dy) in king_offsets.iter() {
            let new_x = sq_x + dx;
            let new_y = sq_y + dy;

            if new_x >= 0 && new_x < 8 && new_y >= 0 && new_y < 8 {
                let attacker_square = (new_y * 8 + new_x) as usize;
                let piece = self.position[attacker_square];
                if piece.piece_type == Type::King && piece.color == by_color {
                    return true;
                }
            }
        }

        // Check for sliding piece attacks (bishop, rook, queen)
        // Check diagonal attacks (bishop and queen)
        let diagonal_dirs = [(1, 1), (1, -1), (-1, 1), (-1, -1)];
        for (dx, dy) in diagonal_dirs.iter() {
            let mut current_x = sq_x + dx;
            let mut current_y = sq_y + dy;

            while current_x >= 0 && current_x < 8 && current_y >= 0 && current_y < 8 {
                let current_square = (current_y * 8 + current_x) as usize;
                let piece = self.position[current_square];

                if piece.piece_type != Type::None {
                    if piece.color == by_color &&
                       (piece.piece_type == Type::Bishop || piece.piece_type == Type::Queen) {
                        return true;
                    }
                    break; // Blocked by any piece
                }

                current_x += dx;
                current_y += dy;
            }
        }

        // Check rank/file attacks (rook and queen)
        let orthogonal_dirs = [(1, 0), (-1, 0), (0, 1), (0, -1)];
        for (dx, dy) in orthogonal_dirs.iter() {
            let mut current_x = sq_x + dx;
            let mut current_y = sq_y + dy;

            while current_x >= 0 && current_x < 8 && current_y >= 0 && current_y < 8 {
                let current_square = (current_y * 8 + current_x) as usize;
                let piece = self.position[current_square];

                if piece.piece_type != Type::None {
                    if piece.color == by_color &&
                       (piece.piece_type == Type::Rook || piece.piece_type == Type::Queen) {
                        return true;
                    }
                    break; // Blocked by any piece
                }

                current_x += dx;
                current_y += dy;
            }
        }

        false
    }

    /// Checks if the king of the given color is currently in check
    pub fn is_in_check(&self, color: Color) -> bool {
        // Find the king
        let king_square = self.position.iter()
            .enumerate()
            .find(|(_, piece)| piece.piece_type == Type::King && piece.color == color)
            .map(|(idx, _)| idx);

        match king_square {
            Some(square) => self.is_square_attacked(square, color.opposite()),
            None => false, // No king found (shouldn't happen in a valid position)
        }
    }

    /// Checks if a move is legal (doesn't leave/put the king in check)
    pub fn is_move_legal(&self, mv: Move) -> bool {
        // Create a temporary position to test the move
        let mut temp_position = Position {
            position: self.position.clone(),
            prev_moves: self.prev_moves.clone(),
            castling_cond: self.castling_cond.clone(),
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
        // Iterate through all squares on the board
        for idx in 0..64 {
            let piece = self.position[idx];
            // Check if this square has a piece of the given color
            if piece.piece_type != Type::None && piece.color == color {
                // Get legal moves for this piece
                let moves = self.legal_moves(idx);
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
}
