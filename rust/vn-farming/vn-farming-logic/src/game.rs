use std::rc::Rc;
use winit::event::{ElementState, KeyEvent, MouseButton};
use winit::event_loop::ActiveEventLoop;
use vn_scene::{CloneableScene, ConstructableScene};
use vn_wgpu_window::rendering_context::EventDispatcher;
use vn_wgpu_window::{GraphicsContext, SceneRenderer, StateLogic};
use vn_wgpu_window::resource_manager::ResourceManager;
use crate::PlatformHooks;

pub enum GameEvent {}

pub struct Game<
    S: CloneableScene + ConstructableScene + 'static,
    Platform: PlatformHooks,
> {
    pub dispatcher: Rc<EventDispatcher<Game<S, Platform>, SceneRenderer<S>>>,
    pub platform: Rc<Platform>,
    pub graphics_context: Rc<GraphicsContext>,
    pub resource_manager: Rc<ResourceManager>,
}

impl<
    S: CloneableScene + ConstructableScene + 'static,
    Platform: PlatformHooks,
> Game<S, Platform> {
    pub async fn new(
        dispatcher: Rc<EventDispatcher<Game<S, Platform>, SceneRenderer<S>>>,
        platform: Rc<Platform>,
        graphics_context: Rc<GraphicsContext>,
        resource_manager: Rc<ResourceManager>,
    ) -> anyhow::Result<Self> {
        Ok(Self {
            dispatcher,
            platform,
            graphics_context,
            resource_manager,
        })
    }
}

impl<
    S: CloneableScene + ConstructableScene + 'static,
    Platform: PlatformHooks,
> StateLogic<SceneRenderer<S>> for Game<S, Platform>{
    type Event = GameEvent;

    fn handle_event(&mut self, event: Self::Event) {}

    fn update(&mut self) {
    }

    fn handle_key(&mut self, event_loop: &ActiveEventLoop, event: &KeyEvent) {
    }

    fn handle_mouse_position(&mut self, x: f32, y: f32) {
    }

    fn handle_mouse_button(&mut self, button: MouseButton, state: ElementState) {
    }

    fn handle_mouse_wheel(&mut self, delta_x: f32, delta_y: f32) {
    }

    fn resized(&mut self, width: u32, height: u32) {
    }

    fn render_target(&self) -> S {
        todo!()
    }
}