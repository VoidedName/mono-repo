use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowId};
use crate::logic::StateLogic;
use crate::state::RenderingContext;

pub struct App<T: StateLogic> {
    #[cfg(target_arch = "wasm32")]
    proxy: Option<winit::event_loop::EventLoopProxy<RenderingContext<T>>>,
    // we just need to hold it temporarily as we need to use it in the resumed callback
    logic: Option<T>,
    state: Option<RenderingContext<T>>,
}

impl<T: StateLogic> App<T> {
    pub fn new(
        #[cfg(target_arch = "wasm32")] event_loop: &winit::event_loop::EventLoop<RenderingContext<T>>,
        logic: T,
    ) -> Self {
        #[cfg(target_arch = "wasm32")]
        let proxy = Some(event_loop.create_proxy());

        Self {
            #[cfg(target_arch = "wasm32")]
            proxy,
            state: None,
            logic: Some(logic),
        }
    }
}

impl<T: StateLogic> ApplicationHandler<RenderingContext<T>> for App<T> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.state.is_some() {
            log::info!("Window already exists, skipping creation");
            return;
        }

        let logic = self.logic.take().expect("Logic should be present");
        #[allow(unused_mut)]
        let mut window_attributes = Window::default_attributes();

        #[cfg(target_arch = "wasm32")]
        {
            use wasm_bindgen::JsCast;
            use winit::platform::web::WindowAttributesExtWebSys;

            const CANVAS_ID: &str = "canvas";

            let window = wgpu::web_sys::window().unwrap();
            let document = window.document().unwrap();
            let canvas = document
                .get_element_by_id(CANVAS_ID)
                .expect("Failed to find canvas!");
            let canvas: web_sys::HtmlCanvasElement = canvas.unchecked_into();
            window_attributes = window_attributes.with_canvas(Some(canvas));
        }

        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());

        #[cfg(not(target_arch = "wasm32"))]
        {
            self.state = Some(pollster::block_on(RenderingContext::new(window, logic)).unwrap());
        }

        #[cfg(target_arch = "wasm32")]
        {
            if let Some(proxy) = self.proxy.take() {
                wasm_bindgen_futures::spawn_local(async move {
                    assert!(
                        // send_event sends it to user_event
                        proxy
                            .send_event(
                                RenderingContext::new(window, logic)
                                    .await
                                    .expect("Failed to create canvas!")
                            )
                            .is_ok()
                    )
                });
            }
        }
    }

    #[allow(unused_mut)]
    fn user_event(&mut self, _event_loop: &ActiveEventLoop, mut event: RenderingContext<T>) {
        #[cfg(target_arch = "wasm32")]
        {
            event.context.window.request_redraw();
            event.resize(
                event.context.window.inner_size().width,
                event.context.window.inner_size().height,
            );
        }

        self.state = Some(event);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        let state = match &mut self.state {
            Some(state) => state,
            None => return,
        };

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                let size = size.to_logical(state.context.window.scale_factor());
                state.resize(size.width, size.height)
            }
            WindowEvent::RedrawRequested => {
                state.update();
                match state.render() {
                    Ok(_) => {}
                    Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::OutOfMemory) => {
                        let size = state
                            .context
                            .window
                            .inner_size()
                            .to_logical(state.context.window.scale_factor());
                        state.resize(size.width, size.height);
                    }
                    Err(e) => log::error!("Failed to render: {:?}", e),
                }
            }
            WindowEvent::KeyboardInput { event, .. } => state.handle_key(event_loop, &event),
            _ => {}
        }
    }
}
