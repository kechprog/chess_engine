use cfg_if::cfg_if;

pub mod agent;
pub mod assets;
pub mod game_repr;
pub mod renderer;

cfg_if! {
    if #[cfg(target_arch = "wasm32")] {
        use agent::agent::Agent;
        use agent::TwoPlayerAgent;
        use renderer::wgpu_renderer::WgpuRenderer;
        use std::sync::Arc;
        use wasm_bindgen::prelude::*;
        use winit::{
            application::ApplicationHandler,
            event::WindowEvent,
            event_loop::{ControlFlow, EventLoop, EventLoopProxy},
            window::{Window, WindowAttributes},
            platform::web::{EventLoopExtWebSys, WindowAttributesExtWebSys},
        };

        /// User event type for async initialization on WASM
        enum UserEvent {
            RendererReady(Box<WgpuRenderer>),
        }

        struct App {
            agent: Option<TwoPlayerAgent>,
            window: Option<Arc<Window>>,
            initializing: bool,
            event_loop_proxy: EventLoopProxy<UserEvent>,
        }

        impl App {
            fn new(event_loop_proxy: EventLoopProxy<UserEvent>) -> Self {
                Self {
                    agent: None,
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
                            self.agent = Some(TwoPlayerAgent::new(*renderer, window.clone()));
                            self.initializing = false;
                            window.request_redraw();
                        }
                    }
                }
            }

            fn window_event(
                &mut self,
                event_loop: &winit::event_loop::ActiveEventLoop,
                window_id: winit::window::WindowId,
                event: WindowEvent,
            ) {
                if let Some(agent) = &mut self.agent {
                    // Convert WindowEvent to Event for agent compatibility
                    let event = winit::event::Event::WindowEvent { window_id, event };
                    agent.handle_input(event, event_loop);
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
}
