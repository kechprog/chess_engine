//! Application lifecycle management and game mode coordination.
//!
//! This module contains the [`Orchestrator`] component, which serves as the root
//! coordinator for the chess application. It manages:
//! - Application state and game mode selection
//! - Player instantiation and turn management
//! - Game flow (move execution, end detection, menu transitions)
//! - Event routing to appropriate handlers
//!
//! # Architecture
//!
//! The Orchestrator follows a component-based design where:
//! - [`Board`] is shared state (via `Arc<RefCell<>>`) between orchestrator and players
//! - [`Player`] trait abstractions provide moves through a uniform interface
//! - [`GameMode`] enum drives state machine transitions
//!
//! # Example Flow
//!
//! ```text
//! [Menu] -> User selects PvP -> [Create Players] -> [Active Game]
//!   -> [Request Move] -> [Player provides move] -> [Execute Move]
//!   -> [Check End] -> [Switch Turn] -> [Request Move] ...
//! ```

use crate::agent::human_player::HumanPlayer;
use crate::agent::player::{GameResult, Player};
use crate::board::Board;
use crate::game_repr::{Color, Move};
use crate::renderer::wgpu_renderer::WgpuRenderer;
use std::cell::RefCell;
use std::sync::Arc;
use winit::event::WindowEvent;
use winit::window::Window;

/// Game mode enumeration representing the current application state.
///
/// The orchestrator uses this to determine which UI to display and how to
/// handle events. Transitions between modes are managed by [`Orchestrator::set_game_mode`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameMode {
    /// Mode selection screen - initial state on startup
    Menu,

    /// Player vs Player - two human players on same device
    PvP,

    /// Player vs AI - human vs computer (future implementation)
    PvAI,

    /// AI vs AI - watch two engines play (future implementation)
    AIvAI,

    /// Network multiplayer - play against remote opponent (future implementation)
    Online,
}

/// Root component managing application lifecycle and game coordination.
///
/// The Orchestrator is responsible for:
/// - Managing the application window
/// - Coordinating between shared [`Board`] state and [`Player`] instances
/// - Handling game mode selection and transitions
/// - Executing the game loop (request move, process move, check end, switch turn)
/// - Routing window events to appropriate handlers
///
/// # Shared State
///
/// The `board` field is wrapped in `Arc<RefCell<>>` to allow shared mutable access
/// between the orchestrator and player instances. This is safe because:
/// - All access happens on the main thread (winit event loop)
/// - RefCell provides runtime borrow checking
/// - Borrows are kept short-lived to avoid panics
///
/// # Game Flow
///
/// When a game is active (`game_active = true`), the orchestrator manages turns:
/// 1. Calls [`request_move`](Orchestrator::request_move) for current player
/// 2. Waits for player to provide move (blocks for human, computes for AI)
/// 3. Calls [`process_move`](Orchestrator::process_move) to execute and validate
/// 4. Calls [`check_game_end`](Orchestrator::check_game_end) to detect checkmate/stalemate
/// 5. Switches `current_turn` and repeats, or returns to menu if game ended
pub struct Orchestrator {
    /// Handle to the application window
    window: Arc<Window>,

    /// Shared reference to the board state and rendering
    /// Wrapped in Arc<RefCell<>> for shared mutable access
    board: Arc<RefCell<Board>>,

    /// Current game mode (Menu, PvP, PvAI, etc.)
    game_mode: GameMode,

    /// Active player instances for current game
    /// None when in Menu mode, Some when game is active
    /// Tuple represents (white_player, black_player)
    players: Option<(Box<dyn Player>, Box<dyn Player>)>,

    /// Whose turn it is (White or Black)
    /// Only meaningful when game_active is true
    current_turn: Color,

    /// Whether a game is currently in progress
    /// false in Menu mode or after game end
    game_active: bool,

    /// FEN string for starting position (used when starting a new game)
    /// Empty string means use default starting position
    starting_fen: String,

    /// Result of the game if it has ended
    /// None if game is in progress or in menu
    game_result: Option<GameResult>,
}

