//! AI Type Registry - Centralized registry for AI implementations
//!
//! This module provides a way to enumerate and instantiate different AI algorithms.
//! Currently only Negamax is implemented, but the architecture supports adding new
//! AI types (MCTS, Neural, Random, etc.) in the future.

use super::{NegamaxPlayer, Difficulty};
use super::search::iterative_deepening_search;
use crate::game_repr::{Color, Move, Position};
use crate::board::Board;
use crate::agent::player::Player;
use std::sync::Arc;
use std::cell::RefCell;

/// Enumeration of available AI algorithms
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AIType {
    /// Classical Negamax AI with alpha-beta pruning
    #[default]
    Negamax,
    // Future: MCTS, Neural, Random, etc.
}

impl AIType {
    /// Get all available AI types for UI enumeration
    pub fn all() -> &'static [AIType] {
        &[AIType::Negamax]
    }

    /// Get the display name for this AI type
    pub fn display_name(&self) -> &'static str {
        match self {
            AIType::Negamax => "Negamax",
        }
    }

    /// Get a short description of this AI type
    pub fn description(&self) -> &'static str {
        match self {
            AIType::Negamax => "Classical minimax with alpha-beta pruning",
        }
    }

    /// Check if this AI type supports difficulty levels
    pub fn supports_difficulty(&self) -> bool {
        match self {
            AIType::Negamax => true,
        }
    }

    /// Get available difficulty levels for this AI type
    pub fn available_difficulties(&self) -> &'static [Difficulty] {
        match self {
            AIType::Negamax => &[
                Difficulty::Easy,
                Difficulty::Medium,
                Difficulty::Hard,
                Difficulty::Expert,
            ],
        }
    }

    /// Get the default difficulty for this AI type
    pub fn default_difficulty(&self) -> Difficulty {
        match self {
            AIType::Negamax => Difficulty::Medium,
        }
    }

    /// Create a Player instance for this AI type
    ///
    /// This factory method creates a boxed Player trait object configured
    /// with the specified difficulty.
    pub fn create_player(
        &self,
        board: Arc<RefCell<Board>>,
        difficulty: Difficulty,
    ) -> Box<dyn Player> {
        match self {
            AIType::Negamax => Box::new(NegamaxPlayer::with_difficulty(board, difficulty)),
        }
    }

    /// Generate a move directly without creating a Player instance
    ///
    /// This is useful for AIvAI mode where we don't need persistent Player objects.
    /// The search is performed on the given position and returns the best move.
    pub fn generate_move(
        &self,
        position: &Position,
        color: Color,
        difficulty: Difficulty,
    ) -> Option<Move> {
        match self {
            AIType::Negamax => {
                let max_depth = difficulty.max_depth();
                let time_limit_ms = difficulty.time_limit_ms();

                let result = iterative_deepening_search(
                    position,
                    color,
                    max_depth,
                    time_limit_ms,
                );

                result.best_move
            }
        }
    }
}

/// Configuration for a single AI player
///
/// This stores all settings needed to create or invoke an AI player,
/// including the algorithm type and difficulty level.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AIConfig {
    /// The AI algorithm to use
    pub ai_type: AIType,
    /// The difficulty/strength level
    pub difficulty: Difficulty,
}

impl AIConfig {
    /// Create a new AI configuration
    pub fn new(ai_type: AIType, difficulty: Difficulty) -> Self {
        Self { ai_type, difficulty }
    }

    /// Generate a move using this configuration
    pub fn generate_move(&self, position: &Position, color: Color) -> Option<Move> {
        self.ai_type.generate_move(position, color, self.difficulty)
    }

    /// Create a Player instance from this configuration
    pub fn create_player(&self, board: Arc<RefCell<Board>>) -> Box<dyn Player> {
        self.ai_type.create_player(board, self.difficulty)
    }

    /// Get a display string for this configuration
    pub fn display_string(&self) -> String {
        format!("{} ({})", self.ai_type.display_name(), self.difficulty.name())
    }
}

impl Default for AIConfig {
    fn default() -> Self {
        Self {
            ai_type: AIType::default(),
            difficulty: AIType::default().default_difficulty(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ai_type_all() {
        let all = AIType::all();
        assert!(!all.is_empty());
        assert!(all.contains(&AIType::Negamax));
    }

    #[test]
    fn test_ai_type_display_name() {
        assert_eq!(AIType::Negamax.display_name(), "Negamax");
    }

    #[test]
    fn test_ai_type_supports_difficulty() {
        assert!(AIType::Negamax.supports_difficulty());
    }

    #[test]
    fn test_ai_type_available_difficulties() {
        let difficulties = AIType::Negamax.available_difficulties();
        assert_eq!(difficulties.len(), 4);
        assert!(difficulties.contains(&Difficulty::Easy));
        assert!(difficulties.contains(&Difficulty::Medium));
        assert!(difficulties.contains(&Difficulty::Hard));
        assert!(difficulties.contains(&Difficulty::Expert));
    }

    #[test]
    fn test_ai_config_default() {
        let config = AIConfig::default();
        assert_eq!(config.ai_type, AIType::Negamax);
        assert_eq!(config.difficulty, Difficulty::Medium);
    }

    #[test]
    fn test_ai_config_display_string() {
        let config = AIConfig::new(AIType::Negamax, Difficulty::Hard);
        assert_eq!(config.display_string(), "Negamax (Hard)");
    }
}
