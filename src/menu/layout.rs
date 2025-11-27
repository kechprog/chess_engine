//! Menu layout and button coordinate system.
//!
//! This module provides the single source of truth for all menu button
//! coordinates. The same definitions are used for both rendering and hit detection.
//!
//! # Coordinate System
//!
//! Normalized Device Coordinates (NDC):
//! - x: -1.0 (left) to 1.0 (right)
//! - y: -1.0 (bottom) to 1.0 (top) in rendering
//! - Note: Screen Y is inverted (0 at top, increases downward)

use winit::dpi::PhysicalPosition;

/// A rectangle in NDC (Normalized Device Coordinates).
///
/// Used to define button positions. The same rect is used for both
/// rendering (vertices) and hit detection (contains check).
#[derive(Debug, Clone, Copy)]
pub struct ButtonRect {
    /// Left edge in NDC (-1.0 to 1.0)
    pub left: f32,
    /// Top edge in NDC (-1.0 to 1.0)
    pub top: f32,
    /// Width in NDC units
    pub width: f32,
    /// Height in NDC units
    pub height: f32,
}

impl ButtonRect {
    /// Create a new button rect.
    pub const fn new(left: f32, top: f32, width: f32, height: f32) -> Self {
        Self { left, top, width, height }
    }

    /// Get the right edge.
    pub fn right(&self) -> f32 {
        self.left + self.width
    }

    /// Get the bottom edge (lower Y value in NDC).
    pub fn bottom(&self) -> f32 {
        self.top - self.height
    }

    /// Check if a screen position is within this button.
    ///
    /// Converts screen coordinates to NDC and checks containment.
    pub fn contains(&self, pos: PhysicalPosition<f64>, window_size: (u32, u32)) -> bool {
        let (width, height) = window_size;
        if width == 0 || height == 0 {
            return false;
        }

        // Convert screen coords to NDC
        // Screen: (0,0) at top-left, Y increases downward
        // NDC: (-1,-1) at bottom-left, Y increases upward
        let norm_x = (pos.x / width as f64) * 2.0 - 1.0;
        let norm_y = 1.0 - (pos.y / height as f64) * 2.0; // Flip Y

        // Check containment
        norm_x >= self.left as f64
            && norm_x <= self.right() as f64
            && norm_y <= self.top as f64
            && norm_y >= self.bottom() as f64
    }

    /// Generate the 4 vertex positions for rendering this button as a quad.
    ///
    /// Returns positions in order: top-left, top-right, bottom-left, bottom-right.
    /// This matches the expected order for indexed drawing with standard indices.
    pub fn positions(&self) -> [[f32; 2]; 4] {
        let right = self.right();
        let bottom = self.bottom();
        [
            [self.left, self.top],    // top-left
            [right, self.top],         // top-right
            [self.left, bottom],       // bottom-left
            [right, bottom],           // bottom-right
        ]
    }
}

// ============================================================================
// Button Layout Definitions - Single Source of Truth
// ============================================================================

/// Main menu button layouts (PvP, PvAI, AIvAI)
pub mod main_menu {
    use super::ButtonRect;

    pub const PVP: ButtonRect = ButtonRect::new(-0.5, 0.45, 1.0, 0.2);
    pub const PVAI: ButtonRect = ButtonRect::new(-0.5, 0.1, 1.0, 0.2);
    pub const AIVAI: ButtonRect = ButtonRect::new(-0.5, -0.25, 1.0, 0.2);

    /// Get all main menu buttons.
    pub fn buttons() -> [ButtonRect; 3] {
        [PVP, PVAI, AIVAI]
    }
}

/// Side selection buttons (Play as White, Play as Black)
pub mod side_selection {
    use super::ButtonRect;

    pub const WHITE: ButtonRect = ButtonRect::new(-0.5, 0.3, 1.0, 0.2);
    pub const BLACK: ButtonRect = ButtonRect::new(-0.5, -0.1, 1.0, 0.2);

    /// Get all side selection buttons.
    pub fn buttons() -> [ButtonRect; 2] {
        [WHITE, BLACK]
    }
}

/// Difficulty selection buttons
pub mod difficulty {
    use super::ButtonRect;

    const BUTTON_WIDTH: f32 = 0.18;
    const BUTTON_HEIGHT: f32 = 0.12;
    const START_X: f32 = -0.45;
    const SPACING: f32 = 0.05;

    /// Y position for white AI difficulty row
    const WHITE_Y: f32 = 0.27;
    /// Y position for black AI difficulty row
    const BLACK_Y: f32 = -0.23;
    /// Y position for single difficulty selection (PvAI mode)
    const SINGLE_Y: f32 = 0.06;

    /// Get the rect for a difficulty button at given index.
    fn button_at(index: usize, y_top: f32) -> ButtonRect {
        let x = START_X + (index as f32) * (BUTTON_WIDTH + SPACING);
        ButtonRect::new(x, y_top, BUTTON_WIDTH, BUTTON_HEIGHT)
    }

