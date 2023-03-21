use glium::{Texture2d, Display};
use std::{error::Error, rc::Rc};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Piece {
    None = 0,

    /*--- WHITE ---*/
    WPawn = 1,
    WKnight = 2,
    WBishop = 3,
    WRook = 4,
    WQueen = 5,
    WKing = 6,

    /*--- BLACK ---*/
    BPawn = 7,
    BKnight = 8,
    BBishop = 9,
    BRook = 10,
    BQueen = 11,
    BKing = 12,
}
impl Piece {
    pub fn from_char(c: char) -> Self {
        match c {
            'p' => Self::BPawn,
            'n' => Self::BKnight,
            'b' => Self::BBishop,
            'r' => Self::BRook,
            'q' => Self::BQueen,
            'k' => Self::BKing,
            'P' => Self::WPawn,
            'N' => Self::WKnight,
            'B' => Self::WBishop,
            'R' => Self::WRook,
            'Q' => Self::WQueen,
            'K' => Self::WKing,
            '1'..='8' => Self::None,
            _ => panic!("Invalid character, unable to transfrom into piece"),
        }
    }

    pub fn as_char(&self) -> char {
        match self {
            Self::WPawn => 'P',
            Self::WKnight => 'N',
            Self::WBishop => 'B',
            Self::WRook => 'R',
            Self::WQueen => 'Q',
            Self::WKing => 'K',
            Self::BPawn => 'p',
            Self::BKnight => 'n',
            Self::BBishop => 'b',
            Self::BRook => 'r',
            Self::BQueen => 'q',
            Self::BKing => 'k',
            Self::None => '_',
        }
    }

    pub fn get_texture(&self, display: &Display) -> Texture2d {
        let prefix = match self {
            Piece::WPawn | Piece::WKnight | Piece::WBishop | Piece::WRook | Piece::WQueen | Piece::WKing => "w_",
            Piece::BPawn | Piece::BKnight | Piece::BBishop | Piece::BRook | Piece::BQueen | Piece::BKing => "b_",
            _ => unreachable!("this should never happen"),
        };
        let name = match self {
            Piece::WPawn   | Piece::BPawn   => "pawn",
            Piece::WKnight | Piece::BKnight => "knight",
            Piece::WBishop | Piece::BBishop => "bishop",
            Piece::WRook   | Piece::BRook   => "rook",
            Piece::WQueen  | Piece::BQueen  => "queen",
            Piece::WKing   | Piece::BKing   => "king",
            _ => unreachable!("this should never happen"),
        };

        let img = image::open(format!("src/assets/{}{}_png_128px.png", prefix, name)).expect(format!("check ur hard drive: {}", self.as_char()).as_str())
            .to_rgba8();
        let img_dimensions = img.dimensions();
        glium::texture::Texture2d::new(
            display,
            glium::texture::RawImage2d::from_raw_rgba(img.into_raw(), img_dimensions),
        ).expect("pc is burning")
    }
}