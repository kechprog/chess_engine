#![allow(unused)]

use orchestrator::Orchestrator;
use renderer::wgpu_renderer::WgpuRenderer;
use std::sync::Arc;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowAttributes},
};

mod agent;
mod board;
mod game_repr;
mod orchestrator;
mod renderer;

use game_repr::{Move, MoveType};

struct App {
    orchestrator: Option<Orchestrator>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.orchestrator.is_none() {
            let window_attrs = WindowAttributes::default()
                .with_title("chess")
                .with_inner_size(winit::dpi::LogicalSize::new(800.0, 800.0));

            let window = Arc::new(event_loop.create_window(window_attrs).expect("Failed to create window"));

            // Ensure minimum size
            let _ = window.request_inner_size(winit::dpi::PhysicalSize::new(800, 800));

            let renderer = pollster::block_on(WgpuRenderer::new(window.clone()));
            self.orchestrator = Some(Orchestrator::new(window.clone(), renderer));

            // TODO: Remove this auto-start once menu UI is implemented
            // Automatically start PvP game for testing
            if let Some(orch) = &mut self.orchestrator {
                orch.set_game_mode(orchestrator::GameMode::PvP);
                orch.start_game();
            }

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
        // Handle app-level events first
        match &event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
                return;
            }
            _ => {}
        }

        // Delegate to orchestrator
        if let Some(orchestrator) = &mut self.orchestrator {
            orchestrator.handle_event(event);
        }
    }

}

fn main() {
    let event_loop = EventLoop::new().expect("Failed to create event loop");
    let mut app = App { orchestrator: None };
    let _ = event_loop.run_app(&mut app);
}
