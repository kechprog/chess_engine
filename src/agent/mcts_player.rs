//! MCTS (Monte Carlo Tree Search) AI player implementation.
//!
//! This module provides an AI player that uses Monte Carlo Tree Search with limited depth
//! and rule-based position evaluation. When the search reaches the maximum depth,
//! it evaluates the position using a rule-based heuristic rather than continuing simulation.
//!
//! # Algorithm
//!
//! The MCTS algorithm consists of four phases:
//! 1. **Selection**: Traverse the tree using UCT (Upper Confidence Bound for Trees)
//! 2. **Expansion**: Add a new node to the tree
//! 3. **Simulation**: Play out moves until depth limit, then evaluate
//! 4. **Backpropagation**: Update node statistics back up the tree
//!
//! # Evaluation Function
//!
//! The rule-based evaluation considers:
//! - Material balance (piece values)
//! - Piece-square tables (positional bonuses)
//! - King safety
//! - Mobility (number of legal moves)
//! - Checkmate and stalemate detection

use crate::board::Board;
use crate::game_repr::{Color, Move, Position, Type, Piece};
use crate::agent::player::Player;
use std::cell::RefCell;
use std::sync::Arc;
use std::f32;
use rand::Rng;
use rayon::prelude::*;
use parking_lot::Mutex;
use std::collections::HashMap;

/// Configuration for the MCTS algorithm
#[derive(Clone, Copy)]
pub struct MCTSConfig {
    /// Maximum depth for simulation (after which evaluation is used)
    pub max_depth: u32,
    /// Number of MCTS iterations to run
    pub iterations: u32,
    /// Exploration constant for UCT formula (typically sqrt(2) ≈ 1.414)
    pub exploration_constant: f32,
}

impl Default for MCTSConfig {
    fn default() -> Self {
        Self {
            max_depth: 10,
            iterations: 1000,
            exploration_constant: 1.414,
        }
    }
}

/// MCTS AI player
pub struct MCTSPlayer {
    /// Shared reference to the board for querying state
    board: Arc<RefCell<Board>>,
    /// Configuration for the MCTS algorithm
    config: MCTSConfig,
    /// Display name for this player
    name: String,
}

impl MCTSPlayer {
    /// Create a new MCTS AI player
    ///
    /// # Arguments
    ///
    /// * `board` - Shared reference to the board for querying game state
    /// * `config` - MCTS configuration (depth, iterations, exploration constant)
    /// * `name` - Display name for this player
    ///
    /// # Returns
    ///
    /// A new `MCTSPlayer` instance.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let board = Arc::new(RefCell::new(Board::new(renderer)));
    /// let config = MCTSConfig::default();
    /// let player = MCTSPlayer::new(board.clone(), config, "AI".to_string());
    /// ```
    pub fn new(board: Arc<RefCell<Board>>, config: MCTSConfig, name: String) -> Self {
        Self { board, config, name }
    }

    /// Run MCTS to find the best move (parallel version)
    fn search(&self, position: &Position, color: Color) -> Option<Move> {
        let legal_moves = position.all_legal_moves();

        if legal_moves.is_empty() {
            return None;
        }

        if legal_moves.len() == 1 {
            return Some(legal_moves[0]);
        }

        // Use root parallelization: run multiple independent MCTS searches
        // and combine their results
        let num_threads = rayon::current_num_threads();
        let iterations_per_thread = self.config.iterations / num_threads as u32;
        let extra_iterations = self.config.iterations % num_threads as u32;

        // Clone config and position for thread-safe access
        let config = self.config.clone();
        let position = position.clone();

        // Shared storage for aggregating results from all threads
        let move_stats: Arc<Mutex<HashMap<Move, (u32, f32)>>> = Arc::new(Mutex::new(HashMap::new()));

        // Run parallel MCTS searches
        (0..num_threads).into_par_iter().for_each(|thread_id| {
            // Calculate iterations for this thread
            let iterations = if thread_id == 0 {
                iterations_per_thread + extra_iterations
            } else {
                iterations_per_thread
            };

            // Create a root node for this thread
            let mut root = MCTSNode::new(None, color);

            // Run MCTS iterations for this thread
            for _ in 0..iterations {
                let mut pos = position.clone();
                root.iterate(&mut pos, &config, 0);
            }

            // Collect statistics from this thread's root node
            let mut stats = move_stats.lock();
            for child in &root.children {
                if let Some(mv) = child.mv {
                    let entry = stats.entry(mv).or_insert((0, 0.0));
                    entry.0 += child.visits;
                    entry.1 += child.score;
                }
            }
        });

        // Select the move with the highest total visit count
        let stats = move_stats.lock();
        stats.iter()
            .max_by_key(|(_, (visits, _))| visits)
            .map(|(mv, _)| *mv)
    }
}

