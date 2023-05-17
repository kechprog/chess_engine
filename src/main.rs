#![allow(unused)]

use agent::{TwoPlayerAgent, Agent};
use glium::glutin::{
    self,
    dpi::{PhysicalPosition, Position},
    event::{ElementState, Event, MouseButton, WindowEvent},
    event_loop::ControlFlow, window::{self, WindowId},
};

mod game_repr;
mod agent;
mod board_drawer;

use game_repr::{Move, MoveType};
fn main() {

    let ev = glutin::event_loop::EventLoop::new();
    let wb = glutin::window::WindowBuilder::new()
        .with_title("chess");

    let cb = glutin::ContextBuilder::new()
        .with_vsync(true);

    let display = glium::Display::new(wb, cb, &ev)
        .expect("Could not create");

    
    let mut agent = TwoPlayerAgent::new(display);
    ev.run(move |event, _, control_flow| {
        *control_flow = agent.handle_input(event);
    });
}
