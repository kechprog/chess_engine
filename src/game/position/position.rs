use crate::game::helpers::piece::{Color, Piece};

pub struct Position {
    pub board: [Piece; 64],
}

impl Position {
    pub fn from_fen(fen_str: &str) -> Position {
        let mut idx = 0;
        let mut board = [Piece::None; 64];

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

    /// returns a idx of all tiles on diagonals           
    pub fn diagonals(&self, idx: usize) -> Vec<u8> {
        self.board
            .iter()
            .enumerate()
            .map(|(p_idx, p)| {
                p_idx != idx
                    && ((p_idx as i64 % 8 - idx as i64 % 8).abs()
                        == (p_idx as i64 / 8 - idx as i64 / 8).abs())
            })
            .enumerate()
            .filter(|(p_idx, p)| *p)
            .map(|(p_idx, p)| p_idx as u8)
            .collect()
    }

    /// returns a idx of all tiles on lines
    pub fn lines(&self, idx: usize) -> Vec<u8> {
        self.board
            .iter()
            .enumerate()
            .map(|(p_idx, p)| p_idx != idx && ((p_idx % 8 == idx % 8) || (p_idx / 8 == idx / 8)))
            .enumerate()
            .filter(|(p_idx, p)| *p)
            .map(|(p_idx, p)| p_idx as u8)
            .collect()
    }

    /// returns a idx of all tiles on lines and diagonals, basically
    /// all queen moves no matter the possiblity of a piece being in the way
    pub fn lines_n_diagonals(&self, idx: usize) -> Vec<u8> {
        self.board
            .iter()
            .enumerate()
            .map(|(p_idx, p)| {
                p_idx != idx
                    && ((p_idx % 8 == idx % 8)
                        || (p_idx / 8 == idx / 8)
                        || ((p_idx as i64 % 8 - idx as i64 % 8).abs()
                            == (p_idx as i64 / 8 - idx as i64 / 8).abs()))
            })
            .enumerate()
            .filter(|(p_idx, p)| *p)
            .map(|(p_idx, p)| p_idx as u8)
            .collect()
    }

    // TODO: make it more consise
    /// basically all legal moves for bishop
    pub fn diagonals_till_collision(&self, idx: usize, include_friendly: bool) -> Vec<u8> {
        // ne 
        let mut moves = vec![0;0];
        let mut p_idx = idx;
        while p_idx % 8 != 7 && p_idx / 8 != 7 {
            p_idx += 9;
            match self.board[p_idx] {
                Piece::None => moves.push(p_idx as u8),
                p if p.color() != self.board[idx].color() => {
                    moves.push(p_idx as u8);
                    break
                },
                p if include_friendly && p.color() == self.board[idx].color() => {
                    moves.push(p_idx as u8);
                    break
                },
                _ => break
            }
        }

        // nw
        p_idx = idx;
        while p_idx % 8 != 0 && p_idx / 8 != 7 {
            p_idx += 7;
            match self.board[p_idx] {
                Piece::None => moves.push(p_idx as u8),
                p if p.color() != self.board[idx].color() => {
                    moves.push(p_idx as u8);
                    break
                },
                p if include_friendly && p.color() == self.board[idx].color() => {
                    moves.push(p_idx as u8);
                    break
                },
                _ => break
            }
        }

        // se
        p_idx = idx;
        while p_idx % 8 != 7 && p_idx / 8 != 0 {
            p_idx -= 7;
            match self.board[p_idx] {
                Piece::None => moves.push(p_idx as u8),
                p if p.color() != self.board[idx].color() => {
                    moves.push(p_idx as u8);
                    break
                },
                p if include_friendly && p.color() == self.board[idx].color() => {
                    moves.push(p_idx as u8);
                    break
                },
                _ => break
            }
        }

        // sw
        p_idx = idx;
        while p_idx % 8 != 0 && p_idx / 8 != 0 {
            p_idx -= 9;
            match self.board[p_idx] {
                Piece::None => moves.push(p_idx as u8),
                p if p.color() != self.board[idx].color() => {
                    moves.push(p_idx as u8);
                    break
                },
                p if include_friendly && p.color() == self.board[idx].color() => {
                    moves.push(p_idx as u8);
                    break
                },
                _ => break
            }
        }

        moves
    }

    /// basically all legal moves for rook
    pub fn lines_till_collision(&self, idx: usize, include_friendly: bool) -> Vec<u8> {
        // n
        let mut moves = vec![0;0];
        let mut p_idx = idx;
        while p_idx / 8 != 7 {
            p_idx += 8;
            match self.board[p_idx] {
                Piece::None => moves.push(p_idx as u8),
                p if p.color() != self.board[idx].color() => {
                    moves.push(p_idx as u8);
                    break
                },
                p if include_friendly && p.color() == self.board[idx].color() => {
                    moves.push(p_idx as u8);
                    break
                },
                _ => break
            }
        }

        // s
        p_idx = idx;
        while p_idx / 8 != 0 {
            p_idx -= 8;
            match self.board[p_idx] {
                Piece::None => moves.push(p_idx as u8),
                p if p.color() != self.board[idx].color() => {
                    moves.push(p_idx as u8);
                    break
                },
                p if include_friendly && p.color() == self.board[idx].color() => {
                    moves.push(p_idx as u8);
                    break
                },
                _ => break
            }
        }

        // e
        p_idx = idx;
        while p_idx % 8 != 7 {
            p_idx += 1;
            match self.board[p_idx] {
                Piece::None => moves.push(p_idx as u8),
                p if p.color() != self.board[idx].color() => {
                    moves.push(p_idx as u8);
                    break
                },
                p if include_friendly && p.color() == self.board[idx].color() => {
                    moves.push(p_idx as u8);
                    break
                },
                _ => break
            }
        }

        // w
        p_idx = idx;
        while p_idx % 8 != 0 {
            p_idx -= 1;
            match self.board[p_idx] {
                Piece::None => moves.push(p_idx as u8),
                p if p.color() != self.board[idx].color() => {
                    moves.push(p_idx as u8);
                    break
                },
                p if include_friendly || p.color() == self.board[idx].color() => {
                    moves.push(p_idx as u8);
                    break
                },
                _ => break
            }
        }

        moves
    }

    /// returns true if there is a piece which can pin
    /// between the king and the piece at p_idx
    pub fn is_pinned(&self, p_idx: usize) -> bool {
        let king_color = self.board[p_idx].color();
        let king_idx = self.board.iter().position(|p| p == match king_color {
            Color::White => &Piece::WKing,
            Color::Black => &Piece::BKing
        }).unwrap();
        
        // check if king is pinned on the diagonals
        let diagonals = self.diagonals_till_collision(p_idx, true);
        let diag_pin = diagonals.contains(&(king_idx as u8)) && {
            let e_queen = match king_color {
                Color::White => Piece::BQueen,
                Color::Black => Piece::WQueen
            };
            let e_bishop = match king_color {
                Color::White => Piece::BBishop,
                Color::Black => Piece::WBishop
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

        // check if king is pinned on the lines
        let lines = self.lines_till_collision(p_idx, true);
        let line_pin = lines.contains(&(king_idx as u8)) && {
            let e_queen = match king_color {
                Color::White => Piece::BQueen,
                Color::Black => Piece::WQueen
            };
            let e_rook = match king_color {
                Color::White => Piece::BRook,
                Color::Black => Piece::WRook
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

        diag_pin || line_pin
    }

    pub fn get_legal_moves(&self, idx: usize) -> Vec<u8> {
        let sel_piece = self.board[idx];
        if (sel_piece == Piece::None) || self.is_pinned(idx) {
            return vec![0; 0];
        }

        self.diagonals_till_collision(idx, false)
    }
}
