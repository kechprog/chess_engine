#![allow(unused)]

use agent::{TwoPlayerAgent, Agent};
use glium::glutin::{
    self,
    dpi::{PhysicalPosition, Position},
    event::{ElementState, Event, MouseButton, WindowEvent},
    event_loop::ControlFlow,
};

mod game_repr;
mod agent;
mod board_drawer;

use game_repr::{Move, MoveType};
fn main() {

    let ev = glutin::event_loop::EventLoop::new();
    let wb = glutin::window::WindowBuilder::new().with_title("chess");
    let cb = glutin::ContextBuilder::new().with_vsync(true);
    let display = glium::Display::new(wb, cb, &ev).unwrap();
    
    let mut agent = TwoPlayerAgent::new(display);
    
    ev.run(move |event, _, control_flow| {
        *control_flow = agent.handle_input(event);
    });

    // // init my things
    // let fen = args()
    //     .nth(1)
    //     .unwrap_or("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR".to_string());
    // let mut game_state = GameState::from_fen(fen.as_str(), Pov::White);
    // let mut board_drawer = BoardDrawer::new(display);
    // let mut current_pos = PhysicalPosition::new(0.0, 0.0);

    // ev.run(move |event, _, control_flow| {
    //     *control_flow = match event {
    //         glutin::event::Event::WindowEvent { event, .. } => match event {


    //             glutin::event::WindowEvent::CloseRequested => glutin::event_loop::ControlFlow::Exit,
    //             glutin::event::WindowEvent::CursorMoved { position, .. } => {
    //                 current_pos = position;
    //                 ControlFlow::Poll
    //             }

    //             // ---------------------- Mouse interaction ----------------------
    //             glutin::event::WindowEvent::MouseInput { button, state, .. } => {

    //                 if button == MouseButton::Left && state == ElementState::Released {
    //                     match board_drawer.coord_to_tile(current_pos) {
    //                         Some(clicked_tile) => {
    //                             match game_state.selected_tile {
    //                                 Some(selected_piece) => {

    //                                     // check if clicked on legal move
    //                                     let legal_move = game_state
    //                                         .position
    //                                         .legal_moves(selected_piece)
    //                                         .contains(&(clicked_tile as u8));

    //                                     if legal_move {
    //                                         game_state.position.board[clicked_tile as usize] =
    //                                             game_state.position.board[selected_piece as usize];
    //                                         game_state.position.board[selected_piece as usize] = Piece::none();
    //                                         game_state.selected_tile = None;
    //                                         board_drawer.draw_position(&game_state);
    //                                     } else {
    //                                         game_state.selected_tile = Some(clicked_tile);
    //                                         board_drawer.draw_position(&game_state);
    //                                     }
    //                                 },
    //                                 None => {
    //                                     game_state.selected_tile = Some(clicked_tile);
    //                                     board_drawer.draw_position(&game_state);
    //                                 }
    //                             }
    //                         }
    //                         None => {
    //                             println!("outside board");
    //                             game_state.selected_tile = None;
    //                             board_drawer.draw_position(&game_state);
    //                         }
    //                     }
    //                 }

    //                 ControlFlow::Poll
    //             }



    //             glutin::event::WindowEvent::Resized(_) => {
    //                 board_drawer.draw_position(&game_state);
    //                 ControlFlow::Poll
    //             }
    //             _ => ControlFlow::Poll,
    //         },
    //         glutin::event::Event::RedrawRequested(_) => {
    //             board_drawer.draw_position(&game_state);
    //             ControlFlow::Poll
    //         }
    //         _ => ControlFlow::Poll,
    //     }
    // });
}
