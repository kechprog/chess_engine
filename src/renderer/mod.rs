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
}
