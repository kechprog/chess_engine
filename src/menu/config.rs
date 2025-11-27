//! Game configuration types returned by the menu system.
//!
//! When the user completes menu selection, a [`GameConfig`] is returned
//! containing all the information needed to start the game.

use crate::agent::ai::Difficulty;
use crate::game_repr::Color;

/// Configuration for a single player.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayerConfig {
    /// Human player controlled by mouse/keyboard input
    Human,
    /// AI player with specified difficulty
    AI { difficulty: Difficulty },
}

/// Complete game configuration returned by the menu.
///
/// Contains the game mode and player configurations for both sides.
#[derive(Debug, Clone)]
pub struct GameConfig {
    /// The selected game mode
    pub mode: GameMode,
    /// Configuration for the White player
    pub white_player: PlayerConfig,
    /// Configuration for the Black player
    pub black_player: PlayerConfig,
}

/// Game mode selection (separate from orchestrator's GameMode for clean separation).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameMode {
    /// Player vs Player - two humans
    PvP,
    /// Player vs AI - one human, one AI
    PvAI,
    /// AI vs AI - two AIs (for watching/testing)
    AIvAI,
}

impl GameConfig {
    /// Create a PvP game configuration.
    pub fn pvp() -> Self {
        Self {
            mode: GameMode::PvP,
            white_player: PlayerConfig::Human,
            black_player: PlayerConfig::Human,
        }
    }

    /// Create a PvAI game configuration.
    ///
    /// # Arguments
    /// * `user_color` - The color the human player will play as
    /// * `ai_difficulty` - The difficulty level for the AI opponent
    pub fn pvai(user_color: Color, ai_difficulty: Difficulty) -> Self {
        let (white_player, black_player) = match user_color {
            Color::White => (
                PlayerConfig::Human,
                PlayerConfig::AI { difficulty: ai_difficulty },
            ),
            Color::Black => (
                PlayerConfig::AI { difficulty: ai_difficulty },
                PlayerConfig::Human,
            ),
        };

        Self {
            mode: GameMode::PvAI,
            white_player,
            black_player,
        }
    }

    /// Create an AIvAI game configuration.
    ///
    /// # Arguments
    /// * `white_difficulty` - Difficulty for the White AI
    /// * `black_difficulty` - Difficulty for the Black AI
    pub fn aivai(white_difficulty: Difficulty, black_difficulty: Difficulty) -> Self {
        Self {
            mode: GameMode::AIvAI,
            white_player: PlayerConfig::AI { difficulty: white_difficulty },
            black_player: PlayerConfig::AI { difficulty: black_difficulty },
        }
    }

    /// Get the human player's color in a PvAI game.
    /// Returns None for PvP or AIvAI games.
    pub fn human_color(&self) -> Option<Color> {
        match self.mode {
            GameMode::PvAI => {
                if matches!(self.white_player, PlayerConfig::Human) {
                    Some(Color::White)
                } else {
                    Some(Color::Black)
                }
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pvp_config() {
        let config = GameConfig::pvp();
        assert_eq!(config.mode, GameMode::PvP);
        assert_eq!(config.white_player, PlayerConfig::Human);
        assert_eq!(config.black_player, PlayerConfig::Human);
    }

    #[test]
    fn test_pvai_config_white() {
        let config = GameConfig::pvai(Color::White, Difficulty::Hard);
        assert_eq!(config.mode, GameMode::PvAI);
        assert_eq!(config.white_player, PlayerConfig::Human);
        assert_eq!(config.black_player, PlayerConfig::AI { difficulty: Difficulty::Hard });
        assert_eq!(config.human_color(), Some(Color::White));
    }

    #[test]
    fn test_pvai_config_black() {
        let config = GameConfig::pvai(Color::Black, Difficulty::Easy);
        assert_eq!(config.mode, GameMode::PvAI);
        assert_eq!(config.white_player, PlayerConfig::AI { difficulty: Difficulty::Easy });
        assert_eq!(config.black_player, PlayerConfig::Human);
        assert_eq!(config.human_color(), Some(Color::Black));
    }

    #[test]
    fn test_aivai_config() {
        let config = GameConfig::aivai(Difficulty::Medium, Difficulty::Expert);
        assert_eq!(config.mode, GameMode::AIvAI);
        assert_eq!(config.white_player, PlayerConfig::AI { difficulty: Difficulty::Medium });
        assert_eq!(config.black_player, PlayerConfig::AI { difficulty: Difficulty::Expert });
        assert_eq!(config.human_color(), None);
    }
}
