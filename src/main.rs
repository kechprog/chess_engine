use agent::{Agent, TwoPlayerAgent};
use cfg_if::cfg_if;
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
mod game_repr;
mod renderer;

/// User event type for async initialization on WASM
enum UserEvent {
    RendererReady(Box<WgpuRenderer>),
}

struct App {
    agent: Option<TwoPlayerAgent>,
    window: Option<Arc<Window>>,
    initializing: bool,
    #[cfg(target_arch = "wasm32")]
    event_loop_proxy: winit::event_loop::EventLoopProxy<UserEvent>,
}

impl App {
    #[cfg(not(target_arch = "wasm32"))]
    fn new() -> Self {
        Self {
            agent: None,
            window: None,
            initializing: false,
        }
    }

    #[cfg(target_arch = "wasm32")]
    fn new(event_loop_proxy: winit::event_loop::EventLoopProxy<UserEvent>) -> Self {
        Self {
            agent: None,
            window: None,
            initializing: false,
            event_loop_proxy,
        }
    }
}

impl ApplicationHandler<UserEvent> for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // Only initialize once
        if self.window.is_none() && !self.initializing {
            self.initializing = true;

            #[allow(unused_mut)]
            let mut window_attrs = WindowAttributes::default()
                .with_title("Chess Engine")
                .with_inner_size(winit::dpi::LogicalSize::new(800.0, 800.0));

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

            // Initialize renderer asynchronously based on platform
            cfg_if! {
                if #[cfg(target_arch = "wasm32")] {
                    // WASM: Spawn async task
                    let event_loop_proxy = self.event_loop_proxy.clone();
                    wasm_bindgen_futures::spawn_local(async move {
                        let renderer = WgpuRenderer::new(window.clone()).await;
                        let _ = event_loop_proxy.send_event(UserEvent::RendererReady(Box::new(renderer)));
                    });
                } else {
                    // Native: Block on async
                    let renderer = pollster::block_on(WgpuRenderer::new(window.clone()));
                    self.agent = Some(TwoPlayerAgent::new(renderer, window.clone()));
                    self.initializing = false;
                    window.request_redraw();
                }
            }
        }
    }

    fn user_event(&mut self, _event_loop: &ActiveEventLoop, event: UserEvent) {
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
        event_loop: &ActiveEventLoop,
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

    let event_loop = EventLoop::<UserEvent>::with_user_event()
        .build()
        .expect("Failed to create event loop");

    event_loop.set_control_flow(ControlFlow::Wait);

    cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            use winit::platform::web::EventLoopExtWebSys;
            let event_loop_proxy = event_loop.create_proxy();
            let app = App::new(event_loop_proxy);
            let _ = event_loop.spawn_app(app);
        } else {
            let mut app = App::new();
            let _ = event_loop.run_app(&mut app);
        }
    }
}
