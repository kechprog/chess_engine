//! Debug test to understand the illegal move issue

use chess_engine::agent::mcts_player::{MCTSConfig, MCTSPlayer};
use chess_engine::agent::player::Player;
use chess_engine::board::Board;
use chess_engine::game_repr::{Color, Move, Position, Type};
use chess_engine::renderer::Renderer;
use std::cell::RefCell;
use std::sync::Arc;
use winit::dpi::PhysicalPosition;

struct MockRenderer;

impl Renderer for MockRenderer {
    fn draw_position(&mut self, _position: &Position, _selected_tile: Option<u8>, _pov: Color) {}
    fn coord_to_tile(&self, _coords: PhysicalPosition<f64>, _pov: Color) -> Option<u8> { None }
    fn resize(&mut self, _new_size: (u32, u32)) {}
    fn draw_menu(&mut self, _show_coming_soon: bool) {}
    fn is_coord_in_button(&self, _coords: PhysicalPosition<f64>, _button_index: usize) -> bool { false }
    fn draw_game_end(&mut self, _position: &Position, _selected_tile: Option<u8>, _pov: Color, _result: chess_engine::agent::player::GameResult) {}
    fn draw_promotion_selection(&mut self, _position: &Position, _selected_tile: Option<u8>, _pov: Color, _promoting_color: Color) {}
    fn get_promotion_piece_at_coords(&self, _coords: PhysicalPosition<f64>) -> Option<Type> { None }
}

#[test]
fn test_debug_move_generation() {
    println!("\n=== Debug: Move Generation After First Move ===");

    // Create board and make one move for white
    let board = Arc::new(RefCell::new(Board::new(Box::new(MockRenderer))));

    // Get initial position
    let pos_before = board.borrow().position().clone();
    println!("Initial position prev_moves.len() = {}", pos_before.prev_moves.len());

    let white_moves = pos_before.all_legal_moves();
    println!("White has {} legal moves initially", white_moves.len());

    // Play a move for White: e2-e4
    let mv = Move::new(12, 28, chess_engine::game_repr::MoveType::Normal);
    board.borrow_mut().execute_move(mv);

    // Check position after White's move
    let pos_after_white = board.borrow().position().clone();
    println!("\nAfter White's move:");
    println!("  prev_moves.len() = {}", pos_after_white.prev_moves.len());

    // Check what color should move
    let current_side = if pos_after_white.prev_moves.len() % 2 == 0 {
        Color::White
    } else {
        Color::Black
    };
    println!("  Current side to move: {:?}", current_side);

    // Get Black's legal moves
    let black_moves = pos_after_white.all_legal_moves();
    println!("  Black has {} legal moves", black_moves.len());

    if black_moves.len() > 0 {
        println!("  First 5 Black moves:");
        for (i, m) in black_moves.iter().take(5).enumerate() {
            println!("    {}. from {} to {} ({:?})",
                i+1, m._from(), m._to(), m.move_type());
        }
    }

    // Now test MCTS player
    println!("\n=== Testing MCTS Player ===");
    let config = MCTSConfig {
        max_depth: 5,
        iterations: 50,
        exploration_constant: 1.414,
    };
    let mut mcts = MCTSPlayer::new(board.clone(), config, "MCTS".to_string());

    // Get move from MCTS for Black
    let mcts_move = mcts.get_move(Color::Black);

    match mcts_move {
        Some(m) => {
            println!("MCTS chose: from {} to {} ({:?})", m._from(), m._to(), m.move_type());

            // Check if this move is in Black's legal moves
            if black_moves.contains(&m) {
                println!("✓ MCTS move IS in legal moves list");
            } else {
                println!("✗ MCTS move is NOT in legal moves list!");
                println!("This is the bug!");
            }
        }
        None => {
            println!("MCTS returned None (no move)");
        }
    }
}