impl Orchestrator {
    /// Create a new orchestrator with the given window.
    ///
    /// Initializes in Menu mode with no active game. The board is created with
    /// a default starting position.
    ///
    /// # Arguments
    ///
    /// * `window` - Shared reference to the application window
    ///
    /// # Returns
    ///
    /// A new `Orchestrator` instance ready to handle events
    ///
    /// # Example
    ///
    /// ```ignore
    /// let window = Arc::new(event_loop.create_window(attrs)?);
    /// let renderer = WgpuRenderer::new(window.clone()).await;
    /// let orchestrator = Orchestrator::new(window, renderer);
    /// ```
    pub fn new(window: Arc<Window>, renderer: WgpuRenderer) -> Self {
        let board = Arc::new(RefCell::new(Board::new(Box::new(renderer))));

        // TODO: Add logging once log crate is added to dependencies
        // log::debug!("Orchestrator created in Menu mode");

        Self {
            window,
            board,
            game_mode: GameMode::Menu,
            players: None,
            current_turn: Color::White,
            game_active: false,
            starting_fen: String::new(),
            game_result: None,
        }
    }

    /// Handle window events, routing them based on current mode.
    ///
    /// Events are processed differently depending on the current [`GameMode`]:
    /// - **Menu mode**: Handle menu UI interactions (button clicks, navigation)
    /// - **Active game**: Delegate to current player's event handler
    /// - **All modes**: Handle window resize and redraw events
    ///
    /// # Arguments
    ///
    /// * `event` - The window event to process
    ///
    /// # Implementation Notes
    ///
    /// This method maintains short borrow lifetimes to avoid RefCell panics.
    /// Player event handlers may borrow the board, so we don't hold any borrows
    /// when calling player methods.
    pub fn handle_event(&mut self, event: WindowEvent) {
        match event {
            WindowEvent::RedrawRequested => {
                if self.game_mode == GameMode::Menu {
                    // Draw menu screen
                    self.board.borrow_mut().draw_menu(false);
                } else if self.game_mode == GameMode::PvAI && !self.game_active {
                    // Show "Coming Soon!" for PvAI
                    self.board.borrow_mut().draw_menu(true);
                } else if let Some(result) = self.game_result {
                    // Game has ended - draw board with game end overlay
                    self.board.borrow_mut().draw_game_end(result);
                } else {
                    // Draw normal game board
                    self.board.borrow_mut().draw();
                }
            }

            WindowEvent::Resized(_new_size) => {
                self.board.borrow_mut().resize((_new_size.width, _new_size.height));
                self.window.request_redraw();
            }

            WindowEvent::CloseRequested => {
                // Event loop will handle actual close
            }

            WindowEvent::MouseInput { state, button, .. } => {
                use winit::event::{ElementState, MouseButton};

                // Handle menu button clicks
                if self.game_mode == GameMode::Menu && state == ElementState::Pressed && button == MouseButton::Left {
                    let board = self.board.borrow();
                    let mouse_pos = board.mouse_pos();

                    if board.is_coord_in_button(mouse_pos, 0) {
                        // PvP button clicked
                        drop(board);
                        self.set_game_mode(GameMode::PvP);
                        self.start_game();
                    } else if board.is_coord_in_button(mouse_pos, 1) {
                        // PvAI button clicked - show "Coming Soon!"
                        drop(board);
                        self.set_game_mode(GameMode::PvAI);
                        self.window.request_redraw();
                    }
                } else if self.game_mode == GameMode::PvAI && !self.game_active && state == ElementState::Pressed {
                    // Click anywhere on "Coming Soon!" screen to return to menu
                    self.return_to_menu();
                } else if self.game_result.is_some() && state == ElementState::Pressed {
                    // Click anywhere on game end overlay to return to menu
                    self.return_to_menu();
                } else if self.game_active {
                    // Delegate to current player when game is active
                    if let Some((player1, player2)) = &mut self.players {
                        let current_player = match self.current_turn {
                            Color::White => player1,
                            Color::Black => player2,
                        };
                        current_player.handle_event(&event);

                        // Request redraw after handling event to show UI updates
                        self.window.request_redraw();

                        // Poll for move after each event
                        self.poll_current_player();
                    }
                }
            }

            WindowEvent::CursorMoved { position, .. } => {
                // Track mouse position for menu button detection
                self.board.borrow_mut().update_mouse_pos(position);

                // Delegate to player if game is active
                if self.game_active {
                    if let Some((player1, player2)) = &mut self.players {
                        let current_player = match self.current_turn {
                            Color::White => player1,
                            Color::Black => player2,
                        };
                        current_player.handle_event(&event);
                    }
                }
            }

            _ => {
                // Delegate other input events to current player when game is active
                if self.game_active {
                    if let Some((player1, player2)) = &mut self.players {
                        let current_player = match self.current_turn {
                            Color::White => player1,
                            Color::Black => player2,
                        };
                        current_player.handle_event(&event);

                        // Request redraw after handling event to show UI updates
                        self.window.request_redraw();

                        // Poll for move after each event
                        self.poll_current_player();
                    }
                }
            }
        }
    }

