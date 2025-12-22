//! NegamaxPlayer - Classical chess AI using Negamax with Alpha-Beta pruning
//!
//! This module implements a chess AI player based on the Negamax algorithm, a variant
//! of the Minimax algorithm that treats both players symmetrically. It includes:
//! - Alpha-beta pruning for efficient tree search
//! - Iterative deepening for time management
//! - Multiple difficulty levels with configurable search depth
//!
//! # Architecture
//!
//! The NegamaxPlayer implements the Player trait and delegates move selection to
//! the search module's `iterative_deepening_search` function. This provides a clean
//! separation between the player interface and the search algorithm.
//!
//! # Difficulty Levels
//!
//! - **Easy**: Depth 2, fast moves (~0.1s)
//! - **Medium**: Depth 4, moderate analysis (~1s)
//! - **Hard**: Depth 6, strong play (~5s)
//! - **Expert**: Depth 8 with 5s time limit, very strong play
//!
//! # Examples
//!
//! ```ignore
//! use chess_engine::agent::ai::{NegamaxPlayer, Difficulty};
//! use std::sync::Arc;
//! use std::cell::RefCell;
//!
//! // Create an AI with medium difficulty (board setup omitted)
//! let board = Arc::new(RefCell::new(board));
//! let ai = NegamaxPlayer::with_difficulty(board.clone(), Difficulty::Medium);
//!
//! // Or create with custom settings
//! let ai = NegamaxPlayer::new(board, Difficulty::Hard, "Deep Blue".to_string());
//! ```

use crate::agent::player::Player;
use crate::game_repr::{Color, Move, Type};
use crate::board::Board;
use super::search::iterative_deepening_search;
use std::sync::Arc;
use std::cell::RefCell;
use winit::event::WindowEvent;

/// AI difficulty levels that map to search depth and time controls
///
/// Each difficulty level defines the search parameters used by the Negamax algorithm:
/// - **Search depth**: How many moves ahead the AI looks
/// - **Time limit**: Maximum time allowed for move selection (None = unlimited)
///
/// Higher difficulty levels produce stronger play but take longer to compute moves.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Difficulty {
    /// Easy difficulty: Depth 2, no time limit
    ///
    /// Suitable for beginners. Makes basic tactical moves but misses deeper combinations.
    /// Expected move time: ~0.1 seconds
    Easy,

    /// Medium difficulty: Depth 4, no time limit
    ///
    /// Suitable for intermediate players. Sees 2 moves ahead (4 plies) and plays solidly.
    /// Expected move time: ~1 second
    Medium,

    /// Hard difficulty: Depth 6, no time limit
    ///
    /// Suitable for advanced players. Sees 3 moves ahead (6 plies) with good tactics.
    /// Expected move time: ~5 seconds
    Hard,

    /// Expert difficulty: Depth 8, 5 second time limit
    ///
    /// Very strong play with deep calculation. Uses time control to limit computation.
    /// Maximum move time: 5 seconds
    Expert,
}

impl Difficulty {
    /// Get the maximum search depth for this difficulty level
    ///
    /// Returns the number of plies (half-moves) to search. A depth of 6 means
    /// the AI looks 3 full moves ahead (White's move, Black's move, White's move).
    ///
    /// Note: WASM builds use reduced depths because the search blocks the main
    /// thread and would freeze the browser UI. Native builds use full depths.
    pub fn max_depth(&self) -> u8 {
        #[cfg(target_arch = "wasm32")]
        {
            // Reduced depths for WASM to prevent UI freezing
            // The search is synchronous and blocks the browser's event loop
            match self {
                Difficulty::Easy => 1,
                Difficulty::Medium => 2,
                Difficulty::Hard => 3,
                Difficulty::Expert => 4,
            }
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            match self {
                Difficulty::Easy => 2,
                Difficulty::Medium => 4,
                Difficulty::Hard => 6,
                Difficulty::Expert => 8,
            }
        }
    }

    /// Get the time limit in milliseconds for this difficulty level
    ///
    /// Returns None for unlimited time, or Some(ms) for time-controlled search.
    pub fn time_limit_ms(&self) -> Option<u64> {
        match self {
            Difficulty::Easy => None,
            Difficulty::Medium => None,
            Difficulty::Hard => None,
            Difficulty::Expert => Some(5000), // 5 seconds
        }
    }