impl Player for MCTSPlayer {
    fn get_move(&mut self, color: Color) -> Option<Move> {
        // Get the current position from the board
        let position = {
            let board = self.board.borrow();
            board.position().clone()
        };

        // Run MCTS search to find the best move
        self.search(&position, color)
    }

    fn name(&self) -> &str {
        &self.name
    }
}

/// Node in the MCTS tree
struct MCTSNode {
    /// The move that led to this node (None for root)
    mv: Option<Move>,
    /// Number of times this node has been visited
    visits: u32,
    /// Sum of evaluation scores from simulations
    score: f32,
    /// Children nodes (expanded moves)
    children: Vec<MCTSNode>,
    /// Untried moves (not yet expanded)
    untried_moves: Vec<Move>,
    /// Color of the player who made the move to reach this node
    player_color: Color,
}

impl MCTSNode {
    /// Create a new MCTS node
    fn new(mv: Option<Move>, player_color: Color) -> Self {
        Self {
            mv,
            visits: 0,
            score: 0.0,
            children: Vec::new(),
            untried_moves: Vec::new(),
            player_color,
        }
    }

    /// Run one MCTS iteration from this node
    fn iterate(&mut self, position: &mut Position, config: &MCTSConfig, depth: u32) -> f32 {
        // Initialize untried moves on first visit
        if self.visits == 0 {
            self.untried_moves = position.all_legal_moves();
        }

        let result = if depth >= config.max_depth {
            // Reached depth limit - evaluate position
            evaluate_position(position, self.player_color)
        } else if !self.untried_moves.is_empty() {
            // Expansion phase - try a new move
            self.expand(position, config, depth)
        } else if !self.children.is_empty() {
            // Selection phase - choose best child
            self.select_child(position, config, depth)
        } else {
            // Terminal node (no legal moves) - evaluate
            evaluate_position(position, self.player_color)
        };

        // Backpropagation
        self.visits += 1;
        self.score += result;
        result
    }

    /// Expand a new child node
    fn expand(&mut self, position: &mut Position, config: &MCTSConfig, depth: u32) -> f32 {
        // Pick a random untried move
        let mut rng = rand::thread_rng();
        let move_idx = rng.gen_range(0..self.untried_moves.len());
        let mv = self.untried_moves.swap_remove(move_idx);

        // Make the move
        let undo = position.make_move_undoable(mv);

        // Create child node
        let mut child = MCTSNode::new(Some(mv), self.player_color.opposite());

        // Simulate from this child
        let result = child.iterate(position, config, depth + 1);

        // Unmake the move
        position.unmake_move(mv, undo);

        // Add child to tree
        self.children.push(child);

        result
    }

    /// Select the best child using UCT
    fn select_child(&mut self, position: &mut Position, config: &MCTSConfig, depth: u32) -> f32 {
        // Calculate UCT value for each child
        let total_visits = self.visits as f32;
        let mut best_value = f32::NEG_INFINITY;
        let mut best_idx = 0;

        for (i, child) in self.children.iter().enumerate() {
            let uct = child.uct_value(total_visits, config.exploration_constant);
            if uct > best_value {
                best_value = uct;
                best_idx = i;
            }
        }

        // Get the best child
        let child = &mut self.children[best_idx];
        let mv = child.mv.unwrap();

        // Make the move
        let undo = position.make_move_undoable(mv);

        // Recurse
        let result = child.iterate(position, config, depth + 1);

        // Unmake the move
        position.unmake_move(mv, undo);

        result
    }

