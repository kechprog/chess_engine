use super::piece::Piece;

pub fn position_from_fen(fen_str: &str) -> [Piece; 64] {
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

    return board;
}
