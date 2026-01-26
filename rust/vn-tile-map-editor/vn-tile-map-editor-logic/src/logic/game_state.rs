use std::cell::RefCell;
use std::rc::Rc;
use std::time::Instant;
use winit::event::ElementState;
use winit::event::KeyEvent;
use winit::event::MouseButton;

pub mod editor;
pub use editor::Editor;

pub mod load_tile_set_menu;
pub use load_tile_set_menu::*;

pub mod ui_helper;
pub use ui_helper::*;

use vn_ui::{DynamicDimension, DynamicSize, Element, ElementSize, EventManager, InteractionEventKind, SimpleLayoutCache, SizeConstraints, UiContext};
use vn_ui::InteractionEventKind::MouseScroll;
use vn_wgpu_window::WgpuScene;

pub trait ApplicationStateEx {
    type StateEvent;
    type State;
    type ApplicationEvent: 'static;

    fn ui(&self) -> &RefCell<Box<dyn Element<State=Self::State, Message=Self::StateEvent>>>;
    fn state(&self) -> &Self::State;
    fn event_manager(&self) -> Rc<RefCell<EventManager>>;
    fn handle_event(&mut self, event: Self::StateEvent) -> Option<Self::ApplicationEvent>;

    fn process_events(&mut self) -> Option<Self::ApplicationEvent> {
        let events = self.event_manager().borrow_mut().process_events();

        let mut ctx = UiContext {
            event_manager: self.event_manager().clone(),
            parent_id: None,
            layout_cache: Box::new(SimpleLayoutCache::new()),
            interactive: true,
            clip_rect: vn_scene::Rect::NO_CLIP,
            now: Instant::now(),
        };

        for event in &events {
            let messages = self.ui().borrow_mut().handle_event(&mut ctx, self.state(), event);
            for msg in messages {
                if let Some(msg) = self.handle_event(msg) {
                    return Some(msg);
                };
            }
        }

        None
    }

    fn render_target(&self, size: (f32, f32)) -> WgpuScene {
        let mut scene = WgpuScene::new((size.0, size.1));

        let event_manager = self.event_manager().clone();
        event_manager.borrow_mut().clear_hitboxes();

        let mut ctx = UiContext {
            event_manager,
            parent_id: None,
            layout_cache: Box::new(SimpleLayoutCache::new()),
            interactive: true,
            clip_rect: vn_scene::Rect::NO_CLIP,
            now: Instant::now(),
        };

        self.ui().borrow_mut().layout(
            &mut ctx,
            self.state(),
            SizeConstraints {
                min_size: ElementSize {
                    width: 0.0,
                    height: 0.0,
                },
                max_size: DynamicSize {
                    width: DynamicDimension::Limit(size.0),
                    height: DynamicDimension::Limit(size.1),
                },
                scene_size: (size.0, size.1),
            },
        );

        self.ui().borrow_mut().draw(
            &mut ctx,
            self.state(),
            (0.0, 0.0),
            ElementSize {
                width: size.0,
                height: size.1,
            },
            &mut scene,
        );

        scene
    }

    fn handle_key(&mut self, event: &KeyEvent) {
        self.event_manager()
            .borrow_mut()
            .queue_event(InteractionEventKind::Keyboard(event.clone()));
    }

    fn handle_mouse_position(&mut self, x: f32, y: f32) {
        self.event_manager()
            .borrow_mut()
            .queue_event(InteractionEventKind::MouseMove {
                x,
                y,
                local_x: x,
                local_y: y,
            });
    }

    fn handle_mouse_button(
        &mut self,
        mouse_position: (f32, f32),
        button: MouseButton,
        state: ElementState,
    ) {
        use vn_ui::MouseButton as UiMouseButton;
        let button = match button {
            MouseButton::Left => UiMouseButton::Left,
            MouseButton::Right => UiMouseButton::Right,
            MouseButton::Middle => UiMouseButton::Middle,
            _ => return,
        };

        let kind = match state {
            ElementState::Pressed => InteractionEventKind::MouseDown {
                button,
                x: mouse_position.0,
                y: mouse_position.1,
                local_x: mouse_position.0,
                local_y: mouse_position.1,
            },
            ElementState::Released => InteractionEventKind::MouseUp {
                button,
                x: mouse_position.0,
                y: mouse_position.1,
                local_x: mouse_position.0,
                local_y: mouse_position.1,
            },
        };
        self.event_manager().borrow_mut().queue_event(kind);
    }

    fn handle_mouse_wheel(&mut self, _delta_x: f32, delta_y: f32) {
        self.event_manager()
            .borrow_mut()
            .queue_event(MouseScroll { y: delta_y })
    }
}

pub enum ApplicationState<ApplicationEvent> {
    Editor(Editor<ApplicationEvent>),
    LoadTileSetMenu(LoadTileSetMenu<ApplicationEvent>),
}

macro_rules! dispatch {
    ($self:ident, $inner:ident, $action:expr) => {
        match $self {
            ApplicationState::Editor($inner) => $action,
            ApplicationState::LoadTileSetMenu($inner) => $action,
        }
    };
}

impl<ApplicationEvent: 'static> ApplicationState<ApplicationEvent> {
    pub fn process_events(&mut self) -> Option<ApplicationEvent> {
        dispatch!(self, inner, inner.process_events())
    }

    pub fn render_target(&self, size: (f32, f32)) -> vn_wgpu_window::scene::WgpuScene {
        dispatch!(self, inner, inner.render_target(size))
    }

    pub fn handle_key(&mut self, event: &KeyEvent) {
        dispatch!(self, inner, inner.handle_key(event))
    }

    pub fn handle_mouse_position(&mut self, x: f32, y: f32) {
        dispatch!(self, inner, inner.handle_mouse_position(x, y))
    }

    pub fn handle_mouse_button(
        &mut self,
        mouse_position: (f32, f32),
        button: MouseButton,
        state: ElementState,
    ) {
        dispatch!(self, inner, inner.handle_mouse_button(mouse_position, button, state))
    }

    pub fn handle_mouse_wheel(&mut self, delta_x: f32, delta_y: f32) {
        dispatch!(self, inner, inner.handle_mouse_wheel(delta_x, delta_y))
    }
}
