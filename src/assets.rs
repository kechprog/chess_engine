//! Chess piece asset management module
//!
//! This module embeds all chess piece PNG assets at compile time using the `include_bytes!` macro,
//! making them available for both native and WASM builds without requiring runtime file system access.
//!
//! # Assets
//!
//! The module includes:
//! - All 12 chess piece images (6 white pieces + 6 black pieces) at 128x128px resolution
//! - A circle.png for indicating legal move positions
//!
//! # Usage
//!
//! ```ignore
//! use chess_engine::game_repr::{Piece, Color, Type};
//! use chess_engine::assets::{get_asset_bytes, get_circle_asset_bytes};
//!
//! let piece = Piece {
//!     color: Color::White,
//!     piece_type: Type::King,
//! };
//!
//! let bytes = get_asset_bytes(&piece);
//! // Use bytes to load texture...
//! ```

use crate::game_repr::{Color, Piece, Type};

// White piece assets
const WHITE_PAWN: &[u8] = include_bytes!("assets/w_pawn_png_128px.png");
const WHITE_KNIGHT: &[u8] = include_bytes!("assets/w_knight_png_128px.png");
const WHITE_BISHOP: &[u8] = include_bytes!("assets/w_bishop_png_128px.png");
const WHITE_ROOK: &[u8] = include_bytes!("assets/w_rook_png_128px.png");
const WHITE_QUEEN: &[u8] = include_bytes!("assets/w_queen_png_128px.png");
const WHITE_KING: &[u8] = include_bytes!("assets/w_king_png_128px.png");

// Black piece assets
const BLACK_PAWN: &[u8] = include_bytes!("assets/b_pawn_png_128px.png");
const BLACK_KNIGHT: &[u8] = include_bytes!("assets/b_knight_png_128px.png");
const BLACK_BISHOP: &[u8] = include_bytes!("assets/b_bishop_png_128px.png");
const BLACK_ROOK: &[u8] = include_bytes!("assets/b_rook_png_128px.png");
const BLACK_QUEEN: &[u8] = include_bytes!("assets/b_queen_png_128px.png");
const BLACK_KING: &[u8] = include_bytes!("assets/b_king_png_128px.png");

// Move indicator asset
const CIRCLE: &[u8] = include_bytes!("assets/circle.png");

/// Returns the embedded PNG bytes for a given chess piece.
///
/// # Arguments
///
/// * `piece` - A reference to a `Piece` containing both color and piece type information
///
/// # Returns
///
/// A static byte slice containing the PNG image data for the requested piece.
///
/// # Panics
///
/// Panics if the piece type is `Type::None`, as there is no asset for empty squares.
///
/// # Example
///
/// ```ignore
/// use chess_engine::game_repr::{Piece, Color, Type};
/// use chess_engine::assets::get_asset_bytes;
///
/// let white_king = Piece {
///     color: Color::White,
///     piece_type: Type::King,
/// };
///
/// let bytes = get_asset_bytes(&white_king);
/// ```
pub fn get_asset_bytes(piece: &Piece) -> &'static [u8] {
    match (piece.color, piece.piece_type) {
        // White pieces
        (Color::White, Type::Pawn) => WHITE_PAWN,
        (Color::White, Type::Knight) => WHITE_KNIGHT,
        (Color::White, Type::Bishop) => WHITE_BISHOP,
        (Color::White, Type::Rook) => WHITE_ROOK,
        (Color::White, Type::Queen) => WHITE_QUEEN,
        (Color::White, Type::King) => WHITE_KING,

        // Black pieces
        (Color::Black, Type::Pawn) => BLACK_PAWN,
        (Color::Black, Type::Knight) => BLACK_KNIGHT,
        (Color::Black, Type::Bishop) => BLACK_BISHOP,
        (Color::Black, Type::Rook) => BLACK_ROOK,
        (Color::Black, Type::Queen) => BLACK_QUEEN,
        (Color::Black, Type::King) => BLACK_KING,

        // No asset for empty squares
        (_, Type::None) => panic!("Cannot get asset bytes for piece type None"),
    }
}

/// Legacy function name for backward compatibility.
/// Use `get_asset_bytes` instead.
#[deprecated(since = "0.1.0", note = "Use get_asset_bytes instead")]
pub fn get_piece_bytes(piece: Piece) -> &'static [u8] {
    get_asset_bytes(&piece)
}

/// Returns the embedded PNG bytes for the circle move indicator.
///
/// This asset is used to visually indicate legal move positions on the chess board.
///
/// # Returns
///
/// A static byte slice containing the PNG image data for the circle indicator.
///
/// # Example
///
/// ```ignore
/// use chess_engine::assets::get_circle_asset_bytes;
///
/// let circle_bytes = get_circle_asset_bytes();
/// // Use bytes to create a texture for legal move indicators
/// ```
pub fn get_circle_asset_bytes() -> &'static [u8] {
    CIRCLE
}

/// Legacy function name for backward compatibility.
/// Use `get_circle_asset_bytes` instead.
#[deprecated(since = "0.1.0", note = "Use get_circle_asset_bytes instead")]
pub fn get_dot_bytes() -> &'static [u8] {
    CIRCLE
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_white_pieces_have_assets() {
        let pieces = vec![
            Type::Pawn,
            Type::Knight,
            Type::Bishop,
            Type::Rook,
            Type::Queen,
            Type::King,
        ];

        for piece_type in pieces {
            let piece = Piece {
                color: Color::White,
                piece_type,
            };
            let bytes = get_asset_bytes(&piece);
            assert!(!bytes.is_empty(), "White {:?} asset is empty", piece_type);
            // PNG files start with the PNG signature
            assert_eq!(&bytes[0..4], b"\x89PNG", "White {:?} is not a valid PNG", piece_type);
        }
    }

    #[test]
    fn test_all_black_pieces_have_assets() {
        let pieces = vec![
            Type::Pawn,
            Type::Knight,
            Type::Bishop,
            Type::Rook,
            Type::Queen,
            Type::King,
        ];

        for piece_type in pieces {
            let piece = Piece {
                color: Color::Black,
                piece_type,
            };
            let bytes = get_asset_bytes(&piece);
            assert!(!bytes.is_empty(), "Black {:?} asset is empty", piece_type);
            assert_eq!(&bytes[0..4], b"\x89PNG", "Black {:?} is not a valid PNG", piece_type);
        }
    }

    #[test]
    fn test_circle_asset_exists() {
        let bytes = get_circle_asset_bytes();
        assert!(!bytes.is_empty(), "Circle asset is empty");
        assert_eq!(&bytes[0..4], b"\x89PNG", "Circle is not a valid PNG");
    }

    #[test]
    #[should_panic(expected = "Cannot get asset bytes for piece type None")]
    fn test_none_piece_panics() {
        let piece = Piece::none();
        get_asset_bytes(&piece);
    }

    #[test]
    fn test_backward_compatibility() {
        let piece = Piece {
            color: Color::White,
            piece_type: Type::King,
        };

        #[allow(deprecated)]
        let bytes_old = get_piece_bytes(piece);
        let bytes_new = get_asset_bytes(&piece);

        assert_eq!(bytes_old, bytes_new);

        #[allow(deprecated)]
        let dot_old = get_dot_bytes();
        let dot_new = get_circle_asset_bytes();

        assert_eq!(dot_old, dot_new);
    }
}
