use crate::{
    board_drawer::board::BoardDrawer,
    game_repr::{Color, Position},
};
use glium::{
    glutin::{
        dpi::PhysicalPosition,
        event::{ElementState, Event, WindowEvent},
        event_loop::ControlFlow,
    },
    Display,
};

use super::Agent;

pub struct TwoPlayerAgent {
    position: Position,
    board_drawer: BoardDrawer,

    pov: Color,
    mouse_pos: PhysicalPosition<f64>,
    selected_tile: Option<u8>,
}

impl TwoPlayerAgent {

    // TODO: refactor!!!
    // TODO: FIXME
    fn mouse_click(&mut self, pos: PhysicalPosition<f64>) {

        let clicked_tile = if self.board_drawer.coord_to_tile(pos).is_none() {
            self.selected_tile = None;
            self.board_drawer
                .draw_position(&self.position, self.selected_tile, self.pov);
            return;
        } else if self.selected_tile.is_none() {
            self.selected_tile = self.board_drawer.coord_to_tile(pos);
            self.board_drawer
                .draw_position(&self.position, self.selected_tile, self.pov);
            return;
        } else {
            self.board_drawer.coord_to_tile(pos).unwrap()
        };
        let selected_tile = self.selected_tile.unwrap();


        let legal_moves = self.position.legal_moves(selected_tile as usize);
        match legal_moves
            .iter()
            .position(|m| m._to() == clicked_tile as usize && m._from() == selected_tile as usize) {
            Some(i) => {
                dbg!(legal_moves[i]._from(), legal_moves[i]._to(), legal_moves[i].move_type());
                self.position.mk_move(legal_moves[i]);
                self.selected_tile = None;
                
                self.pov = self.pov.opposite();
                self.board_drawer
                    .draw_position(&self.position, self.selected_tile, self.pov);
            }
            None => {
                self.selected_tile = Some(clicked_tile);
            }
        }


        // redraw the staff
        self.board_drawer
            .draw_position(&self.position, self.selected_tile, self.pov);
    }

    fn mouse_moved(&mut self, pos: PhysicalPosition<f64>) {
        self.mouse_pos = pos;
    }
}

impl Agent for TwoPlayerAgent {
    fn new(display: Display) -> Self {
        TwoPlayerAgent {
            position: Position::default(),
            board_drawer: BoardDrawer::new(display),
            mouse_pos: PhysicalPosition::new(0.0, 0.0),
            pov: Color::White,
            selected_tile: None,
        }
    }

    fn handle_input(&mut self, ev: Event<()>) -> ControlFlow {
        match ev {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => ControlFlow::Exit,

                WindowEvent::MouseInput { state, button, .. } => {
                    if state == ElementState::Pressed {
                        self.mouse_click(self.mouse_pos);
                    }
                    ControlFlow::Poll
                }
                WindowEvent::CursorMoved { position, .. } => {
                    self.mouse_moved(position);
                    ControlFlow::Poll
                }

                WindowEvent::Resized(_) => {
                    self.board_drawer
                        .draw_position(&self.position, self.selected_tile, self.pov);
                    ControlFlow::Poll
                }
                _ => ControlFlow::Poll,
            },
            Event::RedrawRequested(_) => {
                self.board_drawer
                    .draw_position(&self.position, self.selected_tile, self.pov);
                ControlFlow::Poll
            }
            _ => ControlFlow::Poll,
        }
    }
}