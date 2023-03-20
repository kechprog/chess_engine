#![allow(unused)]

/*
 * TODO:
 * MVP of the board
 * Heavy refactoring
 * function -> beautifull object
 * board is a square not a rectangle, fix it
 */

/*===============================================================*/
/*==================== GAME RELATED STAFF =======================*/
/*===============================================================*/
use glium::glutin::{
    event::{Event, WindowEvent}, 
    self, event_loop::ControlFlow,
};

mod game;
use game::board::Board;

fn main() {
    let ev = glutin::event_loop::EventLoop::new();
    let wb = glutin::window::WindowBuilder::new().with_title("textures");
    let cb = glutin::ContextBuilder::new().with_vsync(true);
    let display = glium::Display::new(wb, cb, &ev).unwrap();

    let mut board = Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR", display);


    ev.run(move |event, _, control_flow| {
        *control_flow = match event {
            glutin::event::Event::WindowEvent { event, .. } => match event {
                glutin::event::WindowEvent::CloseRequested => glutin::event_loop::ControlFlow::Exit,
                glutin::event::WindowEvent::Resized(_) => {
                    board.draw_position();
                    ControlFlow::Poll
                }
                _ => ControlFlow::Poll,
            },
            glutin::event::Event::RedrawRequested(_) => {
                board.draw_position();
                ControlFlow::Poll
            }
            _ => ControlFlow::Poll,
        }
    });
}
