//! Menu state machine types.
//!
//! Defines the various states the menu can be in and the data associated with each.

use crate::agent::ai::Difficulty;
use crate::game_repr::Color;

/// State of the AIvAI setup screen.
///
/// Tracks selected difficulty for both White and Black AI players.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AIvAISetupState {
    /// Currently selected difficulty for White AI
    pub white_difficulty: Difficulty,
    /// Currently selected difficulty for Black AI
    pub black_difficulty: Difficulty,
}

impl Default for AIvAISetupState {
    fn default() -> Self {
        Self {
            white_difficulty: Difficulty::Medium,
            black_difficulty: Difficulty::Medium,
        }
    }
}

/// Menu state machine.
///
/// Represents the current screen/state of the menu system.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MenuState {
    /// Main menu - select game mode (PvP, PvAI, AIvAI)
    ModeSelection,

    /// Side selection for PvAI - choose to play as White or Black
    SideSelection,

    /// Difficulty selection for PvAI - choose AI difficulty
    /// Contains the user's chosen color from the previous screen
    DifficultySelection {
        /// The color the user chose to play as
        user_color: Color,
    },

    /// AI setup for AIvAI mode - configure both AIs
    AIvAISetup(AIvAISetupState),
}

impl Default for MenuState {
    fn default() -> Self {
        Self::ModeSelection
    }
}

impl MenuState {
    /// Create a new menu state starting at mode selection.
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if this is the top-level menu (can't go back further).
    pub fn is_top_level(&self) -> bool {
        matches!(self, MenuState::ModeSelection)
    }

    /// Get the parent state to return to when pressing Escape.
    /// Returns None if already at top level.
    pub fn parent(&self) -> Option<MenuState> {
        match self {
            MenuState::ModeSelection => None,
            MenuState::SideSelection => Some(MenuState::ModeSelection),
            MenuState::DifficultySelection { .. } => Some(MenuState::SideSelection),
            MenuState::AIvAISetup(_) => Some(MenuState::ModeSelection),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_state() {
        let state = MenuState::default();
        assert!(matches!(state, MenuState::ModeSelection));
        assert!(state.is_top_level());
    }

    #[test]
    fn test_aivai_setup_default() {
        let setup = AIvAISetupState::default();
        assert_eq!(setup.white_difficulty, Difficulty::Medium);
        assert_eq!(setup.black_difficulty, Difficulty::Medium);
    }

    #[test]
    fn test_parent_from_mode_selection() {
        let state = MenuState::ModeSelection;
        assert!(state.parent().is_none());
    }

    #[test]
    fn test_parent_from_side_selection() {
        let state = MenuState::SideSelection;
        assert_eq!(state.parent(), Some(MenuState::ModeSelection));
    }

    #[test]
    fn test_parent_from_difficulty_selection() {
        let state = MenuState::DifficultySelection { user_color: Color::White };
        assert_eq!(state.parent(), Some(MenuState::SideSelection));
    }

    #[test]
    fn test_parent_from_aivai_setup() {
        let state = MenuState::AIvAISetup(AIvAISetupState::default());
        assert_eq!(state.parent(), Some(MenuState::ModeSelection));
    }
}
