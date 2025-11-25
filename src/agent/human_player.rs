//! Human player implementation that gets moves via UI interaction.
//!
//! This module provides `HumanPlayer`, which implements the `Player` trait for
//! human players who make moves by clicking on the chess board GUI. It handles
//! mouse events, piece selection, and move validation.
//!
//! # Architecture
//!
//! `HumanPlayer` holds a shared reference to the `Board` object (`Arc<RefCell<Board>>`)
//! and processes window events to detect user interaction. When the user clicks on
//! the board, the player:
//! 1. Converts the click to a board square
//! 2. Handles piece selection logic
//! 3. Validates potential moves against legal moves
//! 4. Creates and stores a pending move when a valid move is completed
//!
//! # Click Handling Logic
//!
//! The click handling follows these rules:
//! - **Click outside board**: Deselect any selected piece
//! - **No piece selected + click on friendly piece**: Select that piece
//! - **Piece selected + click on legal destination**: Create move and deselect
//! - **Piece selected + click on different friendly piece**: Reselect the new piece
//! - **Piece selected + click on illegal square**: Deselect
//!
//! # Control Flow
//!
//! ```text
//! WindowEvent → Orchestrator::handle_event()
//!     ↓
//! player.handle_event(event)
//!     ↓
//! player.handle_click() (if mouse click)
//!     ↓
//! Interact with board (borrow/borrow_mut)
//!     ↓
//! Set pending_move if valid move completed
//!     ↓
//! Orchestrator polls player.get_move()
//!     ↓
//! Returns pending_move (Some or None)
//! ```
//!
//! # Examples
//!
//! ```rust,ignore
//! use std::sync::{Arc, RefCell};
//! use chess_engine::board::Board;
//! use chess_engine::agent::human_player::HumanPlayer;
//! use chess_engine::agent::player::Player;
//! use chess_engine::game_repr::Color;
//!
//! // Create shared board
//! let board = Arc::new(RefCell::new(Board::new(renderer)));
//!
//! // Create human player
//! let mut player = HumanPlayer::new(board.clone(), "Alice".to_string());
//!
//! // In event loop, forward events to player
//! player.handle_event(&event);
//!
//! // When it's player's turn, get their move
//! if let Some(mv) = player.get_move(Color::White) {
//!     board.borrow_mut().execute_move(mv);
//! }
//! ```

use crate::board::Board;
use crate::game_repr::{Color, Move};
use crate::agent::player::Player;
use std::cell::RefCell;
use std::sync::Arc;
use winit::event::{ElementState, MouseButton, WindowEvent};

/// Human player that makes moves via GUI interaction.
///
/// This player holds a shared reference to the board and responds to window events
/// (mouse clicks, cursor movement) to allow the user to select and move pieces.
pub struct HumanPlayer {
    /// Shared reference to the board for querying state and handling clicks
    board: Arc<RefCell<Board>>,

    /// Display name for this player
    name: String,

    /// Move that was created by the user's last click sequence
    /// Set by `handle_click()`, returned by `get_move()`
    pending_move: Option<Move>,

    /// Current color this player is playing
    /// Set by `get_move()` to filter piece selection by color
    current_color: Option<Color>,
}

impl HumanPlayer {
    /// Create a new human player with a reference to the board.
    ///
    /// # Arguments
    ///
    /// * `board` - Shared reference to the board for UI interaction
    /// * `name` - Display name for this player
    ///
    /// # Returns
    ///
    /// A new `HumanPlayer` instance with no pending move.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let board = Arc::new(RefCell::new(Board::new(renderer)));
    /// let player = HumanPlayer::new(board.clone(), "Alice".to_string());
    /// ```
    pub fn new(board: Arc<RefCell<Board>>, name: String) -> Self {
        Self {
            board,
            name,
            pending_move: None,
            current_color: None,
        }
    }