    /// Poll current player for a move and process it if available.
    ///
    /// This method is called after each event to check if the current player has
    /// a move ready. If so, it processes the move and updates the game state.
    ///
    /// # Control Flow
    ///
    /// 1. Get the current player based on `current_turn`
    /// 2. Call `player.get_move(current_turn)` (non-blocking for HumanPlayer)
    /// 3. If move is returned, call `process_move()` to execute it
    /// 4. Request window redraw to show the new position
    ///
    /// # Design Note
    ///
    /// This polling approach works well with the event-driven architecture:
    /// - Each event potentially changes the player's state (clicks, cursor movement)
    /// - After handling the event, we check if a complete move is ready
    /// - If ready, we process it immediately
    /// - If not ready, we continue processing events
    fn poll_current_player(&mut self) {
        if !self.game_active {
            return;
        }

        let (player1, player2) = match &mut self.players {
            Some(players) => players,
            None => return,
        };

        let current_player = match self.current_turn {
            Color::White => player1,
            Color::Black => player2,
        };

        // Try to get a move from the current player
        if let Some(mv) = current_player.get_move(self.current_turn) {
            self.process_move(mv);
            self.window.request_redraw();
        }
    }

    /// Switch to a new game mode.
    ///
    /// Transitions the orchestrator to the specified mode. If currently in an active
    /// game, that game will be abandoned. The new mode's UI will be displayed.
    ///
    /// # Arguments
    ///
    /// * `mode` - The game mode to switch to
    ///
    /// # State Changes
    ///
    /// - Sets `game_mode` to the new mode
    /// - Sets `game_active` to false
    /// - Clears `players` (will be recreated on [`start_game`](Self::start_game))
    /// - Requests window redraw to show new mode's UI
    ///
    /// # Example
    ///
    /// ```ignore
    /// orchestrator.set_game_mode(GameMode::PvP);
    /// orchestrator.start_game();
    /// ```
    pub fn set_game_mode(&mut self, mode: GameMode) {
        // Clean up any active game
        if self.game_active {
            self.game_active = false;
        }

        self.game_mode = mode;
        self.players = None;

        // Update display for new mode
        self.window.request_redraw();
    }

    /// Get a reference to the FEN string for starting position.
    ///
    /// # Returns
    ///
    /// A reference to the FEN string. Empty string means use default position.
    pub fn starting_fen(&self) -> &str {
        &self.starting_fen
    }

    /// Set the FEN string for starting position.
    ///
    /// This FEN will be used when start_game() is called.
    ///
    /// # Arguments
    ///
    /// * `fen` - FEN string describing the starting position, or empty for default
    pub fn set_starting_fen(&mut self, fen: String) {
        self.starting_fen = fen;
    }

    /// Get a mutable reference to the starting FEN string.
    ///
    /// Used for direct manipulation (e.g., text input).
    pub fn starting_fen_mut(&mut self) -> &mut String {
        &mut self.starting_fen
    }