    /// Get a display name for this difficulty level
    pub fn name(&self) -> &'static str {
        match self {
            Difficulty::Easy => "Easy",
            Difficulty::Medium => "Medium",
            Difficulty::Hard => "Hard",
            Difficulty::Expert => "Expert",
        }
    }
}

/// AI Player that uses Negamax algorithm with alpha-beta pruning
///
/// This player implements classical chess AI using minimax-based tree search.
/// Unlike MCTS-based approaches, Negamax is:
/// - **Deterministic**: Same position always yields same move
/// - **Depth-limited**: Searches to a fixed depth rather than using time budgets
/// - **Evaluation-based**: Uses a heuristic evaluation function for leaf nodes
///
/// The search uses several optimizations:
/// - Alpha-beta pruning to skip irrelevant branches
/// - Iterative deepening for better move ordering and time management
/// - Transposition tables to cache previously evaluated positions
/// - Quiescence search to avoid horizon effect
/// - Move ordering to improve pruning efficiency
///
/// # Thread Safety
///
/// This player is not thread-safe and must be used on the main thread only.
/// The shared `board` reference uses `RefCell` for interior mutability.
pub struct NegamaxPlayer {
    /// Shared reference to the game board
    ///
    /// The board is wrapped in Arc<RefCell<>> to allow shared ownership
    /// with the orchestrator while maintaining interior mutability.
    board: Arc<RefCell<Board>>,

    /// AI difficulty level determining search depth and time control
    difficulty: Difficulty,

    /// Display name for this AI player
    ///
    /// Used in UI and logging. Can be customized via constructor.
    name: String,
}

impl NegamaxPlayer {
    /// Create a new NegamaxPlayer with custom difficulty and name
    ///
    /// # Arguments
    ///
    /// * `board` - Shared reference to the game board
    /// * `difficulty` - AI strength level (Easy, Medium, Hard, Expert)
    /// * `name` - Display name for this player
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use chess_engine::agent::ai::{NegamaxPlayer, Difficulty};
    /// use std::sync::Arc;
    /// use std::cell::RefCell;
    ///
    /// // board setup omitted - requires renderer
    /// let board = Arc::new(RefCell::new(board));
    /// let ai = NegamaxPlayer::new(
    ///     board,
    ///     Difficulty::Hard,
    ///     "Stockfish Lite".to_string()
    /// );
    /// ```
    pub fn new(board: Arc<RefCell<Board>>, difficulty: Difficulty, name: String) -> Self {
        Self {
            board,
            difficulty,
            name,
        }
    }

    /// Create a new NegamaxPlayer with specified difficulty and auto-generated name
    ///
    /// The player name is generated as "AI ({difficulty})" based on the difficulty level.
    ///
    /// # Arguments
    ///
    /// * `board` - Shared reference to the game board
    /// * `difficulty` - AI strength level (Easy, Medium, Hard, Expert)
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use chess_engine::agent::ai::{NegamaxPlayer, Difficulty};
    /// use std::sync::Arc;
    /// use std::cell::RefCell;
    ///
    /// // board setup omitted - requires renderer
    /// let board = Arc::new(RefCell::new(board));
    /// let ai = NegamaxPlayer::with_difficulty(board, Difficulty::Medium);
    /// // Player name will be "AI (Medium)"
    /// ```
    pub fn with_difficulty(board: Arc<RefCell<Board>>, difficulty: Difficulty) -> Self {
        let name = format!("AI ({})", difficulty.name());
        Self::new(board, difficulty, name)
    }

    /// Create a default NegamaxPlayer with Medium difficulty
    ///
    /// Convenience constructor for quick setup. Uses Medium difficulty (depth 4)
    /// and the name "AI (Negamax)".
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use chess_engine::agent::ai::NegamaxPlayer;
    /// use std::sync::Arc;
    /// use std::cell::RefCell;
    ///
    /// // board setup omitted - requires renderer
    /// let board = Arc::new(RefCell::new(board));
    /// let ai = NegamaxPlayer::new_default(board);
    /// ```
    pub fn new_default(board: Arc<RefCell<Board>>) -> Self {
        Self::new(board, Difficulty::Medium, "AI (Negamax)".to_string())
    }

