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
        // Prevent moves when game is over
        if self.game_over {
            return;
        }

        let clicked_tile = if self.renderer.coord_to_tile(pos, self.turn).is_none() {
            self.selected_tile = None;
            self.window.request_redraw();
            return;
        } else if self.selected_tile.is_none() {
            let tile = self.renderer.coord_to_tile(pos, self.turn);
            // Only select pieces that belong to the current player
            if let Some(tile_idx) = tile {
                let piece = self.position.position[tile_idx as usize];
                if piece.color == self.turn {
                    self.selected_tile = tile;
                }
            }
            self.window.request_redraw();
            return;
        } else {
            self.renderer.coord_to_tile(pos, self.turn).unwrap()
        };
        let selected_tile = self.selected_tile.unwrap();

        let legal_moves = self.position.legal_moves(selected_tile as usize);
        match legal_moves
            .iter()
            .position(|m| m._to() == clicked_tile as usize && m._from() == selected_tile as usize)
        {
            Some(i) => {
                self.position.mk_move(legal_moves[i]);
                self.selected_tile = None;

                self.turn = self.turn.opposite();

                // Check for game ending conditions after switching turns
                if self.position.is_checkmate(self.turn) {
                    self.game_over = true;
                    println!("Checkmate! {:?} wins!", self.turn.opposite());
                } else if self.position.is_stalemate(self.turn) {
                    self.game_over = true;
                    println!("Stalemate! Draw.");
                } else if self.position.is_in_check(self.turn) {
                    println!("Check!");
                }
            }
            None => {
                self.selected_tile = Some(clicked_tile);
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
