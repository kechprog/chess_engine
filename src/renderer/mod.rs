use crate::agent::player::GameResult;
use crate::game_repr::{Color, Position};
use crate::agent::ai::{AIType, Difficulty};
use crate::menu::MenuState;
use crate::orchestrator::AISetupButton;
use winit::dpi::PhysicalPosition;

pub mod wgpu_renderer;

/// Actions triggered by game control buttons
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControlAction {
    /// Navigate to previous move (undo)
    Undo,
    /// Navigate to next move (redo)
    Redo,
    /// Flip the board orientation
    FlipBoard,
}

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

    // ===========================
    // New Menu System (Phase 4)
    // ===========================

    /// Draw a menu screen based on the current MenuState.
    ///
    /// This method uses the centralized layout definitions from `crate::menu::layout`
    /// for consistent button positioning between rendering and hit detection.
    ///
    /// # Arguments
    /// * `state` - The current menu state to render
    fn draw_menu_state(&mut self, state: &MenuState);

    /// Get the current window size (for Menu coordinate conversion)
    fn window_size(&self) -> (u32, u32);

    // ===========================
    // Legacy Menu Methods (to be removed)
    // ===========================

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

    /// Draw the side selection screen for PvAI mode
    ///
    /// Shows options to play as White or Black
    fn draw_side_selection(&mut self);

    /// Check if screen coordinates are over a side selection button
    ///
    /// # Arguments
    /// * `coords` - Physical position in pixels
    /// * `button_index` - Which button to check (0 = White, 1 = Black)
    ///
    /// # Returns
    /// * `true` if the coordinate is within the button bounds
    fn is_coord_in_side_button(&self, coords: PhysicalPosition<f64>, button_index: usize) -> bool;

    // ===========================
    // Game Controls Bar
    // ===========================

    /// Draw the game controls bar (undo, redo, flip board buttons)
    ///
    /// The controls bar is drawn below the chess board and provides
    /// navigation and board orientation controls.
    ///
    /// # Arguments
    /// * `can_undo` - Whether the undo button should be enabled
    /// * `can_redo` - Whether the redo button should be enabled
    fn draw_controls_bar(&mut self, can_undo: bool, can_redo: bool);

    /// Check if screen coordinates are over a control button
    ///
    /// # Arguments
    /// * `coords` - Physical position in pixels
    ///
    /// # Returns
    /// * `Some(ControlAction)` - The action for the button under the coordinates
    /// * `None` - If not over any control button
    fn get_control_action_at_coords(&self, coords: PhysicalPosition<f64>) -> Option<ControlAction>;

    // ===========================
    // AI Setup Screen (Combined)
    // ===========================

    /// Draw the combined AIvAI setup screen with both White and Black AI configuration
    ///
    /// Shows AI type selection and difficulty options for both players on a single screen.
    ///
    /// # Arguments
    /// * `ai_types` - Available AI types to choose from
    /// * `white_type_index` - Currently selected AI type index for White
    /// * `white_difficulty` - Currently selected difficulty for White
    /// * `black_type_index` - Currently selected AI type index for Black
    /// * `black_difficulty` - Currently selected difficulty for Black
    /// * `pressed_button` - Currently pressed button (if any) for visual feedback
    fn draw_ai_setup(
        &mut self,
        ai_types: &[AIType],
        white_type_index: usize,
        white_difficulty: Difficulty,
        black_type_index: usize,
        black_difficulty: Difficulty,
        pressed_button: Option<AISetupButton>,
    );

    /// Check if screen coordinates are over a White AI difficulty button
    ///
    /// # Returns
    /// * `Some(usize)` - Index of the difficulty button (0=Easy, 1=Medium, 2=Hard, 3=Expert)
    /// * `None` - If not over any White difficulty button
    fn get_white_difficulty_at_coords(&self, coords: PhysicalPosition<f64>) -> Option<usize>;

    /// Check if screen coordinates are over a Black AI difficulty button
    ///
    /// # Returns
    /// * `Some(usize)` - Index of the difficulty button (0=Easy, 1=Medium, 2=Hard, 3=Expert)
    /// * `None` - If not over any Black difficulty button
    fn get_black_difficulty_at_coords(&self, coords: PhysicalPosition<f64>) -> Option<usize>;

    /// Check if screen coordinates are over the "Start Game" button
    ///
    /// # Arguments
    /// * `coords` - Physical position in pixels
    ///
    /// # Returns
    /// * `true` if over the start button
    fn is_coord_in_start_button(&self, coords: PhysicalPosition<f64>) -> bool;
}