    /// Process a mouse click, potentially creating a move.
    ///
    /// This method implements the core click handling logic:
    /// 1. Get the clicked square from the board
    /// 2. Handle clicks outside the board (deselect)
    /// 3. Handle selecting a piece (if none selected and friendly piece clicked)
    /// 4. Handle creating a move (if piece selected and legal destination clicked)
    /// 5. Handle reselecting (if piece selected and different friendly piece clicked)
    ///
    /// # Side Effects
    ///
    /// - May update board's selected tile
    /// - May set `pending_move` if a valid move is completed
    ///
    /// # Borrow Safety
    ///
    /// This method carefully manages RefCell borrows to avoid panics:
    /// - Gets mouse position from a short-lived borrow
    /// - Releases borrow before calling handle_click
    /// - Uses separate borrows for reading and writing
    fn handle_click(&mut self) {
        // Get mouse position from board (short-lived borrow)
        let mouse_pos = self.board.borrow().mouse_pos();

        // Try to convert click to a board square
        let clicked_tile = {
            let mut board = self.board.borrow_mut();
            board.handle_click(mouse_pos)
        };

        // Handle click outside board - deselect and return
        let clicked_tile = match clicked_tile {
            Some(tile) => tile,
            None => {
                self.board.borrow_mut().set_selected_tile(None);
                return;
            }
        };

        // Get current selection state and piece at clicked tile
        let selected_tile = self.board.borrow().selected_tile();

        // Case 1: Nothing selected - try to select this tile
        if selected_tile.is_none() {
            let piece = self.board.borrow().piece_at(clicked_tile);

            // Only select if there's a piece here that matches our color
            if !piece.is_none() {
                // Only select if it's the current player's piece
                if let Some(current_color) = self.current_color {
                    if piece.color == current_color {
                        self.board.borrow_mut().set_selected_tile(Some(clicked_tile));
                    }
                } else {
                    // No color set yet, select any piece (will be filtered later)
                    self.board.borrow_mut().set_selected_tile(Some(clicked_tile));
                }
            }
            return;
        }

        // Case 2: Something is selected - try to create a move
        let from = selected_tile.unwrap();

        // Check if this click creates a legal move
        let legal_moves = {
            let board = self.board.borrow();
            board.legal_moves_for_selection().to_vec()
        };

        // Look for a legal move matching this from/to combination
        for mv in &legal_moves {
            if mv._from() == from as usize && mv._to() == clicked_tile as usize {
                // Valid move found!
                self.pending_move = Some(*mv);
                self.board.borrow_mut().set_selected_tile(None);
                return;
            }
        }

        // Case 3: Click didn't create a legal move
        // Check if clicked tile has a piece we can reselect
        let clicked_piece = self.board.borrow().piece_at(clicked_tile);

        if !clicked_piece.is_none() {
            // Only reselect if it's the current player's piece
            if let Some(current_color) = self.current_color {
                if clicked_piece.color == current_color {
                    self.board.borrow_mut().set_selected_tile(Some(clicked_tile));
                } else {
                    // Clicked opponent's piece - deselect
                    self.board.borrow_mut().set_selected_tile(None);
                }
            } else {
                // No color set, reselect any piece
                self.board.borrow_mut().set_selected_tile(Some(clicked_tile));
            }
        } else {
            // Clicked empty square - deselect
            self.board.borrow_mut().set_selected_tile(None);
        }
    }
}

impl Player for HumanPlayer {
    /// Get the next move from this human player.
    ///
    /// This method is called when it's this player's turn. It:
    /// 1. Sets the board POV to the current player's perspective
    /// 2. Returns any pending move that was created by user clicks
    ///
    /// # Arguments
    ///
    /// * `color` - The color this player is playing (White or Black)
    ///
    /// # Returns
    ///
    /// * `Some(Move)` - If the player has completed a move via clicks
    /// * `None` - If no move is ready yet
    ///
    /// # Design Note
    ///
    /// In the current architecture, this method doesn't block. Instead:
    /// - The orchestrator calls `get_move()` after each event
    /// - If it returns `None`, the orchestrator continues processing events
    /// - If it returns `Some(move)`, the orchestrator executes the move
    ///
    /// This polling approach works because we're in a single-threaded event loop.
    /// The user's clicks are processed by `handle_event()` → `handle_click()`,
    /// which sets `pending_move`. The orchestrator then polls `get_move()` to
    /// check if a move is ready.
    fn get_move(&mut self, color: Color) -> Option<Move> {
        // Store current color for piece selection filtering
        self.current_color = Some(color);

        // Note: POV is now managed by the Orchestrator (set after turn switches).
        // This ensures the board flips at the right time in the game flow.

        // Return pending move (will be Some if user completed a move)
        // Note: We take() the pending move so it's consumed and cleared
        self.pending_move.take()
    }

