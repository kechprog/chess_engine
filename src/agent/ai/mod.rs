// AI player module with MCTS and evaluation

pub mod evaluation;
pub mod piece_square_tables;
pub mod move_ordering;
pub mod mcts;
pub mod ai_player;

#[cfg(test)]
mod tests;

pub use ai_player::{AIPlayer, Difficulty};
