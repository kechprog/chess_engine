//! Menu system module.
//!
//! This module provides a complete separation of menu functionality from the game logic.
//! The [`Menu`] struct handles all menu state, event processing, and produces a
//! [`GameConfig`] when the user has completed their selections.
//!
//! # Architecture
//!
//! The menu system is designed with clean separation:
//! - [`state`] - Menu state machine types
//! - [`config`] - Output types returned when menu completes
//! - [`layout`] - Button coordinates and hit detection
//! - [`renderer`] - MenuRenderer trait for drawing (TODO)
//!
//! # Example Flow
//!
//! ```text
//! ModeSelection
//!   ├─ PvP → Returns GameConfig::pvp() immediately
//!   ├─ PvAI → SideSelection
//!   │           ├─ White → DifficultySelection { White }
//!   │           └─ Black → DifficultySelection { Black }
//!   │                       └─ Easy/Med/Hard/Expert → Returns GameConfig::pvai()
//!   └─ AIvAI → AIvAISetup
//!               └─ Start → Returns GameConfig::aivai()
//! ```

pub mod config;
pub mod layout;
pub mod state;

use crate::agent::ai::Difficulty;
use crate::game_repr::Color;
use winit::dpi::PhysicalPosition;

// Re-export commonly used types
pub use config::{GameConfig, GameMode, PlayerConfig};
pub use layout::ButtonRect;
pub use state::{AIvAISetupState, MenuState};

/// The main Menu component.
///
/// Manages all menu state, handles user input, and produces a [`GameConfig`]
/// when the user completes their selections.
pub struct Menu {
    /// Current menu state
    state: MenuState,
    /// Last known mouse position
    mouse_pos: PhysicalPosition<f64>,
    /// Window size for coordinate conversion
    window_size: (u32, u32),
    /// Scale factor for WASM coordinate adjustment
    /// On native platforms this is typically 1.0, on WASM it reflects devicePixelRatio
    scale_factor: f64,
}

impl Menu {
    /// Create a new menu starting at mode selection.
    pub fn new() -> Self {
        Self {
            state: MenuState::default(),
            mouse_pos: PhysicalPosition::new(0.0, 0.0),
            window_size: (800, 800),
            scale_factor: 1.0,
        }
    }

    /// Get the current menu state.
    pub fn state(&self) -> &MenuState {
        &self.state
    }

    /// Update the mouse position.
    pub fn update_mouse_pos(&mut self, pos: PhysicalPosition<f64>) {
        self.mouse_pos = pos;
    }

    /// Update the window size (for coordinate conversion).
    pub fn update_window_size(&mut self, size: (u32, u32)) {
        self.window_size = size;
    }

    /// Update the scale factor (for WASM coordinate adjustment).
    ///
    /// On native platforms, this should be 1.0. On WASM, this reflects
    /// the browser's devicePixelRatio and is needed to correctly convert
    /// mouse coordinates to normalized device coordinates.
    pub fn update_scale_factor(&mut self, scale_factor: f64) {
        self.scale_factor = scale_factor;
    }

    /// Handle the Escape key - go back one level.
    ///
    /// Returns true if we went back, false if already at top level.
    pub fn go_back(&mut self) -> bool {
        if let Some(parent) = self.state.parent() {
            self.state = parent;
            true
        } else {
            false
        }
    }

