#![allow(unused)]

use agent::{Agent, TwoPlayerAgent};
use renderer::wgpu_renderer::WgpuRenderer;
use std::sync::Arc;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowAttributes},
};

mod agent;
mod game_repr;
mod renderer;

use game_repr::{Move, MoveType};

struct App {
    agent: Option<TwoPlayerAgent>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.agent.is_none() {
            let window_attrs = WindowAttributes::default()
                .with_title("chess")
                .with_inner_size(winit::dpi::LogicalSize::new(800.0, 800.0));

            let window = Arc::new(event_loop.create_window(window_attrs).expect("Failed to create window"));

            // Ensure minimum size
            let _ = window.request_inner_size(winit::dpi::PhysicalSize::new(800, 800));

            let renderer = pollster::block_on(WgpuRenderer::new(window.clone()));
            self.agent = Some(TwoPlayerAgent::new(renderer, window.clone()));

            // Request initial redraw to render the board
            window.request_redraw();
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        if let Some(agent) = &mut self.agent {
            // Convert WindowEvent to Event for agent compatibility
            let event = winit::event::Event::WindowEvent {
                window_id,
                event,
            };
            agent.handle_input(event, event_loop);
        }
    }

}

fn main() {
    let event_loop = EventLoop::new().expect("Failed to create event loop");
    let mut app = App { agent: None };
    let _ = event_loop.run_app(&mut app);
}
