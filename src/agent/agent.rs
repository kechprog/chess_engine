use std::sync::{Arc, Mutex};

use glium::{
    glutin::{event::Event, event_loop::ControlFlow},
    Display,
};


/*
 *
 *  - THIS MODULE IS RESPONSIBLE FOR HANDALING AN EVENTS DRAWING THE BOARD
 *  - AND IN GENERAL HANDLE THE LOGIC OF THE GAME
 *
 */


pub trait Agent {
    fn new(display: Display) -> Self;
    fn handle_input(&mut self, ev: Event<()>) -> ControlFlow;
}
