use crate::agent::player::GameResult;
use crate::game_repr::{Color, Move, Piece, Position};
use crate::renderer::Renderer;
use winit::dpi::PhysicalPosition;
use smallvec::SmallVec;

/// Board component: Shared state object managing chess position, rendering, and UI interactions.
///
/// The Board wraps the core game logic (Position) and rendering capabilities (Renderer) into
/// a single cohesive interface. It maintains UI state like piece selection and caches legal
/// moves for performance.
///
/// # Architecture
///
/// The Board acts as the central state object in the new architecture:
/// - Owned by the Orchestrator and shared with Players via `Arc<RefCell<Board>>`
/// - Encapsulates Position (game state) and Renderer (graphics)
/// - Provides both read-only queries and mutable state changes
/// - Caches legal moves when a piece is selected to avoid recomputation
///
/// # Thread Safety and Borrowing
///
/// This component is **NOT thread-safe** and should only be accessed from the main thread.
/// When wrapped in `Arc<RefCell<>>`, be aware that:
///
/// - Simultaneous mutable and immutable borrows will **panic** at runtime
/// - Keep RefCell borrows as short-lived as possible to avoid contention
/// - Use the pattern: borrow, extract data, drop borrow, then call methods that may borrow again
///
/// ```rust,ignore
/// // CORRECT: Short-lived borrow
/// let mouse_pos = self.board.borrow().mouse_pos();
/// // borrow is dropped here
/// let mut board = self.board.borrow_mut();
/// board.handle_click(mouse_pos);
///
/// // WRONG: Holding borrow while calling methods that borrow
/// let board = self.board.borrow();
/// self.board.borrow_mut().execute_move(mv); // PANIC! Already borrowed
/// ```
///
/// # Usage
///
/// ```rust,ignore
/// // Create new board with default position
/// let board = Board::new(Box::new(WgpuRenderer::new(window)));
///
/// // Or from FEN string
/// let board = Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR", renderer);
///
/// // Query game state
/// let piece = board.piece_at(0);
/// let legal_moves = board.legal_moves_for(Color::White);
///
/// // Handle user interaction
/// board.update_mouse_pos(mouse_position);
/// if let Some(tile) = board.handle_click(mouse_position) {
///     board.set_selected_tile(Some(tile));
/// }
///
/// // Execute moves
/// if board.is_legal_move(mv) {
///     board.execute_move(mv);
/// }
///
/// // Render
/// board.draw();
/// ```
pub struct Board {
    /// The current chess position (game state)
    position: Position,

    /// Renderer responsible for drawing the board
    renderer: Box<dyn Renderer>,

    /// Currently selected tile (0-63), if any
    selected_tile: Option<u8>,

    /// Cached legal moves for the currently selected piece
    /// Updated automatically when selected_tile changes
    legal_moves_cache: SmallVec<[Move; 64]>,

    /// Point of view - which color is shown at the bottom of the board
    pov: Color,

    /// Last known mouse position (for event handling)
    mouse_pos: PhysicalPosition<f64>,
}

impl Board {
    /// Create a new board with the starting chess position.
    ///
    /// # Arguments
    ///
    /// * `renderer` - Boxed renderer implementation for drawing the board
    ///
    /// # Returns
    ///
    /// A new Board with the standard starting position, White's POV, and no selection.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let renderer = Box::new(WgpuRenderer::new(window));
    /// let board = Board::new(renderer);
    /// ```
    pub fn new(renderer: Box<dyn Renderer>) -> Self {
        Self {
            position: Position::default(),
            renderer,
            selected_tile: None,
            legal_moves_cache: SmallVec::new(),
            pov: Color::White,
            mouse_pos: PhysicalPosition::new(0.0, 0.0),
        }
    }