    /// Handle a mouse click at the current mouse position.
    ///
    /// Returns `Some(GameConfig)` if the user has completed their selection
    /// and a game should start. Returns `None` if the menu should continue.
    pub fn handle_click(&mut self) -> Option<GameConfig> {
        // Apply scale factor adjustment for WASM
        // On WASM, mouse coordinates are in CSS pixels but window_size is in physical pixels
        // Dividing by scale_factor converts CSS pixels to match the coordinate system
        #[cfg(target_arch = "wasm32")]
        let pos = PhysicalPosition::new(
            self.mouse_pos.x / self.scale_factor,
            self.mouse_pos.y / self.scale_factor,
        );
        #[cfg(not(target_arch = "wasm32"))]
        let pos = self.mouse_pos;

        let size = self.window_size;

        match &mut self.state {
            MenuState::ModeSelection => {
                // Check main menu buttons
                let buttons = layout::main_menu::buttons();
                if buttons[0].contains(pos, size) {
                    // PvP - start immediately
                    return Some(GameConfig::pvp());
                } else if buttons[1].contains(pos, size) {
                    // PvAI - go to side selection
                    self.state = MenuState::SideSelection;
                } else if buttons[2].contains(pos, size) {
                    // AIvAI - go to AI setup
                    self.state = MenuState::AIvAISetup(AIvAISetupState::default());
                }
            }

            MenuState::SideSelection => {
                // Check side selection buttons
                let buttons = layout::side_selection::buttons();
                if buttons[0].contains(pos, size) {
                    // Play as White - go to difficulty selection
                    self.state = MenuState::DifficultySelection { user_color: Color::White };
                } else if buttons[1].contains(pos, size) {
                    // Play as Black - go to difficulty selection
                    self.state = MenuState::DifficultySelection { user_color: Color::Black };
                }
            }

            MenuState::DifficultySelection { user_color } => {
                // Check difficulty buttons (using single row layout)
                let buttons = layout::difficulty::single_buttons();
                let difficulties = [
                    Difficulty::Easy,
                    Difficulty::Medium,
                    Difficulty::Hard,
                    Difficulty::Expert,
                ];

                for (i, button) in buttons.iter().enumerate() {
                    if button.contains(pos, size) {
                        return Some(GameConfig::pvai(*user_color, difficulties[i]));
                    }
                }
            }

            MenuState::AIvAISetup(setup) => {
                // Check white difficulty buttons
                let white_buttons = layout::difficulty::white_buttons();
                let difficulties = [
                    Difficulty::Easy,
                    Difficulty::Medium,
                    Difficulty::Hard,
                    Difficulty::Expert,
                ];

                for (i, button) in white_buttons.iter().enumerate() {
                    if button.contains(pos, size) {
                        setup.white_difficulty = difficulties[i];
                        return None;
                    }
                }

                // Check black difficulty buttons
                let black_buttons = layout::difficulty::black_buttons();
                for (i, button) in black_buttons.iter().enumerate() {
                    if button.contains(pos, size) {
                        setup.black_difficulty = difficulties[i];
                        return None;
                    }
                }

                // Check start button
                if layout::difficulty::START.contains(pos, size) {
                    return Some(GameConfig::aivai(
                        setup.white_difficulty,
                        setup.black_difficulty,
                    ));
                }
            }
        }

        None
    }

    /// Reset the menu to the initial state.
    pub fn reset(&mut self) {
        self.state = MenuState::default();
    }
}

impl Default for Menu {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_menu_new() {
        let menu = Menu::new();
        assert!(matches!(menu.state(), MenuState::ModeSelection));
    }

    #[test]
    fn test_menu_go_back() {
        let mut menu = Menu::new();

        // Can't go back from mode selection
        assert!(!menu.go_back());

        // Simulate clicking PvAI to go to side selection
        menu.state = MenuState::SideSelection;
        assert!(menu.go_back());
        assert!(matches!(menu.state(), MenuState::ModeSelection));
    }

    #[test]
    fn test_menu_reset() {
        let mut menu = Menu::new();
        menu.state = MenuState::SideSelection;
        menu.reset();
        assert!(matches!(menu.state(), MenuState::ModeSelection));
    }

    #[test]
    fn test_menu_pvp_click() {
        let mut menu = Menu::new();
        menu.update_window_size((800, 800));

        // PvP button is at NDC (-0.5, 0.45) to (0.5, 0.25)
        // In 800x800 window: x 200-600, y 220-300 (inverted Y)
        menu.update_mouse_pos(PhysicalPosition::new(400.0, 260.0));

        let result = menu.handle_click();
        assert!(result.is_some());
        let config = result.unwrap();
        assert_eq!(config.mode, GameMode::PvP);
    }

    #[test]
    fn test_menu_pvai_flow() {
        let mut menu = Menu::new();
        menu.update_window_size((800, 800));

        // Click PvAI button (middle button)
        // NDC (-0.5, 0.1) to (0.5, -0.1) -> screen x 200-600, y 360-440
        menu.update_mouse_pos(PhysicalPosition::new(400.0, 400.0));
        let result = menu.handle_click();
        assert!(result.is_none());
        assert!(matches!(menu.state(), MenuState::SideSelection));

        // Click "Play as White" button
        // NDC (-0.5, 0.3) to (0.5, 0.1) -> screen y 280-360
        menu.update_mouse_pos(PhysicalPosition::new(400.0, 320.0));
        let result = menu.handle_click();
        assert!(result.is_none());
        assert!(matches!(menu.state(), MenuState::DifficultySelection { user_color: Color::White }));
    }
}
