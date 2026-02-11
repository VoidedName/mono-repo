use std::cell::RefCell;
use std::fmt::{Debug, Formatter};
use std::rc::Rc;
use web_time::Instant;
use winit::event::ElementState;
use winit::event::KeyEvent;
use winit::event::MouseButton;

pub mod editor;
pub use editor::*;

pub mod load_tileset_menu;
pub use load_tileset_menu::*;

pub mod new_layer_menu;
pub use new_layer_menu::*;

pub mod ui_helper;
use crate::logic::{ApplicationEvent, EditorCallback, PlatformHooks};
pub use ui_helper::*;
use vn_scene::{CloneableScene, ConstructableScene, GenericScene, Scene, TextureId};
use vn_ui::InteractionEventKind::MouseScroll;
use vn_ui::{
    DynamicDimension, DynamicSize, Element, ElementSize, EventManager, InteractionEventKind,
    SimpleLayoutCache, SizeConstraints, UiContext,
};
use vn_wgpu_window::SceneRenderer;

pub struct GeneralSceneSubRendererHook {
    scene_renderer: Rc<RefCell<SceneRenderer<GenericScene>>>,
}

pub trait ApplicationStateEx {
    type StateEvent;
    type State;
    type ApplicationEvent: 'static;
    type Scene: ConstructableScene + 'static;

    fn ui(&self) -> &RefCell<Box<dyn Element<State = Self::State, Message = Self::StateEvent>>>;
    fn state(&self) -> &Self::State;
    fn event_manager(&self) -> Rc<RefCell<EventManager>>;
    fn handle_event(&mut self, event: Self::StateEvent) -> Option<Self::ApplicationEvent>;

    fn update(&mut self) {}

    fn process_events(&mut self, sub_scene_renderer: Rc<RefCell<SceneRenderer<GenericScene>>>) -> Option<Self::ApplicationEvent> {
        self.update();

        let events = self.event_manager().borrow_mut().process_events();

        let mut ctx = UiContext {
            event_manager: self.event_manager().clone(),
            parent_id: None,
            layout_cache: Box::new(SimpleLayoutCache::new()),
            interactive: true,
            clip_rect: vn_scene::Rect::NO_CLIP,
            now: Instant::now(),
            scene_renderer: sub_scene_renderer,
        };

        for event in &events {
            let messages = self
                .ui()
                .borrow_mut()
                .handle_event(&mut ctx, self.state(), event);
            for msg in messages {
                if let Some(msg) = self.handle_event(msg) {
                    return Some(msg);
                };
            }
        }

        None
    }