    /// Create a board from a FEN (Forsyth-Edwards Notation) string.
    ///
    /// # Arguments
    ///
    /// * `fen` - FEN string describing the position
    /// * `renderer` - Boxed renderer implementation for drawing the board
    ///
    /// # Returns
    ///
    /// A new Board with the position from the FEN string.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let board = Board::from_fen(
    ///     "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
    ///     renderer
    /// );
    /// ```
    pub fn from_fen(fen: &str, renderer: Box<dyn Renderer>) -> Self {
        Self {
            position: Position::from_fen(fen),
            renderer,
            selected_tile: None,
            legal_moves_cache: SmallVec::new(),
            pov: Color::White,
            mouse_pos: PhysicalPosition::new(0.0, 0.0),
        }
    }

    // ===========================
    // Game State Access (Read-Only)
    // ===========================

    /// Get a reference to the current position.
    ///
    /// This provides read-only access to the game state. Use this when you need
    /// to query the position without modifying it.
    ///
    /// # Returns
    ///
    /// A reference to the current Position.
    pub fn position(&self) -> &Position {
        &self.position
    }

    /// Get the piece at a specific square.
    ///
    /// # Arguments
    ///
    /// * `square` - Square index (0-63), where 0=a1, 63=h8
    ///
    /// # Returns
    ///
    /// The piece at the given square, or a None piece if the square is empty.
    ///
    /// # Panics
    ///
    /// Panics if `square` is >= 64 (out of bounds).
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let piece = board.piece_at(0); // Get piece at a1
    /// if !piece.is_none() {
    ///     println!("Found a {:?}", piece);
    /// }
    /// ```
    pub fn piece_at(&self, square: u8) -> Piece {
        assert!(square < 64, "Square index {} out of bounds (0-63)", square);
        self.position.position[square as usize]
    }

    /// Get all legal moves for a specific color.
    ///
    /// This computes all legal moves for all pieces of the given color.
    /// Note: This can be expensive for complex positions. Consider caching
    /// or using `legal_moves_for_selection()` when a piece is selected.
    ///
    /// # Arguments
    ///
    /// * `color` - The color whose legal moves to compute
    ///
    /// # Returns
    ///
    /// A vector of all legal moves for the given color.
    pub fn legal_moves_for(&self, color: Color) -> SmallVec<[Move; 64]> {
        let mut all_moves = SmallVec::new();

        for idx in 0..64 {
            let piece = self.position.position[idx];
            if !piece.is_none() && piece.color == color {
                let moves = self.position.legal_moves(idx);
                all_moves.extend(moves);
            }
        }

        all_moves
    }

    /// Check if a move is legal in the current position.
    ///
    /// # Arguments
    ///
    /// * `mv` - The move to validate
    ///
    /// # Returns
    ///
    /// `true` if the move is legal, `false` otherwise.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// if board.is_legal_move(mv) {
    ///     board.execute_move(mv);
    /// }
    /// ```
    pub fn is_legal_move(&self, mv: Move) -> bool {
        let from = mv._from();
        let legal_moves = self.position.legal_moves(from);
        legal_moves.contains(&mv)
    }

    /// Execute a move, updating the position.
    ///
    /// This modifies the board state by applying the given move. The move should
    /// be validated with `is_legal_move()` before calling this method.
    ///
    /// # Arguments
    ///
    /// * `mv` - The move to execute
    ///
    /// # Side Effects
    ///
    /// - Updates the position
    /// - Clears the selected tile
    /// - Clears the legal moves cache
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// if board.is_legal_move(mv) {
    ///     board.execute_move(mv);
    /// }
    /// ```
    pub fn execute_move(&mut self, mv: Move) {
        self.position.mk_move(mv);
        // Clear selection state after move
        self.selected_tile = None;
        self.legal_moves_cache.clear();
    }

    /// Check if the given color is in checkmate.
    ///
    /// # Arguments
    ///
    /// * `color` - The color to check
    ///
    /// # Returns
    ///
    /// `true` if the color is in checkmate, `false` otherwise.
    pub fn is_checkmate(&self, color: Color) -> bool {
        self.position.is_checkmate(color)
    }

    /// Check if the given color is in stalemate.
    ///
    /// # Arguments
    ///
    /// * `color` - The color to check
    ///
    /// # Returns
    ///
    /// `true` if the color is in stalemate, `false` otherwise.
    pub fn is_stalemate(&self, color: Color) -> bool {
        self.position.is_stalemate(color)
    }

