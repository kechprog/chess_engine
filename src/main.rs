//! Chess Engine - Native Binary Entry Point
//!
//! This module provides the native (non-WASM) entry point for the chess engine.
//! For WASM builds, see `lib.rs` which provides the `#[wasm_bindgen(start)]` entry point.

#![allow(unused)]

use orchestrator::Orchestrator;
use renderer::wgpu_renderer::WgpuRenderer;
use std::sync::Arc;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowAttributes},
};

mod agent;
mod assets;
mod board;
mod game_repr;
mod menu;
mod orchestrator;
mod renderer;

/// Native application handler for the chess engine.
///
/// Manages the window lifecycle, renderer initialization, and event delegation
/// to the Orchestrator which handles game logic and rendering.
struct App {
    orchestrator: Option<Orchestrator>,
    window: Option<Arc<Window>>,
    initializing: bool,
}

impl App {
    fn new() -> Self {
        Self {
            orchestrator: None,
            window: None,
            initializing: false,
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // Only initialize once
        if self.window.is_none() && !self.initializing {
            self.initializing = true;

            let window_attrs = WindowAttributes::default()
                .with_title("Chess Engine")
                .with_inner_size(winit::dpi::LogicalSize::new(800.0, 800.0))
                .with_min_inner_size(winit::dpi::LogicalSize::new(600.0, 600.0));

            let window = Arc::new(
                event_loop
                    .create_window(window_attrs)
                    .expect("Failed to create window"),
            );

            self.window = Some(window.clone());

            // Native: Block on async renderer initialization
            let renderer = pollster::block_on(WgpuRenderer::new(window.clone()));
            self.orchestrator = Some(Orchestrator::new(window.clone(), renderer));

            self.initializing = false;
            window.request_redraw();
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: winit::window::WindowId,
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
    // Initialize logging for native builds
    env_logger::init();

    // Create and run the native event loop
    let event_loop = EventLoop::new().expect("Failed to create event loop");
    event_loop.set_control_flow(ControlFlow::Wait);
    let mut app = App::new();
    let _ = event_loop.run_app(&mut app);
}
