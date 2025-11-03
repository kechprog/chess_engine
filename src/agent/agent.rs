use std::sync::Arc;
use winit::{event::Event, event_loop::ActiveEventLoop};

/*
 *
 *  - THIS MODULE IS RESPONSIBLE FOR HANDLING EVENTS DRAWING THE BOARD
 *  - AND IN GENERAL HANDLE THE LOGIC OF THE GAME
 *
 */

pub trait Agent<T> {
    fn handle_input(&mut self, ev: Event<T>, window_target: &ActiveEventLoop);
}
