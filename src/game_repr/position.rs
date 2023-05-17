use super::*;

/*
 * MODULE IS RESPONSIBLE FOR 
 * GAME REPRESENTATION AND LOGIC
 */


pub struct Position {
    pub position: [Piece; 64],
    pub prev_moves: Vec<Move>,
}

impl Default for Position {
    fn default() -> Self {
        Self::from_fen(r"rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR")
    }
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
        Self { 
            position: board,
            prev_moves: vec![] 
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
                panic!("I'm to lazy to implement it")
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