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

    // TODO OTHER TYPES OF MOVES
    pub fn mk_move(&mut self, _move: Move) {

        match _move.move_type(){
            MoveType::Normal => {
                self.position[_move._to()] = self.position[_move._from() as usize];
                self.position[_move._from() as usize] = Piece::default();
            },
            MoveType::Promotion => {
                self.position[_move._to()] = Piece{
                    piece_type: Type::Queen, 
                    color: self.position[_move._from() as usize].color
                };
                self.position[_move._from() as usize] = Piece::default();
            },
            MoveType::EnPassant => {
                match self.position[_move._from()].color{
                    Color::White => {
                        self.position[_move._to() - 8] = Piece::default();
                        self.position[_move._to()] = self.position[_move._from() as usize];
                        self.position[_move._from() as usize] = Piece::default();
                    },
                    Color::Black => {
                        self.position[_move._to() + 8] = Piece::default();
                        self.position[_move._to()] = self.position[_move._from() as usize];
                        self.position[_move._from() as usize] = Piece::default();
                    }
                }
            },
            _ => todo!()
        }

        self.prev_moves.push(_move);
    }

    pub fn legal_moves(&self, idx: usize) -> Vec<Move> {
        match self.position[idx] {
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
