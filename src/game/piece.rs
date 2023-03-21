use glium::{Texture2d, Display};

#[derive(Debug, Clone, Copy, PartialEq)]
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

    pub fn get_texture(&self) -> Texture2d {
        todo!()
    }
}

fn load_ogl_texture(
    path: &str,
    display: &Display,
) -> Result<Texture2d, Box<dyn std::error::Error>> {
    let image = image::open(path)?;
    let image = image.to_rgba8();
    let image_dimensions = image.dimensions();
    Ok(glium::texture::Texture2d::new(
        display,
        glium::texture::RawImage2d::from_raw_rgba(image.into_raw(), image_dimensions),
    )?)
}