//! Player trait and associated types for chess game agents.
//!
//! This module provides the core abstraction for entities that can provide chess moves.
//! Different player types (Human, AI, Network) implement the `Player` trait to participate
//! in games orchestrated by the game coordinator.
//!
//! # Design Philosophy
//!
//! The `Player` trait focuses on **behavior** rather than construction. Different player
//! implementations require different initialization parameters:
//! - `HumanPlayer` needs a reference to the board for UI interaction
//! - `AIPlayer` needs engine configuration and difficulty settings
//! - `NetworkPlayer` needs connection details and authentication
//!
//! Therefore, the trait does not define a constructor method. Each implementation provides
//! its own constructor tailored to its specific needs.
//!
//! # Examples
//!
//! ```rust,no_run
//! use chess_engine::agent::player::{Player, GameResult};
//! use chess_engine::game_repr::{Color, Move};
//! use std::sync::{Arc, RefCell};
//!
//! // Example: HumanPlayer construction (conceptual)
//! // let board = Arc::new(RefCell::new(Board::new(renderer)));
//! // let player = HumanPlayer::new(board.clone(), "Alice".to_string());
//! //
//! // Example: AIPlayer construction (conceptual)
//! // let ai_player = AIPlayer::new(board.clone(), Difficulty::Hard, "Stockfish".to_string());
//! ```
//!
//! # Synchronous Design
//!
//! The `get_move()` method is intentionally synchronous (blocking) rather than async.
//! This design choice simplifies the control flow for a turn-based game:
//! - `HumanPlayer` can block waiting for user input events
//! - `AIPlayer` can block during move search/computation
//! - The orchestrator simply calls `get_move()` and waits for a result
//!
//! This approach is sufficient for single-threaded gameplay. If future requirements
//! demand non-blocking behavior (e.g., background AI computation, network operations),
//! the trait can be migrated to async methods.

use crate::game_repr::{Color, Move};
use winit::event::WindowEvent;

/// Result of a completed chess game.
///
/// This enum represents all possible game outcomes. It is passed to players
/// via `game_ended()` to notify them of the final result.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameResult {
    /// White player won the game (Black was checkmated or resigned)
    WhiteWins,
    /// Black player won the game (White was checkmated or resigned)
    BlackWins,
    /// Game ended in a draw (by agreement, insufficient material, 50-move rule, etc.)
    Draw,
    /// Game ended in stalemate (player to move has no legal moves but is not in check)
    Stalemate,
}

impl GameResult {
    /// Create a GameResult from the winning color
    pub fn from_winner(winner: Color) -> Self {
        match winner {
            Color::White => GameResult::WhiteWins,
            Color::Black => GameResult::BlackWins,
        }
    }
}