    /// Calculate UCT (Upper Confidence Bound for Trees) value
    fn uct_value(&self, parent_visits: f32, exploration_constant: f32) -> f32 {
        if self.visits == 0 {
            return f32::INFINITY;
        }

        let exploitation = self.score / self.visits as f32;
        let exploration = exploration_constant * (parent_visits.ln() / self.visits as f32).sqrt();

        exploitation + exploration
    }

    /// Select the best move based on visit counts
    fn best_move(&self) -> Option<Move> {
        self.children
            .iter()
            .max_by_key(|child| child.visits)
            .and_then(|child| child.mv)
    }
}

/// Evaluate a chess position from the perspective of the given color
/// Returns a score where positive is good for the color, negative is bad
fn evaluate_position(position: &Position, color: Color) -> f32 {
    // Check for terminal states first
    if position.is_checkmate(color) {
        return -10000.0; // Loss
    }
    if position.is_checkmate(color.opposite()) {
        return 10000.0; // Win
    }
    if position.is_stalemate(color) || position.is_stalemate(color.opposite()) {
        return 0.0; // Draw
    }

    let mut score = 0.0;

    // Material evaluation
    score += material_score(position, color);

    // Positional evaluation (piece-square tables)
    score += positional_score(position, color);

    // Mobility (number of legal moves)
    score += mobility_score(position, color);

    score
}

/// Calculate material balance
fn material_score(position: &Position, color: Color) -> f32 {
    let mut score = 0.0;

    for square in 0..64 {
        let piece = position.position[square];
        if piece.piece_type == Type::None {
            continue;
        }

        let value = piece_value(piece.piece_type);
        if piece.color == color {
            score += value;
        } else {
            score -= value;
        }
    }

    score
}

/// Get the material value of a piece
fn piece_value(piece_type: Type) -> f32 {
    match piece_type {
        Type::Pawn => 100.0,
        Type::Knight => 320.0,
        Type::Bishop => 330.0,
        Type::Rook => 500.0,
        Type::Queen => 900.0,
        Type::King => 20000.0,
        Type::None => 0.0,
    }
}

/// Calculate positional score using piece-square tables
fn positional_score(position: &Position, color: Color) -> f32 {
    let mut score = 0.0;

    for square in 0..64 {
        let piece = position.position[square];
        if piece.piece_type == Type::None {
            continue;
        }

        let bonus = piece_square_value(piece, square);
        if piece.color == color {
            score += bonus;
        } else {
            score -= bonus;
        }
    }

    score
}

/// Get positional bonus for a piece at a given square
fn piece_square_value(piece: Piece, square: usize) -> f32 {
    // Flip square for black pieces (they see the board from the opposite side)
    let sq = if piece.color == Color::Black {
        63 - square
    } else {
        square
    };

    let rank = sq / 8;
    let file = sq % 8;
    let center_distance = ((3.5 - file as f32).abs() + (3.5 - rank as f32).abs()) / 2.0;

    match piece.piece_type {
        Type::Pawn => {
            // Pawns are better when advanced
            rank as f32 * 10.0
        }
        Type::Knight | Type::Bishop => {
            // Knights and bishops prefer the center
            (3.5 - center_distance) * 10.0
        }
        Type::Rook => {
            // Rooks prefer the 7th rank
            if rank == 6 {
                20.0
            } else {
                0.0
            }
        }
        Type::Queen => {
            // Queens slightly prefer the center
            (3.5 - center_distance) * 5.0
        }
        Type::King => {
            // Kings prefer corners in the middlegame (simplified)
            -center_distance * 10.0
        }
        Type::None => 0.0,
    }
}