    /// White AI difficulty buttons (Easy, Medium, Hard, Expert)
    pub fn white_buttons() -> [ButtonRect; 4] {
        [
            button_at(0, WHITE_Y),
            button_at(1, WHITE_Y),
            button_at(2, WHITE_Y),
            button_at(3, WHITE_Y),
        ]
    }

    /// Black AI difficulty buttons (Easy, Medium, Hard, Expert)
    pub fn black_buttons() -> [ButtonRect; 4] {
        [
            button_at(0, BLACK_Y),
            button_at(1, BLACK_Y),
            button_at(2, BLACK_Y),
            button_at(3, BLACK_Y),
        ]
    }

    /// Single row difficulty buttons for PvAI mode
    pub fn single_buttons() -> [ButtonRect; 4] {
        [
            button_at(0, SINGLE_Y),
            button_at(1, SINGLE_Y),
            button_at(2, SINGLE_Y),
            button_at(3, SINGLE_Y),
        ]
    }

    /// Start button for AIvAI setup
    pub const START: ButtonRect = ButtonRect::new(-0.3, -0.6, 0.6, 0.15);
}

/// Button colors
pub mod colors {
    /// PvP button color (greenish)
    pub const PVP: [f32; 4] = [0.5, 0.7, 0.5, 1.0];
    /// PvAI button color (blueish)
    pub const PVAI: [f32; 4] = [0.5, 0.6, 0.8, 1.0];
    /// AIvAI button color (purplish)
    pub const AIVAI: [f32; 4] = [0.6, 0.5, 0.7, 1.0];

    /// Play as White button (light)
    pub const SIDE_WHITE: [f32; 4] = [0.85, 0.85, 0.8, 1.0];
    /// Play as Black button (dark)
    pub const SIDE_BLACK: [f32; 4] = [0.3, 0.3, 0.35, 1.0];

    /// White AI difficulty buttons
    pub mod white_ai {
        pub const NORMAL: [f32; 4] = [0.35, 0.4, 0.35, 1.0];
        pub const SELECTED: [f32; 4] = [0.5, 0.7, 0.5, 1.0];
        pub const PRESSED: [f32; 4] = [0.4, 0.55, 0.4, 1.0];
    }

    /// Black AI difficulty buttons
    pub mod black_ai {
        pub const NORMAL: [f32; 4] = [0.35, 0.35, 0.4, 1.0];
        pub const SELECTED: [f32; 4] = [0.5, 0.5, 0.7, 1.0];
        pub const PRESSED: [f32; 4] = [0.4, 0.4, 0.55, 1.0];
    }

    /// Start button
    pub const START: [f32; 4] = [0.4, 0.55, 0.4, 1.0];

    /// Background color
    pub const BACKGROUND: [f32; 4] = [0.15, 0.15, 0.18, 1.0];
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_button_rect_contains_center() {
        let btn = ButtonRect::new(-0.5, 0.5, 1.0, 1.0);
        // Center of 800x800 window should be in center of NDC (0, 0)
        let center = PhysicalPosition::new(400.0, 400.0);
        assert!(btn.contains(center, (800, 800)));
    }

    #[test]
    fn test_button_rect_contains_corners() {
        let btn = ButtonRect::new(-0.5, 0.5, 1.0, 1.0);
        let size = (800, 800);

        // Top-left of button (-0.5, 0.5) -> screen (200, 200)
        let top_left = PhysicalPosition::new(200.0, 200.0);
        assert!(btn.contains(top_left, size));

        // Outside button
        let outside = PhysicalPosition::new(100.0, 100.0);
        assert!(!btn.contains(outside, size));
    }

    #[test]
    fn test_button_rect_positions() {
        let btn = ButtonRect::new(-0.5, 0.5, 1.0, 1.0);
        let positions = btn.positions();

        assert_eq!(positions[0], [-0.5, 0.5]);   // top-left
        assert_eq!(positions[1], [0.5, 0.5]);    // top-right
        assert_eq!(positions[2], [-0.5, -0.5]);  // bottom-left
        assert_eq!(positions[3], [0.5, -0.5]);   // bottom-right
    }

    #[test]
    fn test_main_menu_buttons_dont_overlap() {
        let buttons = main_menu::buttons();
        for i in 0..buttons.len() {
            for j in (i + 1)..buttons.len() {
                let a = &buttons[i];
                let b = &buttons[j];
                // Check vertical separation (buttons are stacked vertically)
                assert!(a.bottom() > b.top || b.bottom() > a.top,
                    "Buttons {} and {} overlap", i, j);
            }
        }
    }

    #[test]
    fn test_difficulty_buttons_dont_overlap() {
        let white = difficulty::white_buttons();
        for i in 0..white.len() {
            for j in (i + 1)..white.len() {
                let a = &white[i];
                let b = &white[j];
                // Check horizontal separation (buttons are side by side)
                assert!(a.right() < b.left || b.right() < a.left,
                    "White difficulty buttons {} and {} overlap", i, j);
            }
        }
    }
}
