use super::{piece::{Piece, self}, game_state::GameState};


// gets legal move for a selected piece
// returns None if the piece is not selected or if there is no legal moves
// pub fn get_legal_moves(state: &GameState) -> Option<Vec<u8>> {
//     let sel_piece = if let Some(idx) = state.selected_tile {
//         state.position[idx]
//     } else {
//         return None;
//     };



//     Some(vec![])
// }
