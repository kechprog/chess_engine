use crate::agent::player::GameResult;
use crate::game_repr::{Color, Position};
use winit::dpi::PhysicalPosition;

pub mod wgpu_renderer;

/// Trait for rendering the chess board.
/// This abstraction allows for different rendering backends (wgpu for native/web, etc.)
pub trait Renderer {
    /// Draw the current board position
    ///
    /// # Arguments
    /// * `position` - The current board state
    /// * `selected_tile` - Optional tile index (0-63) that is currently selected
    /// * `pov` - Point of view (White or Black) - determines board orientation
    fn draw_position(&mut self, position: &Position, selected_tile: Option<u8>, pov: Color);

    /// Convert screen coordinates to a tile index
    ///
    /// # Arguments
    /// * `coords` - Physical position in pixels
    /// * `pov` - Point of view (affects coordinate mapping)
    ///
    /// # Returns
    /// * `Some(u8)` - Tile index (0-63) if click was on the board
    /// * `None` - If click was outside the board
    fn coord_to_tile(&self, coords: PhysicalPosition<f64>, pov: Color) -> Option<u8>;

    /// Handle window resize events
    ///
    /// # Arguments
    /// * `new_size` - New window dimensions in pixels
    fn resize(&mut self, new_size: (u32, u32));

    /// Draw the menu screen
    ///
    /// # Arguments
    /// * `show_coming_soon` - If true, display "Coming Soon!" overlay instead of menu buttons
    fn draw_menu(&mut self, show_coming_soon: bool);

    /// Check if a screen coordinate is within a button's bounds
    ///
    /// # Arguments
    /// * `coords` - Physical position in pixels
    /// * `button_index` - Which button to check (0 = PvP, 1 = PvAI)
    ///
    /// # Returns
    /// * `true` if the coordinate is within the button bounds
    fn is_coord_in_button(&self, coords: PhysicalPosition<f64>, button_index: usize) -> bool;

    /// Draw the game end overlay
    ///
    /// # Arguments
    /// * `position` - The current board state (to draw underneath)
    /// * `selected_tile` - Optional tile index (0-63) that is currently selected
    /// * `pov` - Point of view (White or Black) - determines board orientation
    /// * `result` - The game result to display
    fn draw_game_end(&mut self, position: &Position, selected_tile: Option<u8>, pov: Color, result: GameResult);

    /// Draw the promotion piece selection overlay
    ///
    /// Shows 4 piece options (Queen, Rook, Bishop, Knight) for pawn promotion.
    /// The overlay is drawn on top of the current board position.
    ///
    /// # Arguments
    /// * `position` - The current board state (to draw underneath)
    /// * `selected_tile` - Optional tile index (0-63) that is currently selected
    /// * `pov` - Point of view (White or Black) - determines board orientation
    /// * `promoting_color` - The color of the pawn being promoted
    fn draw_promotion_selection(&mut self, position: &Position, selected_tile: Option<u8>, pov: Color, promoting_color: Color);

    /// Check if screen coordinates are over a promotion piece option
    ///
    /// # Arguments
    /// * `coords` - Physical position in pixels
    ///
    /// # Returns
    /// * `Some(Type)` - The piece type if clicked on a piece (Queen, Rook, Bishop, or Knight)
    /// * `None` - If not over any piece option
    fn get_promotion_piece_at_coords(&self, coords: PhysicalPosition<f64>) -> Option<crate::game_repr::Type>;
}