    /// Check if the given color is in check.
    ///
    /// # Arguments
    ///
    /// * `color` - The color to check
    ///
    /// # Returns
    ///
    /// `true` if the color is in check, `false` otherwise.
    pub fn is_in_check(&self, color: Color) -> bool {
        self.position.is_in_check(color)
    }

    /// Reset the board to a new position from FEN string.
    ///
    /// # Arguments
    ///
    /// * `fen` - FEN string for the position, or empty string for default starting position
    ///
    /// # Side Effects
    ///
    /// - Clears the selected tile
    /// - Clears the legal moves cache
    /// - Resets the position
    pub fn reset_position(&mut self, fen: &str) {
        self.position = if fen.is_empty() {
            Position::default()
        } else {
            Position::from_fen(fen)
        };
        self.selected_tile = None;
        self.legal_moves_cache.clear();
    }

    // ===========================
    // UI Interaction
    // ===========================

    /// Handle a mouse click, converting screen coordinates to a board square.
    ///
    /// This method uses the renderer's `coord_to_tile()` method to convert
    /// the current mouse position to a board square index.
    ///
    /// # Arguments
    ///
    /// * `pos` - Physical mouse position in pixels
    ///
    /// # Returns
    ///
    /// * `Some(u8)` - The square index (0-63) if the click was on the board
    /// * `None` - If the click was outside the board
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// if let Some(tile) = board.handle_click(mouse_position) {
    ///     // User clicked on the board at the given tile
    ///     board.set_selected_tile(Some(tile));
    /// } else {
    ///     // User clicked outside the board
    ///     board.set_selected_tile(None);
    /// }
    /// ```
    pub fn handle_click(&mut self, pos: PhysicalPosition<f64>) -> Option<u8> {
        self.renderer.coord_to_tile(pos, self.pov)
    }

    /// Get the currently selected tile.
    ///
    /// # Returns
    ///
    /// The selected tile index (0-63), or None if no tile is selected.
    pub fn selected_tile(&self) -> Option<u8> {
        self.selected_tile
    }

    /// Set the selected tile and update the legal moves cache.
    ///
    /// When a tile is selected, this method automatically computes and caches
    /// the legal moves for the piece on that tile. This improves performance
    /// by avoiding repeated move generation.
    ///
    /// # Arguments
    ///
    /// * `tile` - The tile to select (0-63), or None to clear selection
    ///
    /// # Side Effects
    ///
    /// - Updates `selected_tile`
    /// - Updates `legal_moves_cache` with moves for the selected piece
    /// - If `tile` is None, clears both the selection and cache
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// // Select a tile
    /// board.set_selected_tile(Some(12)); // e2
    ///
    /// // Now legal_moves_for_selection() returns cached moves for e2
    /// let moves = board.legal_moves_for_selection();
    ///
    /// // Clear selection
    /// board.set_selected_tile(None);
    /// ```
    pub fn set_selected_tile(&mut self, tile: Option<u8>) {
        self.selected_tile = tile;

        // Update legal moves cache
        if let Some(tile_idx) = tile {
            self.legal_moves_cache = self.position.legal_moves(tile_idx as usize);
        } else {
            self.legal_moves_cache.clear();
        }
    }

    /// Get the cached legal moves for the currently selected piece.
    ///
    /// This returns a slice of the cached legal moves, which were computed
    /// when `set_selected_tile()` was called. This is much more efficient
    /// than recomputing moves on every query.
    ///
    /// # Returns
    ///
    /// A slice of legal moves for the selected piece. Empty if no piece is selected.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// board.set_selected_tile(Some(12)); // Select e2
    /// for mv in board.legal_moves_for_selection() {
    ///     println!("Can move to {}", mv._to());
    /// }
    /// ```
    pub fn legal_moves_for_selection(&self) -> &[Move] {
        &self.legal_moves_cache
    }