    /// Get whether the game is currently active.
    pub fn is_game_active(&self) -> bool {
        self.game_active
    }

    /// Get the current game mode.
    pub fn game_mode(&self) -> GameMode {
        self.game_mode
    }

    /// Start a game with the current game mode.
    ///
    /// Creates the appropriate player instances based on [`game_mode`](Self::game_mode)
    /// and initializes game state. Must not be called in Menu mode.
    ///
    /// # Player Creation
    ///
    /// - **PvP**: Creates two `HumanPlayer` instances
    /// - **PvAI**: Creates `HumanPlayer` and `AIPlayer` (future)
    /// - **AIvAI**: Creates two `AIPlayer` instances (future)
    /// - **Online**: Creates `HumanPlayer` and `NetworkPlayer` (future)
    ///
    /// # State Changes
    ///
    /// - Creates and stores player instances in `players`
    /// - Sets `current_turn` to White (games always start with white)
    /// - Sets `game_active` to true
    /// - Calls [`request_move`](Self::request_move) to begin game loop
    ///
    /// # Panics
    ///
    /// Panics if called when `game_mode` is `Menu`.
    pub fn start_game(&mut self) {
        assert_ne!(self.game_mode, GameMode::Menu, "Cannot start game in Menu mode");

        // TODO: Create players based on game_mode
        // For now, just set game as active
        match self.game_mode {
            GameMode::Menu => unreachable!("Already checked above"),

            GameMode::PvP => {
                let player1 = Box::new(HumanPlayer::new(self.board.clone(), "White".to_string()));
                let player2 = Box::new(HumanPlayer::new(self.board.clone(), "Black".to_string()));
                self.players = Some((player1, player2));
            }

            GameMode::PvAI => {
                // TODO: Future implementation
            }

            GameMode::AIvAI => {
                // TODO: Future implementation
            }

            GameMode::Online => {
                // TODO: Future implementation
            }
        }

        // Initialize game state
        self.current_turn = Color::White;
        self.game_active = true;

        // Reset board to starting position (using FEN if provided)
        {
            let mut board = self.board.borrow_mut();
            board.reset_position(&self.starting_fen);
            board.set_pov(Color::White);
        }

        // Request initial redraw
        self.window.request_redraw();

        // Note: We don't call request_move() anymore - the polling model
        // will automatically detect when a move is ready after events
    }

    /// Request the next move from the current player.
    ///
    /// Polls the current player's [`get_move`](Player::get_move) method. This may block:
    /// - For human players: blocks until user makes a move via UI
    /// - For AI players: blocks during move computation
    /// - For network players: blocks waiting for network response
    ///
    /// The event loop continues to process events while waiting, allowing
    /// human players to interact with the board.
    ///
    /// # Control Flow
    ///
    /// 1. Determines which player's turn it is
    /// 2. Calls `player.get_move(current_turn)`
    /// 3. If move is returned, calls [`process_move`](Self::process_move)
    /// 4. If None is returned, player has resigned/cancelled
    ///
    /// # Implementation Notes
    ///
    /// For human players, this is called repeatedly from the event loop.
    /// The player's `get_move` returns `None` until a valid move is made,
    /// then returns `Some(move)` which triggers move processing.
    pub fn request_move(&mut self) {
        if !self.game_active {
            return;
        }

        if let Some((ref mut white, ref mut black)) = self.players {
            let player = match self.current_turn {
                Color::White => white,
                Color::Black => black,
            };

            if let Some(mv) = player.get_move(self.current_turn) {
                self.process_move(mv);
            } else {
                // Player resigned or cancelled
                self.return_to_menu();
            }
        }
    }