/// Calculate mobility score
fn mobility_score(position: &Position, color: Color) -> f32 {
    // Count legal moves for current side
    let current_side = if position.prev_moves.len() % 2 == 0 {
        Color::White
    } else {
        Color::Black
    };

    let my_moves = position.all_legal_moves().len() as f32;

    // Simple mobility bonus
    if current_side == color {
        my_moves * 0.1
    } else {
        -my_moves * 0.1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_piece_values() {
        assert_eq!(piece_value(Type::Pawn), 100.0);
        assert_eq!(piece_value(Type::Knight), 320.0);
        assert_eq!(piece_value(Type::Queen), 900.0);
    }

    #[test]
    fn test_evaluation_checkmate() {
        // Create a simple position and test evaluation
        let position = Position::default();
        let score = evaluate_position(&position, Color::White);
        // Starting position should be roughly equal
        assert!(score.abs() < 200.0);
    }

    #[test]
    fn test_mcts_node_creation() {
        let node = MCTSNode::new(None, Color::White);
        assert_eq!(node.visits, 0);
        assert_eq!(node.score, 0.0);
        assert!(node.children.is_empty());
    }

    #[test]
    fn test_mcts_config_default() {
        let config = MCTSConfig::default();
        assert_eq!(config.max_depth, 10);
        assert_eq!(config.iterations, 1000);
        assert_eq!(config.exploration_constant, 1.414);
    }

    #[test]
    fn test_mcts_search_finds_move() {
        use crate::board::Board;
        use crate::renderer::Renderer;
        use winit::dpi::PhysicalPosition;

        // Mock renderer for testing
        struct MockRenderer;
        impl Renderer for MockRenderer {
            fn draw_position(&mut self, _position: &Position, _selected_tile: Option<u8>, _pov: Color) {}
            fn coord_to_tile(&self, _coords: PhysicalPosition<f64>, _pov: Color) -> Option<u8> { None }
            fn resize(&mut self, _new_size: (u32, u32)) {}
            fn draw_menu(&mut self, _show_coming_soon: bool) {}
            fn is_coord_in_button(&self, _coords: PhysicalPosition<f64>, _button_index: usize) -> bool { false }
            fn draw_game_end(&mut self, _position: &Position, _selected_tile: Option<u8>, _pov: Color, _result: crate::agent::player::GameResult) {}
            fn draw_promotion_selection(&mut self, _position: &Position, _selected_tile: Option<u8>, _pov: Color, _promoting_color: Color) {}
            fn get_promotion_piece_at_coords(&self, _coords: PhysicalPosition<f64>) -> Option<Type> { None }
        }

        let board = std::sync::Arc::new(std::cell::RefCell::new(Board::new(Box::new(MockRenderer))));

        // Create MCTS player with reduced iterations for faster test
        let config = MCTSConfig {
            max_depth: 5,
            iterations: 100,
            exploration_constant: 1.414,
        };
        let mut player = MCTSPlayer::new(board.clone(), config, "Test AI".to_string());

        // Get a move from the starting position (White to move)
        let mv = player.get_move(Color::White);

        // Should return a valid move
        assert!(mv.is_some());

        // Verify the move is legal in the starting position
        let position = board.borrow().position().clone();
        let legal_moves = position.all_legal_moves();
        assert!(legal_moves.contains(&mv.unwrap()));
    }

    #[test]
    fn test_material_evaluation() {
        // Test that material evaluation correctly counts pieces
        let position = Position::default();
        let score = material_score(&position, Color::White);

        // Starting position is equal, so score should be 0
        assert_eq!(score, 0.0);
    }

    #[test]
    fn test_checkmate_evaluation() {
        // Test that checkmate is correctly evaluated
        // Create a position where white is in checkmate (fool's mate)
        let position = Position::from_fen("rnb1kbnr/pppp1ppp/8/4p3/6Pq/5P2/PPPPP2P/RNBQKBNR w KQkq - 1 3");

        // White is in checkmate
        assert!(position.is_checkmate(Color::White));

        // Evaluation from White's perspective should be very negative
        let score = evaluate_position(&position, Color::White);
        assert!(score < -9000.0);
    }
}