    /// Update the cached mouse position.
    ///
    /// This should be called whenever the mouse moves, so that `handle_click()`
    /// can use the current position.
    ///
    /// # Arguments
    ///
    /// * `pos` - The new mouse position in physical pixels
    pub fn update_mouse_pos(&mut self, pos: PhysicalPosition<f64>) {
        self.mouse_pos = pos;
    }

    /// Get the current mouse position.
    ///
    /// # Returns
    ///
    /// The last known mouse position in physical pixels.
    pub fn mouse_pos(&self) -> PhysicalPosition<f64> {
        self.mouse_pos
    }

    // ===========================
    // Rendering
    // ===========================

    /// Draw the current board state.
    ///
    /// This calls the renderer's `draw_position()` method with the current
    /// position, selected tile, and point of view.
    ///
    /// # Side Effects
    ///
    /// Renders the board to the screen via the underlying renderer.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// // In your render loop
    /// board.draw();
    /// ```
    pub fn draw(&mut self) {
        self.renderer.draw_position(&self.position, self.selected_tile, self.pov);
    }

    /// Draw the menu screen.
    ///
    /// # Arguments
    ///
    /// * `show_coming_soon` - If true, display "Coming Soon!" overlay instead of menu buttons
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// // When in menu mode
    /// board.draw_menu(false);
    /// ```
    pub fn draw_menu(&mut self, show_coming_soon: bool) {
        self.renderer.draw_menu(show_coming_soon);
    }

    /// Check if a screen coordinate is within a button's bounds (for menu).
    ///
    /// # Arguments
    ///
    /// * `coords` - Physical position in pixels
    /// * `button_index` - Which button to check (0 = PvP, 1 = PvAI)
    ///
    /// # Returns
    ///
    /// * `true` if the coordinate is within the button bounds
    pub fn is_coord_in_button(&self, coords: PhysicalPosition<f64>, button_index: usize) -> bool {
        self.renderer.is_coord_in_button(coords, button_index)
    }

    /// Set the point of view for rendering.
    ///
    /// This determines which color is shown at the bottom of the board.
    /// Typically set to the current player's color in a PvP game.
    ///
    /// # Arguments
    ///
    /// * `pov` - The color to show at the bottom of the board
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// // Show board from Black's perspective
    /// board.set_pov(Color::Black);
    /// ```
    pub fn set_pov(&mut self, pov: Color) {
        self.pov = pov;
    }

    /// Get the current point of view.
    ///
    /// # Returns
    ///
    /// The color currently shown at the bottom of the board.
    pub fn pov(&self) -> Color {
        self.pov
    }

    /// Handle window resize events.
    ///
    /// This delegates to the renderer's `resize()` method to update any
    /// size-dependent rendering state.
    ///
    /// # Arguments
    ///
    /// * `new_size` - The new window dimensions (width, height) in pixels
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// // In your WindowEvent::Resized handler
    /// board.resize((width, height));
    /// ```
    pub fn resize(&mut self, new_size: (u32, u32)) {
        self.renderer.resize(new_size);
    }

    /// Draw the game end overlay.
    ///
    /// This draws the current board position with an overlay showing the game result.
    ///
    /// # Arguments
    ///
    /// * `result` - The game result to display
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// // When game ends
    /// board.draw_game_end(GameResult::WhiteWins);
    /// ```
    pub fn draw_game_end(&mut self, result: GameResult) {
        self.renderer.draw_game_end(&self.position, self.selected_tile, self.pov, result);
    }

    /// Draw the promotion piece selection overlay.
    ///
    /// Shows 4 piece options (Queen, Rook, Bishop, Knight) for the user to choose from
    /// when promoting a pawn.
    ///
    /// # Arguments
    ///
    /// * `promoting_color` - The color of the pawn being promoted
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// // When a pawn reaches the back rank
    /// board.draw_promotion_selection(Color::White);
    /// ```
    pub fn draw_promotion_selection(&mut self, promoting_color: Color) {
        self.renderer.draw_promotion_selection(&self.position, self.selected_tile, self.pov, promoting_color);
    }

