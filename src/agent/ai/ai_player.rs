// AI Player implementation using MCTS

use crate::agent::player::Player;
use crate::game_repr::{Color, Move};
use crate::board::Board;
use super::mcts::search_multithreaded;
use std::sync::Arc;
use std::cell::RefCell;
use winit::event::WindowEvent;

/// AI Player that uses MCTS to select moves
pub struct AIPlayer {
    /// Reference to the board
    board: Arc<RefCell<Board>>,
    /// Number of MCTS iterations per move
    iterations: u32,
    /// Display name for this AI
    name: String,
    /// Number of threads to use (None = auto-detect)
    num_threads: Option<usize>,
}

impl AIPlayer {
    /// Create a new AI player
    ///
    /// # Arguments
    ///
    /// * `board` - Shared reference to the game board
    /// * `iterations` - Number of MCTS iterations to run per move (higher = stronger but slower)
    /// * `name` - Display name for this AI
    /// * `num_threads` - Number of threads to use (None = auto-detect from CPU cores)
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let board = Arc::new(RefCell::new(Board::new(renderer)));
    /// let ai = AIPlayer::new(board.clone(), 1000, "MCTS Bot".to_string(), None);
    /// ```
    pub fn new(board: Arc<RefCell<Board>>, iterations: u32, name: String, num_threads: Option<usize>) -> Self {
        Self {
            board,
            iterations,
            name,
            num_threads,
        }
    }

    /// Create a new AI player with default settings
    ///
    /// Uses 1000 iterations, "AI (MCTS)" as the name, and auto-detect threads
    pub fn new_default(board: Arc<RefCell<Board>>) -> Self {
        Self::new(board, 1000, "AI (MCTS)".to_string(), None)
    }

    /// Create an AI player with specific difficulty level
    ///
    /// * Easy: 100 iterations (~0.1s per move)
    /// * Medium: 500 iterations (~0.5s per move)
    /// * Hard: 2000 iterations (~2s per move)
    /// * Expert: 5000 iterations (~5s per move)
    ///
    /// Uses auto-detect for number of threads.
    pub fn with_difficulty(board: Arc<RefCell<Board>>, difficulty: Difficulty) -> Self {
        let (iterations, name) = match difficulty {
            Difficulty::Easy => (100, "AI (Easy)"),
            Difficulty::Medium => (500, "AI (Medium)"),
            Difficulty::Hard => (2000, "AI (Hard)"),
            Difficulty::Expert => (5000, "AI (Expert)"),
        };

        Self::new(board, iterations, name.to_string(), None)
    }

    /// Set the number of threads to use for MCTS search
    ///
    /// # Arguments
    ///
    /// * `num_threads` - Number of threads (None = auto-detect)
    pub fn with_threads(mut self, num_threads: Option<usize>) -> Self {
        self.num_threads = num_threads;
        self
    }
}

/// AI difficulty levels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Difficulty {
    Easy,
    Medium,
    Hard,
    Expert,
}

impl Player for AIPlayer {
    /// Get the next move from the AI
    ///
    /// This method runs MCTS for the configured number of iterations and returns
    /// the best move found. This is a blocking operation that may take several seconds
    /// depending on the iteration count.
    ///
    /// Uses multithreaded search with root parallelization for better performance.
    fn get_move(&mut self, color: Color) -> Option<Move> {
        // Get the current position from the board
        let position = {
            let board = self.board.borrow();
            board.position().clone()
        };

        // Use multithreaded search
        let (best_move, stats) = search_multithreaded(&position, color, self.iterations, self.num_threads);

        // Log search statistics (optional, can be removed in production)
        if cfg!(debug_assertions) {
            println!(
                "[{}] Searched {} iterations across {} threads, expanded {} unique moves, best move visited {} times",
                self.name, stats.root_visits, stats.num_threads, stats.num_children, stats.best_move_visits
            );
            println!("  Per-thread visits: {:?}", stats.per_thread_visits);
        }

        best_move
    }

    /// AI doesn't need to handle window events
    fn handle_event(&mut self, _event: &WindowEvent) {
        // AI players don't respond to UI events
    }

    /// Notification that opponent made a move
    ///
    /// AI could use this to start thinking in the background (not implemented yet)
    fn opponent_moved(&mut self, _mv: Move) {
        // Future: Could start pondering (thinking during opponent's time)
    }

    /// Get the AI's display name
    fn name(&self) -> &str {
        &self.name
    }

    /// AI automatically chooses promotion piece
    ///
    /// The MCTS search already evaluates different promotion pieces (Queen, Rook, Bishop, Knight)
    /// and returns the best one in the move itself. We return Some(Type::Queen) as a signal to
    /// the orchestrator that we don't need the promotion UI shown - the move already contains
    /// the chosen promotion piece type.
    fn get_promotion_choice(&self) -> Option<crate::game_repr::Type> {
        use crate::game_repr::Type;
        // Return Some to signal: "don't show UI, I already chose the piece"
        // The actual piece type is encoded in the move returned by get_move()
        Some(Type::Queen)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_difficulty_enum() {
        // Test that difficulty enum values are distinct
        assert_ne!(Difficulty::Easy, Difficulty::Medium);
        assert_ne!(Difficulty::Medium, Difficulty::Hard);
        assert_ne!(Difficulty::Hard, Difficulty::Expert);
    }

    // Integration tests with full Board would go here
    // For now, we test the AI logic via the MCTS tests
    // Full integration will be tested when wiring into the game
}
