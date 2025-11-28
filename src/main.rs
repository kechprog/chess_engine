#![allow(unused)]

use cfg_if::cfg_if;
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

use game_repr::{Move, MoveType};

/// User event type for async initialization on WASM
#[cfg(target_arch = "wasm32")]
enum UserEvent {
    RendererReady(Box<WgpuRenderer>),
}

struct App {
    orchestrator: Option<Orchestrator>,
    window: Option<Arc<Window>>,
    initializing: bool,
    #[cfg(target_arch = "wasm32")]
    event_loop_proxy: winit::event_loop::EventLoopProxy<UserEvent>,
}

impl App {
    #[cfg(not(target_arch = "wasm32"))]
    fn new() -> Self {
        Self {
            orchestrator: None,
            window: None,
            initializing: false,
        }
    }

    #[cfg(target_arch = "wasm32")]
    fn new(event_loop_proxy: winit::event_loop::EventLoopProxy<UserEvent>) -> Self {
        Self {
            orchestrator: None,
            window: None,
            initializing: false,
            event_loop_proxy,
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
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

#[cfg(target_arch = "wasm32")]
impl ApplicationHandler<UserEvent> for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // Only initialize once
        if self.window.is_none() && !self.initializing {
            self.initializing = true;

            #[allow(unused_mut)]
            let mut window_attrs = WindowAttributes::default()
                .with_title("Chess Engine")
                .with_inner_size(winit::dpi::LogicalSize::new(800.0, 800.0))
                .with_min_inner_size(winit::dpi::LogicalSize::new(600.0, 600.0));

            // Attach to canvas for WASM
            cfg_if! {
                if #[cfg(target_arch = "wasm32")] {
                    use winit::platform::web::WindowAttributesExtWebSys;
                    use wasm_bindgen::JsCast;

                    // Get the canvas element from the document
                    if let Some(canvas) = web_sys::window()
                        .and_then(|win| win.document())
                        .and_then(|doc| doc.get_element_by_id("chess-canvas"))
                        .and_then(|element| element.dyn_into::<web_sys::HtmlCanvasElement>().ok())
                    {
                        window_attrs = window_attrs.with_canvas(Some(canvas));
                    }
                }
            }

            let window = Arc::new(
                event_loop
                    .create_window(window_attrs)
                    .expect("Failed to create window"),
            );

            self.window = Some(window.clone());

            // WASM: Spawn async task for renderer initialization
            let event_loop_proxy = self.event_loop_proxy.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let renderer = WgpuRenderer::new(window.clone()).await;
                let _ = event_loop_proxy.send_event(UserEvent::RendererReady(Box::new(renderer)));
            });
        }
    }

    fn user_event(&mut self, _event_loop: &ActiveEventLoop, event: UserEvent) {
        match event {
            UserEvent::RendererReady(renderer) => {
                if let Some(window) = &self.window {
                    self.orchestrator = Some(Orchestrator::new(window.clone(), *renderer));

                    self.initializing = false;
                    window.request_redraw();
                }
            }
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
    // Initialize logging
    cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            std::panic::set_hook(Box::new(console_error_panic_hook::hook));
            console_log::init_with_level(log::Level::Info).expect("Failed to initialize logger");
        } else {
            env_logger::init();
        }
    }

    cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            use winit::platform::web::EventLoopExtWebSys;

            let event_loop = EventLoop::<UserEvent>::with_user_event()
                .build()
                .expect("Failed to create event loop");

            event_loop.set_control_flow(ControlFlow::Wait);

            let event_loop_proxy = event_loop.create_proxy();
            let app = App::new(event_loop_proxy);
            let _ = event_loop.spawn_app(app);
        } else {
            let event_loop = EventLoop::new().expect("Failed to create event loop");
            event_loop.set_control_flow(ControlFlow::Wait);
            let mut app = App::new();
            let _ = event_loop.run_app(&mut app);
        }
    }
}
