use crate::{
    game_repr::{Color, Position},
    renderer::Renderer,
};
use std::sync::Arc;
use winit::{
    dpi::PhysicalPosition,
    event::{Event, WindowEvent},
    event_loop::ActiveEventLoop,
    window::Window,
};

use super::Agent;

pub struct TwoPlayerAgent {
    position: Position,
    renderer: Box<dyn Renderer>,
    window: Arc<Window>,

    turn: Color,
    mouse_pos: PhysicalPosition<f64>,
    selected_tile: Option<u8>,
    game_over: bool,
}

impl TwoPlayerAgent {
    pub fn new(renderer: impl Renderer + 'static, window: Arc<Window>) -> Self {
        dbg!(Position::default().position);
        TwoPlayerAgent {
            position: Position::default(),
            renderer: Box::new(renderer),
            window,
            mouse_pos: PhysicalPosition::new(0.0, 0.0),
            turn: Color::White,
            selected_tile: None,
            game_over: false,
        }
    }

    pub fn request_redraw(&self) {
        self.window.request_redraw();
    }

    // TODO: refactor!!!
    // TODO: FIXME
    fn mouse_click(&mut self, pos: PhysicalPosition<f64>) {
        #[cfg(debug_assertions)]
        {
            println!("\n=== MOUSE CLICK ===");
            println!("Current turn: {:?}", self.turn);
        }

        // Prevent moves when game is over
        if self.game_over {
            #[cfg(debug_assertions)]
            println!("Game is over - ignoring click");
            return;
        }

        let clicked_tile = if self.renderer.coord_to_tile(pos, self.turn).is_none() {
            #[cfg(debug_assertions)]
            println!("Clicked outside board - deselecting");
            self.selected_tile = None;
            self.window.request_redraw();
            return;
        } else if self.selected_tile.is_none() {
            let tile = self.renderer.coord_to_tile(pos, self.turn);
            // Only select pieces that belong to the current player
            if let Some(tile_idx) = tile {
                let piece = self.position.position[tile_idx as usize];

                #[cfg(debug_assertions)]
                {
                    let square_name = format!("{}{}", (b'a' + (tile_idx % 8)) as char, (tile_idx / 8) + 1);
                    println!("Clicked tile {} (index {})", square_name, tile_idx);
                    println!("Piece at square: {:?} {:?}", piece.color, piece.piece_type);
                }

                if piece.color == self.turn {
                    #[cfg(debug_assertions)]
                    {
                        let legal_moves = self.position.legal_moves(tile_idx as usize);
                        let square_name = format!("{}{}", (b'a' + (tile_idx % 8)) as char, (tile_idx / 8) + 1);
                        println!("Selected {:?} {:?} at {} - {} legal moves available",
                                 piece.color, piece.piece_type, square_name, legal_moves.len());

                        // Print all legal moves
                        if !legal_moves.is_empty() {
                            print!("Legal moves: ");
                            for mv in &legal_moves {
                                let to_square = format!("{}{}",
                                    (b'a' + (mv._to() % 8) as u8) as char,
                                    (mv._to() / 8) + 1);
                                print!("{} ", to_square);
                            }
                            println!();
                        }
                    }

                    self.selected_tile = tile;
                } else {
                    #[cfg(debug_assertions)]
                    println!("Cannot select - piece belongs to {:?}, current turn is {:?}",
                             piece.color, self.turn);
                }
            }
            self.window.request_redraw();
            return;
        } else {
            self.renderer.coord_to_tile(pos, self.turn).unwrap()
        };
        let selected_tile = self.selected_tile.unwrap();

        #[cfg(debug_assertions)]
        let from_square = format!("{}{}",
            (b'a' + (selected_tile % 8)) as char,
            (selected_tile / 8) + 1);
        #[cfg(debug_assertions)]
        let to_square = format!("{}{}",
            (b'a' + (clicked_tile % 8)) as char,
            (clicked_tile / 8) + 1);

        let legal_moves = self.position.legal_moves(selected_tile as usize);

        #[cfg(debug_assertions)]
        println!("Attempting move: {} -> {}", from_square, to_square);

        match legal_moves
            .iter()
            .position(|m| m._to() == clicked_tile as usize && m._from() == selected_tile as usize)
        {
            Some(i) => {
                #[cfg(debug_assertions)]
                {
                    let moving_piece = self.position.position[selected_tile as usize];
                    let captured_piece = self.position.position[clicked_tile as usize];

                    println!("Move is LEGAL!");
                    println!("Moving: {:?} {:?} from {} to {}",
                             moving_piece.color, moving_piece.piece_type, from_square, to_square);
                    println!("Move type: {:?}", legal_moves[i].move_type());

                    if captured_piece.piece_type != crate::game_repr::Type::None {
                        println!("Capturing: {:?} {:?}", captured_piece.color, captured_piece.piece_type);
                    }
                }

                self.position.mk_move(legal_moves[i]);
                self.selected_tile = None;

                self.turn = self.turn.opposite();

                #[cfg(debug_assertions)]
                println!("Turn switched to: {:?}", self.turn);

                // Check for game ending conditions after switching turns
                if self.position.is_checkmate(self.turn) {
                    self.game_over = true;
                    println!("╔════════════════════════════════╗");
                    println!("║   CHECKMATE! {:?} WINS!   ║", self.turn.opposite());
                    println!("╚════════════════════════════════╝");
                } else if self.position.is_stalemate(self.turn) {
                    self.game_over = true;
                    println!("╔════════════════════════════════╗");
                    println!("║      STALEMATE! DRAW.          ║");
                    println!("╚════════════════════════════════╝");
                } else if self.position.is_in_check(self.turn) {
                    println!("⚠️  CHECK! {:?} king is in check!", self.turn);
                }
            }
            None => {
                // Move is illegal - check if clicked tile has current player's piece
                let clicked_piece = self.position.position[clicked_tile as usize];

                #[cfg(debug_assertions)]
                {
                    println!("Move is ILLEGAL - move not in legal moves list");
                    if clicked_piece.color == self.turn && clicked_piece.piece_type != crate::game_repr::Type::None {
                        println!("Switching selection to {} (same color piece)", to_square);
                    } else {
                        println!("Deselecting (clicked empty or opponent piece)");
                    }
                }

                if clicked_piece.color == self.turn && clicked_piece.piece_type != crate::game_repr::Type::None {
                    // Switching selection to another piece of the same color
                    self.selected_tile = Some(clicked_tile);
                } else {
                    // Clicked on empty square or opponent's piece - deselect
                    self.selected_tile = None;
                }
            }
        }

        // Request redraw to update the display
        self.window.request_redraw();
    }

    fn mouse_moved(&mut self, pos: PhysicalPosition<f64>) {
        self.mouse_pos = pos;
    }
}

impl Agent<()> for TwoPlayerAgent {
    fn handle_input(&mut self, ev: Event<()>, window_target: &ActiveEventLoop) {
        match ev {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => window_target.exit(),

                WindowEvent::MouseInput { state, .. } => {
                    if state.is_pressed() {
                        self.mouse_click(self.mouse_pos);
                    }
                }
                WindowEvent::CursorMoved { position, .. } => {
                    self.mouse_moved(position);
                }

                WindowEvent::Resized(new_size) => {
                    self.renderer.resize((new_size.width, new_size.height));
                    self.window.request_redraw();
                }
                WindowEvent::RedrawRequested => {
                    self.renderer
                        .draw_position(&self.position, self.selected_tile, self.turn);
                }
                _ => (),
            },
            _ => (),
        }
    }
}
