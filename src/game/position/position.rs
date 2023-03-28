use crate::game::helpers::piece::{Color, Piece, Type};

pub struct Position {
    pub board: [Piece; 64],
}

impl Position {
    pub fn from_fen(fen_str: &str) -> Position {
        let mut idx = 0;
        let mut board = [Piece::default(); 64];

        for c in fen_str.chars().filter(|x| *x != '/') {
            if c.is_digit(10) {
                idx += c.to_digit(10).unwrap() as usize;
                continue;
            }
            board[idx] = Piece::from_char(c);
            idx += 1;
        }

        board.reverse();
        Self { board }
    }

    /// returns true if there is a piece which can pin
    /// between the king and the piece at p_idx
    pub fn diagonal_pin(&self, p_idx: usize) -> bool {
        let king_color = self.board[p_idx].color();
        let king_idx = self
            .board
            .iter()
            .position(|p| {
                *p == Piece {color: king_color, piece_type: Type::King}
            })
            .unwrap();

        // check if king is pinned on the diagonals
        let diagonals = self.bishop_moves(p_idx, true);
        let diagonal_pin = diagonals.contains(&(king_idx as u8)) && {
            let e_queen = Piece { 
                color: self.board[p_idx].color.opposite() , 
                piece_type: Type::Queen 
            };
            let e_bishop = Piece { 
                color: self.board[p_idx].color.opposite(), 
                piece_type: Type::Bishop 
            };

            let can_pin = |e_piece, e_idx| {
                if e_piece != e_queen && e_piece != e_bishop {
                    return false;
                }
                (king_idx as i8 - e_idx as i8).abs() % 9 == 0
                    || (king_idx as i8 - e_idx as i8).abs() % 7 == 0
            };

            diagonals.iter().any(|&e_idx| {
                let e_piece = self.board[e_idx as usize];
                can_pin(e_piece, e_idx as usize)
            })
        };

        diagonal_pin
    }

    pub fn line_pin(&self, p_idx: usize) -> bool {
        let king_color = self.board[p_idx].color();
        let king_idx = self
            .board
            .iter()
            .position(|p| {
                *p == Piece {color: king_color, piece_type: Type::King} 
            })
            .unwrap();

        let lines = self.rook_moves(p_idx, true);
        let line_pin = lines.contains(&(king_idx as u8)) && {
            let e_queen = Piece { 
                color: self.board[p_idx].color.opposite(), 
                piece_type: Type::Queen
            };
            let e_rook = Piece { 
                color: self.board[p_idx].color.opposite(), 
                piece_type: Type::Rook 
            };

            let can_pin = |e_piece, e_idx| {
                if e_piece != e_queen && e_piece != e_rook {
                    return false;
                }
                (king_idx as i8 - e_idx as i8).abs() % 8 == 0
                    || (king_idx as i8 - e_idx as i8).abs() / 8 == 0
            };

            lines.iter().any(|&e_idx| {
                let e_piece = self.board[e_idx as usize];
                can_pin(e_piece, e_idx as usize)
            })
        };

        line_pin
    }

    pub fn legal_moves(&self, idx: usize) -> Vec<u8> {
        match self.board[idx] {
            Piece { piece_type: Type::Pawn,   ..} => self.pawn_moves(idx),
            Piece { piece_type: Type::Rook,   ..} => self.rook_moves(idx, false),
            Piece { piece_type: Type::Knight, ..} => self.knight_moves(idx),
            Piece { piece_type: Type::Bishop, ..} => self.bishop_moves(idx, false),
            Piece { piece_type: Type::Queen,  ..} => self.queen_moves(idx),
            Piece { piece_type: Type::King,   ..} => self.king_moves(idx),
            Piece { piece_type: Type::None,   ..} => vec![],
        }
    }
}