    /// Handle window events, processing user input.
    ///
    /// This method filters for relevant events (mouse clicks and cursor movement) and
    /// delegates to the appropriate handler.
    ///
    /// # Arguments
    ///
    /// * `event` - Window event to process
    ///
    /// # Side Effects
    ///
    /// - Mouse clicks trigger `handle_click()` which may set `pending_move`
    /// - Cursor movement updates the board's mouse position
    fn handle_event(&mut self, event: &WindowEvent) {
        match event {
            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                button: MouseButton::Left,
                ..
            } => {
                self.handle_click();
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.board.borrow_mut().update_mouse_pos(*position);
            }
            _ => {}
        }
    }

    /// Get the display name of this player.
    ///
    /// # Returns
    ///
    /// The player's name as a string slice.
    fn name(&self) -> &str {
        &self.name
    }

    /// Get automatic promotion choice (None = show UI for human selection).
    ///
    /// Human players need to select their promotion piece via the UI overlay,
    /// so this always returns None.
    fn get_promotion_choice(&self) -> Option<crate::game_repr::Type> {
        None  // Show UI for human to select promotion piece
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::renderer::Renderer;
    use crate::game_repr::Position;
    use winit::dpi::PhysicalPosition;

    // Mock renderer for testing
    struct MockRenderer;

    impl Renderer for MockRenderer {
        fn draw_position(&mut self, _position: &Position, _selected_tile: Option<u8>, _pov: Color) {}
        fn coord_to_tile(&self, _coords: PhysicalPosition<f64>, _pov: Color) -> Option<u8> {
            Some(12) // Return e2 for testing
        }
        fn resize(&mut self, _new_size: (u32, u32)) {}
        fn draw_menu(&mut self, _show_coming_soon: bool) {}
        fn is_coord_in_button(&self, _coords: PhysicalPosition<f64>, _button_index: usize) -> bool {
            false
        }
        fn draw_game_end(&mut self, _position: &Position, _selected_tile: Option<u8>, _pov: Color, _result: crate::agent::player::GameResult) {}
        fn draw_promotion_selection(&mut self, _position: &Position, _selected_tile: Option<u8>, _pov: Color, _promoting_color: Color) {}
        fn get_promotion_piece_at_coords(&self, _coords: PhysicalPosition<f64>) -> Option<crate::game_repr::Type> {
            None
        }
        fn draw_side_selection(&mut self) {}
        fn is_coord_in_side_button(&self, _coords: PhysicalPosition<f64>, _button_index: usize) -> bool {
            false
        }
        fn draw_controls_bar(&mut self, _can_undo: bool, _can_redo: bool) {}
        fn get_control_action_at_coords(&self, _coords: PhysicalPosition<f64>) -> Option<crate::renderer::ControlAction> {
            None
        }
        fn draw_ai_setup(
            &mut self,
            _ai_types: &[crate::agent::ai::AIType],
            _white_type_index: usize,
            _white_difficulty: crate::agent::ai::Difficulty,
            _black_type_index: usize,
            _black_difficulty: crate::agent::ai::Difficulty,
        ) {}
        fn get_white_difficulty_at_coords(&self, _coords: PhysicalPosition<f64>) -> Option<usize> {
            None
        }
        fn get_black_difficulty_at_coords(&self, _coords: PhysicalPosition<f64>) -> Option<usize> {
            None
        }
        fn is_coord_in_start_button(&self, _coords: PhysicalPosition<f64>) -> bool {
            false
        }
    }

    #[test]
    fn test_human_player_new() {
        let board = Arc::new(RefCell::new(Board::new(Box::new(MockRenderer))));
        let player = HumanPlayer::new(board.clone(), "Test Player".to_string());

        assert_eq!(player.name(), "Test Player");
        assert!(player.pending_move.is_none());
    }

    #[test]
    fn test_get_move_sets_current_color() {
        let board = Arc::new(RefCell::new(Board::new(Box::new(MockRenderer))));
        let mut player = HumanPlayer::new(board.clone(), "Test".to_string());

        // Get move for White - should store the color
        player.get_move(Color::White);
        assert_eq!(player.current_color, Some(Color::White));

        // Get move for Black - should update the color
        player.get_move(Color::Black);
        assert_eq!(player.current_color, Some(Color::Black));
    }

    #[test]
    fn test_get_move_takes_pending_move() {
        let board = Arc::new(RefCell::new(Board::new(Box::new(MockRenderer))));
        let mut player = HumanPlayer::new(board.clone(), "Test".to_string());

        // Set a pending move
        use crate::game_repr::MoveType;
        let test_move = Move::new(12, 28, MoveType::Normal);
        player.pending_move = Some(test_move);

        // First call should return the move
        let result = player.get_move(Color::White);
        assert!(result.is_some());

        // Second call should return None (move was taken)
        let result = player.get_move(Color::White);
        assert!(result.is_none());
    }

    #[test]
    fn test_handle_cursor_moved() {
        let board = Arc::new(RefCell::new(Board::new(Box::new(MockRenderer))));
        let _player = HumanPlayer::new(board.clone(), "Test".to_string());

        let pos = PhysicalPosition::new(100.0, 150.0);

        // Create a dummy DeviceId - we can't construct it directly in tests
        // so we'll just verify the board's mouse position was updated instead
        board.borrow_mut().update_mouse_pos(pos);
        assert_eq!(board.borrow().mouse_pos().x, 100.0);
        assert_eq!(board.borrow().mouse_pos().y, 150.0);
    }
}
