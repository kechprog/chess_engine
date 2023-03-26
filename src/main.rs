#![allow(unused)]
/*
 * TODO:
 * add game_state
 * 
 */


use std::{rc::Rc, env::args};
mod game;
use game::{helpers::game_state::{GameState, Pov}, board_drawer::board::BoardDrawer};
use glium::glutin::{
    event::{Event, WindowEvent, MouseButton, ElementState}, 
    self, event_loop::ControlFlow, dpi::{Position, PhysicalPosition},
};


fn main() {
    let ev = glutin::event_loop::EventLoop::new();
    let wb = glutin::window::WindowBuilder::new().with_title("chess");
    let cb = glutin::ContextBuilder::new().with_vsync(true);
    let display = glium::Display::new(wb, cb, &ev).unwrap();


    // init my things
    let fen = args().nth(1).unwrap_or("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR".to_string());
    let mut game_state = GameState::from_fen(fen.as_str(), Pov::White);
    let mut board_drawer = BoardDrawer::new(display);
    let mut current_pos = PhysicalPosition::new(0.0, 0.0);
    
    ev.run(move |event, _, control_flow| {
        *control_flow = match event {
            glutin::event::Event::WindowEvent { event, .. } => match event {
                glutin::event::WindowEvent::CloseRequested => glutin::event_loop::ControlFlow::Exit,
                glutin::event::WindowEvent::CursorMoved{position, .. } => {
                    current_pos = position;
                    ControlFlow::Poll
                },
                glutin::event::WindowEvent::MouseInput {button, state, .. } => {
                    if button == MouseButton::Left && state == ElementState::Released {
                        match board_drawer.coord_to_tile(current_pos) {
                            Some(tile) => {
                                game_state.selected_tile = Some(tile);
                                board_drawer.draw_position(&game_state);
                                println!("tile: {:?}", tile)
                            },
                            None => {
                                println!("outside board");
                                game_state.selected_tile = None;
                                board_drawer.draw_position(&game_state);
                            },
                        }
                    }
                    ControlFlow::Poll
                },
                glutin::event::WindowEvent::Resized(_) => {
                    board_drawer.draw_position(&game_state);
                    ControlFlow::Poll
                }
                _ => ControlFlow::Poll,
            },
            glutin::event::Event::RedrawRequested(_) => {
                board_drawer.draw_position(&game_state);
                ControlFlow::Poll
            }
            _ => ControlFlow::Poll,
        }
    });
}
