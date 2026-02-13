use crate::logic::StateLogic;
use crate::rendering_context::{EventDispatcher, RenderingContext};
use crate::resource_manager::ResourceManager;
use crate::{GraphicsContext, Renderer, UiEvent};
use std::rc::Rc;
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::{MouseScrollDelta, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowId};

pub struct App<FNew, FRet, R: Renderer + 'static, T: StateLogic<R>>
where
    FNew: Fn(EventDispatcher<T, R>, Rc<GraphicsContext>, Rc<ResourceManager>) -> FRet + 'static,
    FRet: Future<Output = anyhow::Result<T>>,
{
    proxy: winit::event_loop::EventLoopProxy<UiEvent<RenderingContext<T, R>, T::Event>>,
    state: Option<RenderingContext<T, R>>,
    new_fn: Rc<FNew>,
    title: String,
    init_size: (f32, f32),
}

impl<FNew, FRet, R: Renderer + 'static, T: StateLogic<R>> App<FNew, FRet, R, T>
where
    FNew: Fn(EventDispatcher<T, R>, Rc<GraphicsContext>, Rc<ResourceManager>) -> FRet + 'static,
    FRet: Future<Output = anyhow::Result<T>>,
{
    pub fn new(
        event_loop: &winit::event_loop::EventLoop<UiEvent<RenderingContext<T, R>, T::Event>>,
        title: String,
        size: (f32, f32),
        new_fn: FNew,
    ) -> Self
    where
        FRet: Future<Output = anyhow::Result<T>>,
    {
        let proxy = event_loop.create_proxy();

        Self {
            proxy,
            state: None,
            new_fn: Rc::new(new_fn),
            init_size: size,
            title,
        }
    }
}

impl<FNew, FRet, R: Renderer + 'static, T: StateLogic<R>>
    ApplicationHandler<UiEvent<RenderingContext<T, R>, T::Event>> for App<FNew, FRet, R, T>
where
    FNew: Fn(EventDispatcher<T, R>, Rc<GraphicsContext>, Rc<ResourceManager>) -> FRet + 'static,
    FRet: Future<Output = anyhow::Result<T>>,
{
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.state.is_some() {
            log::info!("Window already exists, skipping creation");
            return;
        }

        #[allow(unused_mut)]
        let mut window_attributes = Window::default_attributes()
            .with_inner_size(PhysicalSize::new(self.init_size.0, self.init_size.1))
            .with_title(&self.title);

        #[cfg(target_arch = "wasm32")]
        {
            use wasm_bindgen::JsCast;
            use winit::platform::web::WindowAttributesExtWebSys;

            const CANVAS_ID: &str = "canvas";

            let canvas = web_sys::window()
                .unwrap()
                .document()
                .unwrap()
                .get_element_by_id(CANVAS_ID)
                .unwrap()
                .dyn_into::<web_sys::HtmlCanvasElement>()
                .unwrap();

            log::info!("Canvas element: {:?}", canvas);

            window_attributes = window_attributes.with_canvas(Some(canvas));
        }

        let window = event_loop.create_window(window_attributes).unwrap();

        #[cfg(not(target_arch = "wasm32"))]
        {
            self.state = Some(
                pollster::block_on(RenderingContext::new(
                    self.proxy.clone(),
                    window,
                    self.new_fn.clone(),
                ))
                .unwrap(),
            );
        }

        #[cfg(target_arch = "wasm32")]
        {
            let new_fn = self.new_fn.clone();
            let proxy = self.proxy.clone();
            wasm_bindgen_futures::spawn_local(async move {
                assert!(
                    // send_event sends it to user_event
                    proxy
                        .send_event(UiEvent::Context(
                            RenderingContext::new(proxy.clone(), window, new_fn)
                                .await
                                .expect("Failed to create canvas!")
                        ))
                        .is_ok()
                )
            });
        }
    }

    #[allow(unused_mut)]
    fn user_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        mut event: UiEvent<RenderingContext<T, R>, T::Event>,
    ) {
        match event {
            UiEvent::Context(mut state) => {
                #[cfg(target_arch = "wasm32")]
                {
                    state.context.window.request_redraw();
                    state.resize(
                        state.context.window.inner_size().width,
                        state.context.window.inner_size().height,
                    );
                }
                self.state = Some(state);
            }
            UiEvent::Event(event) => {
                if let Some(state) = &mut self.state {
                    T::handle_event(&mut state.logic, event);
                }
            }
        }
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
            WindowEvent::Resized(size) => state.resize(size.width, size.height),
            WindowEvent::RedrawRequested => {
                match state.render() {
                    Ok(_) => {}
                    Err(
                        wgpu::SurfaceError::Lost
                        | wgpu::SurfaceError::OutOfMemory
                        | wgpu::SurfaceError::Outdated,
                    ) => {
                        let size = state.context.window.inner_size();
                        state.resize(size.width, size.height)
                    }
                    Err(e) => log::error!("Failed to render: {:?}", e),
                }
            }
            WindowEvent::KeyboardInput { event, .. } => state.handle_key(event_loop, &event),
            WindowEvent::CursorMoved { position, .. } => {
                state.handle_mouse_position(position.x as f32, position.y as f32);
            }
            WindowEvent::MouseInput {
                state: button_state,
                button,
                ..
            } => {
                state.handle_mouse_button(button, button_state);
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let (x, y) = match delta {
                    MouseScrollDelta::LineDelta(x, y) => (x * 32.0, y * 32.0),
                    MouseScrollDelta::PixelDelta(pos) => (pos.x as f32, pos.y as f32),
                };
                state.handle_mouse_wheel(x, y);
            }
            _ => {}
        }
    }
}