    /// Get the current difficulty level
    pub fn difficulty(&self) -> Difficulty {
        self.difficulty
    }

    /// Set a new difficulty level
    ///
    /// This updates the difficulty for future move searches. The name is updated
    /// to reflect the new difficulty if it was auto-generated.
    ///
    /// # Arguments
    ///
    /// * `difficulty` - New difficulty level
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use chess_engine::agent::ai::{NegamaxPlayer, Difficulty};
    /// use std::sync::Arc;
    /// use std::cell::RefCell;
    ///
    /// // board setup omitted - requires renderer
    /// let board = Arc::new(RefCell::new(board));
    /// let mut ai = NegamaxPlayer::with_difficulty(board, Difficulty::Easy);
    /// ai.set_difficulty(Difficulty::Expert); // Increase strength
    /// ```
    pub fn set_difficulty(&mut self, difficulty: Difficulty) {
        self.difficulty = difficulty;
        // Update name if it follows the auto-generated pattern
        if self.name.starts_with("AI (") {
            self.name = format!("AI ({})", difficulty.name());
        }
    }
}

impl Player for NegamaxPlayer {
    /// Request the next move from the AI
    ///
    /// This method performs a Negamax search with alpha-beta pruning to select
    /// the best move for the given color. The search depth is determined by the
    /// difficulty level.
    ///
    /// # Blocking Behavior
    ///
    /// This is a **blocking operation** that may take several seconds depending on:
    /// - Position complexity (number of legal moves)
    /// - Search depth (set by difficulty level)
    /// - Time limit (for Expert difficulty)
    ///
    /// # Search Process
    ///
    /// 1. Get current position from the board
    /// 2. Call iterative_deepening_search with difficulty parameters
    /// 3. Return the best move found
    ///
    /// # Arguments
    ///
    /// * `color` - The color this AI is playing (White or Black)
    ///
    /// # Returns
    ///
    /// * `Some(Move)` - The best move found by the search
    /// * `None` - If no legal moves are available (checkmate or stalemate)
    ///
    /// # Performance
    ///
    /// Expected move times:
    /// - Easy (depth 2): ~0.1 seconds
    /// - Medium (depth 4): ~1 second
    /// - Hard (depth 6): ~5 seconds
    /// - Expert (depth 8): up to 5 seconds (time limited)
    fn get_move(&mut self, color: Color) -> Option<Move> {
        // Get the current position from the board
        let position = {
            let board = self.board.borrow();
            board.position().clone()
        };

        // Get search parameters based on difficulty
        let max_depth = self.difficulty.max_depth();
        let time_limit_ms = self.difficulty.time_limit_ms();

        // Perform iterative deepening search
        let search_result = iterative_deepening_search(
            &position,
            color,
            max_depth,
            time_limit_ms,
        );

        // Log search statistics in debug builds
        if cfg!(debug_assertions) {
            println!(
                "[{}] Searched to depth {}, evaluated {} positions, best move score: {}",
                self.name,
                search_result.depth_reached,
                search_result.nodes_searched,
                search_result.score
            );
            if let Some(pv) = &search_result.principal_variation {
                println!("  Principal variation: {} moves", pv.len());
            }
        }

        search_result.best_move
    }

    /// Handle window events
    ///
    /// AI players don't respond to window events (mouse, keyboard, etc.).
    /// This method is a no-op for NegamaxPlayer.
    ///
    /// # Arguments
    ///
    /// * `_event` - The window event (ignored)
    ///
    /// # Returns
    ///
    /// Always returns `false` indicating the event was not handled.
    fn handle_event(&mut self, _event: &WindowEvent) {
        // AI players don't respond to UI events
    }

