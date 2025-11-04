pub mod agent;
pub mod assets;
pub mod board;
pub mod game_repr;
pub mod orchestrator;
pub mod renderer;

#[cfg(target_arch = "wasm32")]
mod wasm {
    use crate::orchestrator::{GameMode, Orchestrator};
    use crate::renderer::wgpu_renderer::WgpuRenderer;
    use std::sync::Arc;
    use wasm_bindgen::prelude::*;
    use winit::{
        application::ApplicationHandler,
        event::WindowEvent,
        event_loop::{ControlFlow, EventLoop, EventLoopProxy},
        platform::web::{EventLoopExtWebSys, WindowAttributesExtWebSys},
        window::{Window, WindowAttributes},
    };

    /// User event type for async renderer initialization on WASM
    enum UserEvent {
        RendererReady(Box<WgpuRenderer>),
    }

    /// Application handler for WASM event loop with Orchestrator architecture
    struct App {
        orchestrator: Option<Orchestrator>,
        window: Option<Arc<Window>>,
        initializing: bool,
        event_loop_proxy: EventLoopProxy<UserEvent>,
    }

    impl App {
        fn new(event_loop_proxy: EventLoopProxy<UserEvent>) -> Self {
            Self {
                orchestrator: None,
                window: None,
                initializing: false,
                event_loop_proxy,
            }
        }
    }

    impl ApplicationHandler<UserEvent> for App {
        fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
            // Only initialize once
            if self.window.is_none() && !self.initializing {
                self.initializing = true;

                #[allow(unused_mut)]
                let mut window_attrs = WindowAttributes::default()
                    .with_title("Chess Engine - WASM")
                    .with_inner_size(winit::dpi::LogicalSize::new(800.0, 800.0));

                // Attach to canvas for WASM
                if let Some(canvas) = web_sys::window()
                    .and_then(|win| win.document())
                    .and_then(|doc| doc.get_element_by_id("chess-canvas"))
                    .and_then(|element| element.dyn_into::<web_sys::HtmlCanvasElement>().ok())
                {
                    window_attrs = window_attrs.with_canvas(Some(canvas));
                }

                let window = Arc::new(
                    event_loop
                        .create_window(window_attrs)
                        .expect("Failed to create window"),
                );

                self.window = Some(window.clone());

                // Initialize renderer asynchronously
                let event_loop_proxy = self.event_loop_proxy.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    let renderer = WgpuRenderer::new(window.clone()).await;
                    let _ = event_loop_proxy.send_event(UserEvent::RendererReady(Box::new(renderer)));
                });
            }
        }

        fn user_event(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop, event: UserEvent) {
            match event {
                UserEvent::RendererReady(renderer) => {
                    if let Some(window) = &self.window {
                        // Create orchestrator with the renderer
                        let orchestrator = Orchestrator::new(window.clone(), *renderer);

                        self.orchestrator = Some(orchestrator);
                        self.initializing = false;
                        window.request_redraw();
                    }
                }
            }
        }

        fn window_event(
            &mut self,
            event_loop: &winit::event_loop::ActiveEventLoop,
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

    /// WASM entry point
    #[wasm_bindgen(start)]
    pub fn run() {
        // Initialize panic hook and logging for better debugging
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
        console_log::init_with_level(log::Level::Info).expect("Failed to initialize logger");

        web_sys::console::log_1(&"Initializing Chess Engine WASM...".into());

        let event_loop = EventLoop::<UserEvent>::with_user_event()
            .build()
            .expect("Failed to create event loop");

        event_loop.set_control_flow(ControlFlow::Wait);

        // Create proxy before spawning app
        let event_loop_proxy = event_loop.create_proxy();
        let app = App::new(event_loop_proxy);

        // Use spawn_app for WASM (non-blocking)
        let _ = event_loop.spawn_app(app);

        web_sys::console::log_1(&"Chess Engine WASM initialized successfully!".into());
    }
}