    /// Process and execute a move received from a player.
    ///
    /// Validates the move is legal, executes it on the board, checks for game end
    /// conditions, and switches turns if the game continues.
    ///
    /// # Arguments
    ///
    /// * `mv` - The move to process
    ///
    /// # Processing Steps
    ///
    /// 1. Validates move is legal for current position
    /// 2. Executes move on board (updates position)
    /// 3. Notifies opponent of the move (optional Player trait method)
    /// 4. Checks for checkmate, stalemate, or draw
    /// 5. If game continues, switches turn and requests next move
    /// 6. If game ended, displays result and returns to menu
    ///
    /// # Panics
    ///
    /// Panics if move is illegal (indicates bug in player implementation).
    pub fn process_move(&mut self, mv: Move) {
        {
            let mut board = self.board.borrow_mut();

            // Validate move is legal
            assert!(board.is_legal_move(mv), "Player provided illegal move");

            // Execute move
            board.execute_move(mv);
        } // Release borrow before calling other methods

        // Notify opponent
        if let Some((ref mut white, ref mut black)) = self.players {
            let opponent = match self.current_turn {
                Color::White => black,
                Color::Black => white,
            };
            opponent.opponent_moved(mv);
        }

        // Check for game end
        self.check_game_end();

        if self.game_active {
            // Switch turns
            self.current_turn = self.current_turn.opposite();

            // Update POV to new player's perspective
            self.board.borrow_mut().set_pov(self.current_turn);

            // Request redraw to show new position
            self.window.request_redraw();

            // Note: We don't call request_move() anymore - the polling model
            // will automatically detect when the next move is ready
        }
    }

    /// Check if the game has ended and handle the result.
    ///
    /// Detects checkmate, stalemate, and draw conditions. If the game has ended:
    /// - Displays the result to the user
    /// - Sets `game_active` to false
    /// - Optionally returns to menu or waits for user input
    ///
    /// # Game End Conditions
    ///
    /// - **Checkmate**: Current player is in check with no legal moves
    /// - **Stalemate**: Current player is not in check but has no legal moves
    /// - **Draw**: Insufficient material, threefold repetition, fifty-move rule (future)
    ///
    /// # State Changes
    ///
    /// If game has ended:
    /// - Sets `game_active` to false
    /// - Notifies both players of the result
    /// - Displays result UI (future implementation)
    pub fn check_game_end(&mut self) {
        let board = self.board.borrow();
        let opponent_color = self.current_turn.opposite();

        if board.is_checkmate(opponent_color) {
            // TODO: Add logging once log crate is added to dependencies
            // log::info!("Checkmate! {:?} wins", self.current_turn);
            drop(board); // Release borrow
            self.handle_game_end(GameResult::from_winner(self.current_turn));
        } else if board.is_stalemate(opponent_color) {
            // TODO: Add logging once log crate is added to dependencies
            // log::info!("Stalemate! Game is a draw");
            drop(board);
            self.handle_game_end(GameResult::Stalemate);
        }
        // Other draw conditions (insufficient material, etc.) can be added here
    }

    /// Return to the mode selection menu.
    ///
    /// Ends any active game and transitions back to Menu mode. This can be called:
    /// - When a game ends naturally (checkmate, stalemate)
    /// - When a player resigns or cancels
    /// - When user presses escape or clicks "Return to Menu" button
    ///
    /// # State Changes
    ///
    /// - Sets `game_mode` to `Menu`
    /// - Sets `game_active` to false
    /// - Clears `players` (dropping player instances)
    /// - Clears `game_result`
    /// - Requests window redraw to show menu UI
    pub fn return_to_menu(&mut self) {
        self.game_mode = GameMode::Menu;
        self.game_active = false;
        self.players = None;
        self.game_result = None;

        self.board.borrow_mut().set_selected_tile(None);

        self.window.request_redraw();
    }

    /// Handle game end by notifying players and updating state.
    ///
    /// # Arguments
    ///
    /// * `result` - The final result of the game
    fn handle_game_end(&mut self, result: GameResult) {
        self.game_active = false;
        self.game_result = Some(result);

        // Notify both players of the result
        if let Some((ref mut white, ref mut black)) = self.players {
            white.game_ended(result);
            black.game_ended(result);
        }

        // Trigger redraw to show game end overlay
        self.window.request_redraw();
    }
}
