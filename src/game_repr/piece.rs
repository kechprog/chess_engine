use image::ColorType;
use std::{error::Error, rc::Rc};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Type {
    King,
    Queen,
    Rook,
    Bishop,
    Knight,
    Pawn,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Color {
    White,
    Black,
}

impl Color {
    pub fn opposite(&self) -> Self {
        match self {
            Self::White => Self::Black,
            Self::Black => Self::White,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Piece {
    pub color: Color,
    pub piece_type: Type,
}

impl Default for Piece {
    fn default() -> Self {
        Self {
            color: Color::White,
            piece_type: Type::None,
        }
    }
}

impl Piece {
    pub fn none() -> Self {
        Self {
            color: Color::White,
            piece_type: Type::None,
        }
    }

    pub fn is_none(&self) -> bool {
        self.piece_type == Type::None
    }

    pub fn from_char(c: char) -> Self {
        match c {
            'p' => Self {
                color: Color::Black,
                piece_type: Type::Pawn,
            },
            'n' => Self {
                color: Color::Black,
                piece_type: Type::Knight,
            },
            'b' => Self {
                color: Color::Black,
                piece_type: Type::Bishop,
            },
            'r' => Self {
                color: Color::Black,
                piece_type: Type::Rook,
            },
            'q' => Self {
                color: Color::Black,
                piece_type: Type::Queen,
            },
            'k' => Self {
                color: Color::Black,
                piece_type: Type::King,
            },
            'P' => Self {
                color: Color::White,
                piece_type: Type::Pawn,
            },
            'N' => Self {
                color: Color::White,
                piece_type: Type::Knight,
            },
            'B' => Self {
                color: Color::White,
                piece_type: Type::Bishop,
            },
            'R' => Self {
                color: Color::White,
                piece_type: Type::Rook,
            },
            'Q' => Self {
                color: Color::White,
                piece_type: Type::Queen,
            },
            'K' => Self {
                color: Color::White,
                piece_type: Type::King,
            },
            '1'..='8' => Self {
                color: Color::White,
                piece_type: Type::None,
            },
            _ => panic!("Invalid character, unable to transfrom into piece"),
        }
    }

    // true = white, false = black
    pub fn color(&self) -> Color {
        self.color
    }

    pub fn is(&self, color: Color) -> bool {
        match color {
            Color::White => self.color() == Color::White,
            Color::Black => self.color() == Color::Black,
        }
    }
}