    fn render_target(
        &self,
        size: (f32, f32),
        sub_scene_renderer: Rc<RefCell<SceneRenderer<GenericScene>>>,
    ) -> Self::Scene {
        let mut scene = Self::Scene::new((size.0, size.1));

        let event_manager = self.event_manager().clone();
        event_manager.borrow_mut().clear_hitboxes();

        let mut ctx = UiContext {
            event_manager,
            parent_id: None,
            layout_cache: Box::new(SimpleLayoutCache::new()),
            interactive: true,
            clip_rect: vn_scene::Rect::NO_CLIP,
            now: Instant::now(),
            scene_renderer: sub_scene_renderer,
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

#[derive(Clone, Debug)]
pub enum TryLoadTileSetResult {
    Loaded(LoadedTileSet),
    Reuse(String),
}

#[derive(Clone)]
pub struct LoadedTileSet {
    name: String,
    extension: Option<String>,
    texture_id: TextureId,
    texture_dimensions: (u32, u32),
    tile_dimensions: (u32, u32),
    bytes: Rc<RefCell<Vec<u8>>>,
}

impl LoadedTileSet {
    pub fn file_name(&self) -> String {
        format!(
            "{}{}",
            self.name,
            self.extension
                .as_ref()
                .map(|ext| format!(".{}", ext))
                .unwrap_or_default()
        )
    }
}

impl Debug for LoadedTileSet {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LoadedTileSet")
            .field("name", &self.name)
            .field("extension", &self.extension)
            .field("texture_id", &self.texture_id)
            .field("texture_dimensions", &self.texture_dimensions)
            .field("tile_dimensions", &self.tile_dimensions)
            .field("bytes", &"<bytes>")
            .finish()
    }
}

pub struct LoadTileSetMenuStateWithEditorMemory<
    S: CloneableScene + ConstructableScene + 'static,
    Platform: PlatformHooks + 'static,
> {
    pub menu: LoadTileSetMenu<S, Platform>,
    pub new_layer_menu: NewLayerMenu<S, Platform>,
    pub editor: Editor<S, Platform>,
    pub editor_callback: EditorCallback<Option<TryLoadTileSetResult>, S, Platform>,
}

pub struct NewLayerMenuStateWithEditorMemory<
    S: CloneableScene + ConstructableScene + 'static,
    Platform: PlatformHooks + 'static,
> {
    pub menu: NewLayerMenu<S, Platform>,
    pub editor: Editor<S, Platform>,
    pub editor_callback: EditorCallback<Option<TryLoadTileSetResult>, S, Platform>,
}

impl<S: CloneableScene + ConstructableScene, Platform: PlatformHooks>
    NewLayerMenuStateWithEditorMemory<S, Platform>
{
    pub fn set_error(&mut self, error: String) {
        self.menu.set_error(error);
    }
}

pub enum ApplicationState<
    S: CloneableScene + ConstructableScene + 'static,
    Platform: PlatformHooks + 'static,
> {
    Editor(Editor<S, Platform>),
    NewLayerMenu(NewLayerMenuStateWithEditorMemory<S, Platform>),
    LoadTileSetMenu(LoadTileSetMenuStateWithEditorMemory<S, Platform>),
}

macro_rules! dispatch {
    ($self:ident, $inner:ident, $action:expr) => {
        match $self {
            ApplicationState::LoadTileSetMenu($inner) => $action,
            ApplicationState::Editor($inner) => $action,
            ApplicationState::NewLayerMenu($inner) => $action,
        }
    };
}

impl<S: CloneableScene + ConstructableScene, Platform: PlatformHooks + 'static>
    ApplicationState<S, Platform>
{
    pub fn process_events(&mut self, sub_scene_renderer: Rc<RefCell<SceneRenderer<GenericScene>>>) -> Option<ApplicationEvent<S, Platform>> {
        dispatch!(self, inner, inner.process_events(sub_scene_renderer))
    }

    pub fn render_target(&self, size: (f32, f32), sub_scene_renderer: Rc<RefCell<SceneRenderer<GenericScene>>>) -> S {
        dispatch!(self, inner, inner.render_target(size, sub_scene_renderer))
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
        dispatch!(
            self,
            inner,
            inner.handle_mouse_button(mouse_position, button, state)
        )
    }

    pub fn handle_mouse_wheel(&mut self, delta_x: f32, delta_y: f32) {
        dispatch!(self, inner, inner.handle_mouse_wheel(delta_x, delta_y))
    }
}

impl<S: CloneableScene + ConstructableScene + 'static, Platform: PlatformHooks + 'static>
    ApplicationStateEx for LoadTileSetMenuStateWithEditorMemory<S, Platform>
{
    type StateEvent = LoadTileSetMenuEvent;
    type State = LoadTileSetMenuState;
    type ApplicationEvent = ApplicationEvent<S, Platform>;
    type Scene = S;

    fn ui(&self) -> &RefCell<Box<dyn Element<State = Self::State, Message = Self::StateEvent>>> {
        self.menu.ui()
    }

    fn state(&self) -> &Self::State {
        self.menu.state()
    }

    fn render_target(&self, size: (f32, f32), sub_scene_renderer: Rc<RefCell<SceneRenderer<GenericScene>>>) -> S {
        let mut menu = self.menu.render_target(size, sub_scene_renderer.clone());
        let mut new_menu = self.new_layer_menu.render_target(size, sub_scene_renderer.clone());
        let mut editor = self.editor.render_target(size, sub_scene_renderer.clone());
        editor.extend(&mut new_menu);
        editor.extend(&mut menu);
        editor
    }

    fn event_manager(&self) -> Rc<RefCell<EventManager>> {
        self.menu.event_manager()
    }

    fn handle_event(&mut self, event: Self::StateEvent) -> Option<Self::ApplicationEvent> {
        self.menu.handle_event(event)
    }
}

impl<S: CloneableScene + ConstructableScene + 'static, Platform: PlatformHooks + 'static>
    ApplicationStateEx for NewLayerMenuStateWithEditorMemory<S, Platform>
{
    type StateEvent = NewLayerEvent;
    type State = NewLayerState;
    type ApplicationEvent = ApplicationEvent<S, Platform>;
    type Scene = S;

    fn ui(&self) -> &RefCell<Box<dyn Element<State = Self::State, Message = Self::StateEvent>>> {
        self.menu.ui()
    }

    fn render_target(&self, size: (f32, f32), sub_scene_renderer: Rc<RefCell<SceneRenderer<GenericScene>>>) -> Self::Scene {
        let mut menu = self.menu.render_target(size, sub_scene_renderer.clone());
        self.editor.event_manager().borrow_mut().clear_hitboxes();
        let mut editor = self.editor.render_target(size, sub_scene_renderer.clone());
        editor.extend(&mut menu);
        editor
    }

    fn state(&self) -> &Self::State {
        self.menu.state()
    }

    fn event_manager(&self) -> Rc<RefCell<EventManager>> {
        self.menu.event_manager()
    }

    fn handle_event(&mut self, event: Self::StateEvent) -> Option<Self::ApplicationEvent> {
        self.menu.handle_event(event)
    }
}
