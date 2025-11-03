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
        }
    }

    pub fn request_redraw(&self) {
        self.window.request_redraw();
    }

    // TODO: refactor!!!
    // TODO: FIXME
    fn mouse_click(&mut self, pos: PhysicalPosition<f64>) {
        let clicked_tile = if self.renderer.coord_to_tile(pos, self.turn).is_none() {
            self.selected_tile = None;
            self.window.request_redraw();
            return;
        } else if self.selected_tile.is_none() {
            self.selected_tile = self.renderer.coord_to_tile(pos, self.turn);
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
                dbg!();
                self.position.mk_move(legal_moves[i]);
                self.selected_tile = None;

                self.turn = self.turn.opposite();
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