    /// Get the promotion piece type at the given screen coordinates.
    ///
    /// Used to detect which piece the user clicked on during promotion selection.
    ///
    /// # Arguments
    ///
    /// * `coords` - Physical position in pixels
    ///
    /// # Returns
    ///
    /// * `Some(Type)` - The piece type if clicked on a piece option
    /// * `None` - If not over any piece option
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// if let Some(piece_type) = board.get_promotion_piece_at_coords(mouse_pos) {
    ///     // User clicked on a promotion piece
    /// }
    /// ```
    pub fn get_promotion_piece_at_coords(&self, coords: PhysicalPosition<f64>) -> Option<crate::game_repr::Type> {
        self.renderer.get_promotion_piece_at_coords(coords)
    }

    /// Draw the side selection screen for PvAI mode.
    ///
    /// Shows buttons for the player to choose whether to play as White or Black.
    pub fn draw_side_selection(&mut self) {
        self.renderer.draw_side_selection();
    }

    /// Check if given coordinates are inside a side selection button.
    ///
    /// # Arguments
    ///
    /// * `coords` - Physical position in pixels
    /// * `button_index` - Which button to check (0 = Play as White, 1 = Play as Black)
    ///
    /// # Returns
    ///
    /// `true` if the coordinates are within the button's bounds
    pub fn is_coord_in_side_button(&self, coords: PhysicalPosition<f64>, button_index: usize) -> bool {
        self.renderer.is_coord_in_side_button(coords, button_index)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game_repr::{MoveType, Type};
    use crate::renderer::Renderer;

    // Mock renderer for testing
    struct MockRenderer;

    impl Renderer for MockRenderer {
        fn draw_position(&mut self, _position: &Position, _selected_tile: Option<u8>, _pov: Color) {
            // No-op for tests
        }

        fn coord_to_tile(&self, _coords: PhysicalPosition<f64>, _pov: Color) -> Option<u8> {
            Some(0) // Always return a1 for simplicity
        }

        fn resize(&mut self, _new_size: (u32, u32)) {
            // No-op for tests
        }

        fn draw_menu(&mut self, _show_coming_soon: bool) {
            // No-op for tests
        }

        fn is_coord_in_button(&self, _coords: PhysicalPosition<f64>, _button_index: usize) -> bool {
            false // Always return false for simplicity
        }

        fn draw_game_end(&mut self, _position: &Position, _selected_tile: Option<u8>, _pov: Color, _result: GameResult) {
            // No-op for tests
        }

        fn draw_promotion_selection(&mut self, _position: &Position, _selected_tile: Option<u8>, _pov: Color, _promoting_color: Color) {
            // No-op for tests
        }

        fn get_promotion_piece_at_coords(&self, _coords: PhysicalPosition<f64>) -> Option<Type> {
            None
        }

        fn draw_side_selection(&mut self) {
            // No-op for tests
        }

        fn is_coord_in_side_button(&self, _coords: PhysicalPosition<f64>, _button_index: usize) -> bool {
            false
        }
    }

    #[test]
    fn test_board_new() {
        let board = Board::new(Box::new(MockRenderer));

        // Check starting position has pieces
        assert_eq!(board.piece_at(0).piece_type, Type::Rook);
        assert_eq!(board.piece_at(0).color, Color::White);
        assert_eq!(board.piece_at(4).piece_type, Type::King);
        assert_eq!(board.piece_at(4).color, Color::White);

        // Check initial state
        assert_eq!(board.selected_tile(), None);
        assert_eq!(board.pov(), Color::White);
        assert_eq!(board.legal_moves_for_selection().len(), 0);
    }

    #[test]
    fn test_board_from_fen() {
        let board = Board::from_fen(
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR",
            Box::new(MockRenderer)
        );

        // Verify starting position
        assert_eq!(board.piece_at(0).piece_type, Type::Rook);
        assert_eq!(board.piece_at(56).piece_type, Type::Rook);
        assert_eq!(board.piece_at(56).color, Color::Black);
    }

    #[test]
    fn test_piece_at() {
        let board = Board::new(Box::new(MockRenderer));

        // White rook at a1
        let piece = board.piece_at(0);
        assert_eq!(piece.piece_type, Type::Rook);
        assert_eq!(piece.color, Color::White);

        // Empty square
        let empty = board.piece_at(32);
        assert!(empty.is_none());
    }

    #[test]
    fn test_selected_tile() {
        let mut board = Board::new(Box::new(MockRenderer));

        // Initially no selection
        assert_eq!(board.selected_tile(), None);
        assert_eq!(board.legal_moves_for_selection().len(), 0);

        // Select a pawn at e2 (index 12)
        board.set_selected_tile(Some(12));
        assert_eq!(board.selected_tile(), Some(12));

        // Should have cached legal moves for the pawn
        let moves = board.legal_moves_for_selection();
        assert!(moves.len() > 0);

        // Clear selection
        board.set_selected_tile(None);
        assert_eq!(board.selected_tile(), None);
        assert_eq!(board.legal_moves_for_selection().len(), 0);
    }

    #[test]
    fn test_legal_moves_for_white() {
        let board = Board::new(Box::new(MockRenderer));

        let white_moves = board.legal_moves_for(Color::White);

        // Starting position should have 20 legal moves for white
        // (16 pawn moves: 8 pawns * 2 moves each, + 4 knight moves: 2 knights * 2 moves each)
        assert_eq!(white_moves.len(), 20);
    }

    #[test]
    fn test_execute_move() {
        let mut board = Board::new(Box::new(MockRenderer));

        // Select pawn at e2 (index 12)
        board.set_selected_tile(Some(12));
        assert!(board.legal_moves_for_selection().len() > 0);

        // Create move e2-e4
        let mv = Move::new(12, 28, MoveType::Normal);

        // Execute the move
        board.execute_move(mv);

        // Verify the move was made
        assert!(board.piece_at(12).is_none()); // e2 is now empty
        assert_eq!(board.piece_at(28).piece_type, Type::Pawn); // e4 has the pawn

        // Selection should be cleared
        assert_eq!(board.selected_tile(), None);
        assert_eq!(board.legal_moves_for_selection().len(), 0);
    }

    #[test]
    fn test_pov() {
        let mut board = Board::new(Box::new(MockRenderer));

        // Default is White
        assert_eq!(board.pov(), Color::White);

        // Change to Black
        board.set_pov(Color::Black);
        assert_eq!(board.pov(), Color::Black);
    }

    #[test]
    fn test_is_legal_move() {
        let board = Board::new(Box::new(MockRenderer));

        // e2-e4 should be legal
        let legal_move = Move::new(12, 28, MoveType::Normal);
        assert!(board.is_legal_move(legal_move));

        // e2-e5 should be illegal (pawn can't move 3 squares)
        let illegal_move = Move::new(12, 36, MoveType::Normal);
        assert!(!board.is_legal_move(illegal_move));
    }

    #[test]
    fn test_game_state_checks() {
        let board = Board::new(Box::new(MockRenderer));

        // Starting position should not be check, checkmate, or stalemate
        assert!(!board.is_in_check(Color::White));
        assert!(!board.is_checkmate(Color::White));
        assert!(!board.is_stalemate(Color::White));

        assert!(!board.is_in_check(Color::Black));
        assert!(!board.is_checkmate(Color::Black));
        assert!(!board.is_stalemate(Color::Black));
    }

    #[test]
    fn test_handle_click() {
        let mut board = Board::new(Box::new(MockRenderer));

        let pos = PhysicalPosition::new(100.0, 100.0);
        let tile = board.handle_click(pos);

        // Mock renderer always returns Some(0)
        assert_eq!(tile, Some(0));
    }

    #[test]
    fn test_update_mouse_pos() {
        let mut board = Board::new(Box::new(MockRenderer));

        let pos = PhysicalPosition::new(150.0, 200.0);
        board.update_mouse_pos(pos);

        // Can't directly test mouse_pos as it's private, but we can verify
        // it doesn't panic and state is preserved
        assert_eq!(board.selected_tile(), None);
    }
}