/// Trait for entities that can provide chess moves.
///
/// This trait abstracts the concept of a "player" in a chess game. A player is any
/// entity that can be asked to provide a move for a given color. This could be:
/// - A human player interacting through the UI
/// - An AI engine computing moves algorithmically
/// - A remote player over a network connection
/// - A replay system reading moves from a saved game
///
/// # Required Methods
///
/// Only `get_move()` must be implemented. All other methods have default implementations
/// that can be overridden as needed.
///
/// # Method Behavior
///
/// ## `get_move()`
/// - **Blocking**: This method may block until a move is available
/// - **Returns `None`**: If the player cancels, resigns, or disconnects
/// - **Returns `Some(Move)`**: When a valid move is selected
/// - The move returned must be legal in the current position (validation is typically
///   done by the caller/orchestrator)
///
/// ## `handle_event()`
/// - Default: Does nothing
/// - Override: For interactive players that need to respond to window events (mouse, keyboard)
///
/// ## `opponent_moved()`
/// - Default: Does nothing
/// - Override: To display opponent moves, update UI, log moves, etc.
///
/// ## `game_ended()`
/// - Default: Does nothing
/// - Override: To display game result, show statistics, save game, etc.
///
/// ## `name()`
/// - Default: Returns "Player"
/// - Override: To provide a custom player name for display
///
/// # Thread Safety
///
/// The trait is not `Send` or `Sync` by default. Implementations are expected to
/// run on the main thread (UI thread). If concurrent access is needed, wrap
/// implementations in appropriate synchronization primitives.
pub trait Player {
    /// Request the next move from this player.
    ///
    /// This method is called when it's this player's turn to move. The `color` parameter
    /// indicates which side the player is playing (White or Black).
    ///
    /// # Blocking Behavior
    ///
    /// This method **may block** until a move is available:
    /// - `HumanPlayer` blocks until the user makes a selection via the UI
    /// - `AIPlayer` blocks during search/computation
    /// - `NetworkPlayer` blocks waiting for data from the network
    ///
    /// # Return Value
    ///
    /// - `Some(Move)`: A valid move selected by the player
    /// - `None`: The player cannot or will not provide a move (resignation, cancellation,
    ///   disconnection, etc.)
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use chess_engine::agent::player::Player;
    /// use chess_engine::game_repr::Color;
    ///
    /// fn play_turn(player: &mut dyn Player) {
    ///     match player.get_move(Color::White) {
    ///         Some(mv) => {
    ///             // Execute the move
    ///             println!("Player chose move: {:?}", mv);
    ///         }
    ///         None => {
    ///             // Player resigned or cancelled
    ///             println!("Player resigned!");
    ///         }
    ///     }
    /// }
    /// ```
    fn get_move(&mut self, color: Color) -> Option<Move>;

    /// Handle window event (for interactive players).
    ///
    /// This method is called for each window event when it's this player's turn.
    /// Interactive players (like `HumanPlayer`) override this to respond to
    /// mouse clicks, keyboard input, etc.
    ///
    /// # Default Implementation
    ///
    /// Does nothing. AI players and other non-interactive players don't need to
    /// respond to events.
    ///
    /// # Parameters
    ///
    /// - `event`: The window event to handle
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use chess_engine::agent::player::Player;
    /// use winit::event::WindowEvent;
    ///
    /// fn process_event(player: &mut dyn Player, event: &WindowEvent) {
    ///     player.handle_event(event);
    /// }
    /// ```
    fn handle_event(&mut self, _event: &WindowEvent) {
        // Default: do nothing (for AI players)
    }

    /// Notify this player that the opponent made a move.
    ///
    /// This method is called after the opponent's move has been executed. It allows
    /// the player to react to the opponent's move, such as:
    /// - Updating UI to show the move
    /// - Logging the move
    /// - Starting to think about the next move (for AI)
    /// - Sending the move to a remote client (for network play)
    ///
    /// # Default Implementation
    ///
    /// Does nothing. Override this method if your player needs to react to opponent moves.
    ///
    /// # Parameters
    ///
    /// - `mv`: The move that the opponent just made
    fn opponent_moved(&mut self, _mv: Move) {
        // Default: do nothing
    }

    /// Notify this player that the game has ended.
    ///
    /// This method is called when the game reaches a terminal state (checkmate, stalemate,
    /// draw, resignation, etc.). It allows the player to react to the game ending, such as:
    /// - Displaying the result to the user
    /// - Saving statistics
    /// - Closing network connections
    /// - Cleaning up resources
    ///
    /// # Default Implementation
    ///
    /// Does nothing. Override this method if your player needs to react to game endings.
    ///
    /// # Parameters
    ///
    /// - `result`: The final result of the game
    fn game_ended(&mut self, _result: GameResult) {
        // Default: do nothing
    }

    /// Get the display name of this player.
    ///
    /// This method returns a human-readable name for the player, used for:
    /// - Displaying in the UI
    /// - Logging game events
    /// - Saving game records
    ///
    /// # Default Implementation
    ///
    /// Returns `"Player"`. Override this method to provide a custom name.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use chess_engine::agent::player::Player;
    ///
    /// fn show_player_name(player: &dyn Player) {
    ///     println!("Player name: {}", player.name());
    /// }
    /// ```
    fn name(&self) -> &str {
        "Player"
    }
}