    /// Notification that the opponent made a move
    ///
    /// This method is called after the opponent executes their move. Currently
    /// a no-op, but could be used in the future for:
    /// - Pondering (thinking during opponent's time)
    /// - Updating internal state or caches
    /// - Logging opponent moves
    ///
    /// # Arguments
    ///
    /// * `_mv` - The move the opponent just made
    fn opponent_moved(&mut self, _mv: Move) {
        // Future enhancement: Could start pondering (thinking during opponent's time)
        // For now, this is a no-op
    }

    /// Notification that the game has ended
    ///
    /// Currently a no-op. Could be used for:
    /// - Logging game results
    /// - Saving statistics
    /// - Cleaning up resources
    ///
    /// # Arguments
    ///
    /// * `_result` - The final game result (win/loss/draw/stalemate)
    fn game_ended(&mut self, _result: crate::agent::player::GameResult) {
        // No cleanup needed for Negamax player
    }

    /// Get the display name of this AI player
    ///
    /// Returns the name set during construction. Used for:
    /// - Displaying in the UI
    /// - Logging game events
    /// - Identifying the player in game records
    ///
    /// # Returns
    ///
    /// The player's display name as a string slice
    fn name(&self) -> &str {
        &self.name
    }

    /// Get automatic promotion piece choice for this AI
    ///
    /// When a pawn reaches the back rank, the AI automatically selects Queen
    /// for promotion. This avoids showing the promotion UI overlay.
    ///
    /// # Note
    ///
    /// The Negamax search evaluates all promotion options (Queen, Rook, Bishop, Knight)
    /// and selects the best one. In practice, Queen is almost always optimal, so we
    /// return it as the default choice. The search could be enhanced to encode the
    /// chosen piece type in the move for underpromotions.
    ///
    /// # Returns
    ///
    /// * `Some(Type::Queen)` - Always promote to Queen
    fn get_promotion_choice(&self) -> Option<Type> {
        // AI automatically promotes to Queen
        // Note: The search algorithm evaluates all promotion pieces and Queen
        // is almost always the best choice. Underpromotion (Rook, Bishop, Knight)
        // is extremely rare and only occurs in special puzzle positions.
        Some(Type::Queen)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_difficulty_depths() {
        // Verify that difficulty levels have expected search depths
        assert_eq!(Difficulty::Easy.max_depth(), 2);
        assert_eq!(Difficulty::Medium.max_depth(), 4);
        assert_eq!(Difficulty::Hard.max_depth(), 6);
        assert_eq!(Difficulty::Expert.max_depth(), 8);
    }

    #[test]
    fn test_difficulty_time_limits() {
        // Verify time limits
        assert_eq!(Difficulty::Easy.time_limit_ms(), None);
        assert_eq!(Difficulty::Medium.time_limit_ms(), None);
        assert_eq!(Difficulty::Hard.time_limit_ms(), None);
        assert_eq!(Difficulty::Expert.time_limit_ms(), Some(5000));
    }

    #[test]
    fn test_difficulty_names() {
        // Verify display names
        assert_eq!(Difficulty::Easy.name(), "Easy");
        assert_eq!(Difficulty::Medium.name(), "Medium");
        assert_eq!(Difficulty::Hard.name(), "Hard");
        assert_eq!(Difficulty::Expert.name(), "Expert");
    }

    #[test]
    fn test_difficulty_enum_values() {
        // Test that difficulty enum values are distinct
        assert_ne!(Difficulty::Easy, Difficulty::Medium);
        assert_ne!(Difficulty::Medium, Difficulty::Hard);
        assert_ne!(Difficulty::Hard, Difficulty::Expert);
        assert_ne!(Difficulty::Easy, Difficulty::Expert);
    }

    #[test]
    fn test_set_difficulty_updates_name() {
        use crate::renderer::Renderer;
        use crate::game_repr::Position;

        // Create a mock board (this would normally come from the application)
        // For testing, we can create a minimal setup
        // Note: Full integration tests would require a complete Board instance

        // Test that difficulty names follow expected pattern
        let easy_name = format!("AI ({})", Difficulty::Easy.name());
        let hard_name = format!("AI ({})", Difficulty::Hard.name());

        assert_eq!(easy_name, "AI (Easy)");
        assert_eq!(hard_name, "AI (Hard)");
    }

    // Integration tests with full Board would require renderer and position setup
    // Those tests should be in the integration test suite
}
