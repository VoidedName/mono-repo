use crate::logic::app_state::{
    ApplicationState, ApplicationStateEx, Editor, EditorEvent, LoadTileSetMenu,
    LoadTileSetMenuEvent, LoadTileSetMenuStateWithEditorMemory, LoadedTexture, NewLayerEvent,
    NewLayerMenu, NewLayerMenuStateWithEditorMemory, TryLoadTileSetResult,
};
use std::cell::RefCell;
use std::fmt::Debug;
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;
use thiserror::Error;
use vn_ui::*;
use vn_wgpu_window::graphics::GraphicsContext;
use vn_wgpu_window::resource_manager::{ResourceManager, Sampling};
use vn_wgpu_window::scene_renderer::SceneRenderer;
use vn_wgpu_window::{Renderer, StateLogic};
use web_time::Instant;
use winit::event::KeyEvent;
use winit::event_loop::ActiveEventLoop;

pub mod app_state;
pub mod grid;
pub use grid::*;
use vn_scene::{CloneableScene, ConstructableScene, GenericScene};
use vn_wgpu_window::rendering_context::EventDispatcher;

pub struct TextMetric {
    pub rm: Rc<ResourceManager>,
    pub gc: Rc<GraphicsContext>,
}

impl TextMetrics for TextMetric {
    fn size_of_text(&self, text: &str, font: &str, font_size: f32) -> (f32, f32) {
        let glyphs = self.rm.get_glyphs(&self.gc, text, &font, font_size);
        let mut width = 0.0;
        let mut height: f32 = 0.0;

        if let Some(first) = glyphs.first() {
            width += first.x_bearing;
        }

        for glyph in glyphs {
            width += glyph.advance;
            height = height.max(glyph.size.1);
        }
        (width, height)
    }

    fn line_height(&self, font: &str, font_size: f32) -> f32 {
        self.rm.line_height(font, font_size)
    }

    fn get_glyphs(&self, text: &str, font: &str, font_size: f32) -> Vec<vn_scene::GlyphData> {
        let glyphs = self.rm.get_glyphs(&self.gc, text, font, font_size);
        glyphs
            .into_iter()
            .map(|g| vn_scene::GlyphData {
                texture_id: g.texture.clone(),
                advance: g.advance,
                x_bearing: g.x_bearing,
                y_offset: g.y_offset,
                size: [g.size.0, g.size.1],
                uv_rect: g.uv_rect,
            })
            .collect()
    }
}

pub struct FpsStats {
    key_frame_time: RefCell<Option<Instant>>,
    frame_count: RefCell<u32>,
    current_fps: RefCell<Option<f32>>,
}

impl FpsStats {
    fn new() -> Self {
        Self {
            key_frame_time: RefCell::new(None),
            frame_count: RefCell::new(0),
            current_fps: RefCell::new(None),
        }
    }

    fn tick(&self) {
        let mut key_frame_time = self.key_frame_time.borrow_mut();
        if key_frame_time.is_none() {
            *key_frame_time = Some(Instant::now());
        } else {
            *self.frame_count.borrow_mut() += 1;
        }

        let elapsed = key_frame_time.map(|t| t.elapsed()).unwrap().as_secs_f32();

        if elapsed >= 0.5 {
            let fps = *self.frame_count.borrow() as f32 / elapsed;
            self.current_fps.borrow_mut().replace(fps);
            *key_frame_time = Some(Instant::now());
            *self.frame_count.borrow_mut() = 0;
        }
    }
}

#[derive(Debug, Error)]
pub enum FileLoadingError {
    #[error("{0}")]
    GeneralError(String),
}

#[derive(Clone, Debug)]
pub struct FileDescriptor {
    pub path: String,
    pub name: String,
    pub extension: Option<String>,
}

pub struct File {
    pub descriptor: FileDescriptor,
    pub bytes: Vec<u8>,
}

impl Debug for File {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("File")
            .field("descriptor", &self.descriptor)
            .finish_non_exhaustive()
    }
}

pub trait PlatformHooks: Debug {
    fn execute_async(&self, f: impl Future<Output = ()> + 'static);

    fn has_initialized() {}

    fn load_asset(
        &self,
        path: String,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<Vec<u8>, FileLoadingError>>>>;

    fn exit(&self);

    fn pick_file(&self, extensions: &[&str]) -> Pin<Box<dyn Future<Output = Option<File>>>>;

    fn save_file(
        &self,
        suggested_name: &str,
        extensions: &[&str],
        bytes: &[u8],
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<()>>>>;
}

pub struct EditorCallback<
    Msg,
    S: CloneableScene + ConstructableScene + 'static,
    Platform: PlatformHooks + 'static,
> {
    pub call: Box<dyn Fn(&mut Editor<S, Platform>, Msg)>,
}

impl<Msg, S: CloneableScene + ConstructableScene, Platform: PlatformHooks + 'static> Debug
    for EditorCallback<Msg, S, Platform>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EditorCallback")
            .field("call", &"<closure>")
            .finish()
    }
}

pub enum ApplicationEvent<
    S: CloneableScene + ConstructableScene + 'static,
    Platform: PlatformHooks + 'static,
> {
    TilesetLoaded(TryLoadTileSetResult),
    TilesetReuse(String),
    TilesetLoadCanceled,
    LoadTileset(Vec<String>),
    NewLayer(
        Vec<String>,
        EditorCallback<Option<TryLoadTileSetResult>, S, Platform>,
    ),
    UpdateState(ApplicationState<S, Platform>),
    LoadTilesetFromFile(Option<File>, Vec<String>),
}

impl<S: CloneableScene + ConstructableScene, Platform: PlatformHooks + 'static> Debug
    for ApplicationEvent<S, Platform>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApplicationEvent::TilesetLoaded(result) => {
                f.debug_tuple("TilesetLoaded").field(result).finish()
            }
            ApplicationEvent::TilesetReuse(name) => {
                f.debug_tuple("TilesetReuse").field(name).finish()
            }
            ApplicationEvent::TilesetLoadCanceled => f.write_str("TilesetLoadCanceled"),
            ApplicationEvent::LoadTileset(tilesets) => {
                f.debug_tuple("LoadTileset").field(tilesets).finish()
            }
            ApplicationEvent::NewLayer(tilesets, callback) => f
                .debug_tuple("NewLayer")
                .field(tilesets)
                .field(callback)
                .finish(),
            ApplicationEvent::UpdateState(state) => {
                f.debug_tuple("UpdateState").field(&state.name()).finish()
            }
            ApplicationEvent::LoadTilesetFromFile(file, loaded_tilesets) => f
                .debug_tuple("LoadTilesetFromFile")
                .field(file)
                .field(loaded_tilesets)
                .finish(),
        }
    }
}

pub struct MainLogic<
    S: CloneableScene + ConstructableScene + 'static,
    Platform: PlatformHooks + 'static,
> {
    pub resource_manager: Rc<ResourceManager>,
    pub graphics_context: Rc<GraphicsContext>,
    fps_stats: Rc<RefCell<FpsStats>>,
    size: (u32, u32),
    mouse_position: (f32, f32),
    #[allow(unused)]
    platform: Rc<Platform>,
    app_state: Option<ApplicationState<S, Platform>>,
    dispatcher: Rc<EventDispatcher<MainLogic<S, Platform>, SceneRenderer<S>>>,
    sub_scene_renderer: Rc<RefCell<SceneRenderer<GenericScene>>>,
}

pub struct ApplicationContext<
    S: CloneableScene + ConstructableScene + 'static,
    Platform: PlatformHooks + 'static,
> {
    #[allow(unused)]
    platform: Rc<Platform>,
    #[allow(unused)]
    gv: Rc<GraphicsContext>,
    #[allow(unused)]
    rm: Rc<ResourceManager>,
    #[allow(unused)]
    text_metrics: Rc<TextMetric>,
    #[allow(unused)]
    stats: Rc<RefCell<FpsStats>>,
    dispatcher: Rc<EventDispatcher<MainLogic<S, Platform>, SceneRenderer<S>>>,
}

impl<S: CloneableScene + ConstructableScene, Platform: PlatformHooks + 'static>
    MainLogic<S, Platform>
{
    pub(crate) async fn new(
        dispatcher: Rc<EventDispatcher<MainLogic<S, Platform>, SceneRenderer<S>>>,
        platform: Rc<Platform>,
        graphics_context: Rc<GraphicsContext>,
        resource_manager: Rc<ResourceManager>,
    ) -> anyhow::Result<Self> {
        let font_bytes = include_bytes!("../../assets/fonts/JetBrainsMono-Bold.ttf").to_vec();

        resource_manager.load_font_from_bytes("jetbrains-bold", &font_bytes)?;
        resource_manager.set_glyph_size_increment(4.0);

        let fps_stats = Rc::new(RefCell::new(FpsStats::new()));

        let game_state = ApplicationState::Editor(
            Editor::new(ApplicationContext {
                dispatcher: dispatcher.clone(),
                platform: platform.clone(),
                gv: graphics_context.clone(),
                rm: resource_manager.clone(),
                text_metrics: Rc::new(TextMetric {
                    rm: resource_manager.clone(),
                    gc: graphics_context.clone(),
                }),
                stats: fps_stats.clone(),
            })
            .await?,
        );

        Ok(Self {
            dispatcher,
            sub_scene_renderer: Rc::new(RefCell::new(SceneRenderer::new(
                graphics_context.clone(),
                resource_manager.clone(),
            ))),
            resource_manager,
            mouse_position: (0.0, 0.0),
            size: graphics_context.size(),
            graphics_context,
            fps_stats,
            platform,
            app_state: Some(game_state),
        })
    }

    pub fn update_state(&mut self, new_state: ApplicationState<S, Platform>) {
        self.app_state = Some(new_state);
    }
}

#[derive(Debug)]
pub enum StateLogicDeferredEvent<
    S: CloneableScene + ConstructableScene + 'static,
    Platform: PlatformHooks + 'static,
> {
    ApplicationEvent(ApplicationEvent<S, Platform>),
    Editor(EditorEvent),
    NewLayer(NewLayerEvent),
    LoadTileSetMenu(LoadTileSetMenuEvent),
}

impl<S: CloneableScene + ConstructableScene, T: PlatformHooks> ApplicationState<S, T> {
    pub fn name(&self) -> &'static str {
        match self {
            ApplicationState::Editor(_) => "Editor",
            ApplicationState::LoadTileSetMenu(_) => "Load Tileset",
            ApplicationState::NewLayerMenu(_) => "New Layer",
        }
    }
}

impl<S: CloneableScene + ConstructableScene + 'static, Platform: PlatformHooks + 'static>
    StateLogic<SceneRenderer<S>> for MainLogic<S, Platform>
{
    type Event = StateLogicDeferredEvent<S, Platform>;

    fn handle_event(&mut self, event: Self::Event) {
        match (self.app_state.as_mut(), event) {
            (
                _,
                StateLogicDeferredEvent::ApplicationEvent(ApplicationEvent::UpdateState(state)),
            ) => self.update_state(state),
            (Some(ApplicationState::Editor(editor)), StateLogicDeferredEvent::Editor(event)) => {
                editor.handle_event(event);
            }
            (
                Some(ApplicationState::LoadTileSetMenu(menu)),
                StateLogicDeferredEvent::LoadTileSetMenu(event),
            ) => {
                menu.handle_event(event);
            }
            (
                Some(ApplicationState::NewLayerMenu(menu)),
                StateLogicDeferredEvent::NewLayer(event),
            ) => {
                menu.handle_event(event);
            }
            (None, _) => {
                log::error!("Received event but no state is active");
            }
            (Some(invalid_state), _) => {
                log::error!(
                    "Received invalid event for state {:?}",
                    invalid_state.name()
                );
            }
        }
    }

    fn process_events(&mut self) {
        if let Some(state) = self.app_state.take() {
            match state {
                ApplicationState::Editor(mut editor) => {
                    if let Some(event) = editor.process_events(self.sub_scene_renderer.clone()) {
                        match event {
                            ApplicationEvent::NewLayer(already_loaded, editor_callback) => {
                                return self.update_state(ApplicationState::NewLayerMenu(
                                    NewLayerMenuStateWithEditorMemory {
                                        menu: NewLayerMenu::new(
                                            already_loaded,
                                            ApplicationContext {
                                                dispatcher: self.dispatcher.clone(),
                                                platform: self.platform.clone(),
                                                gv: self.graphics_context.clone(),
                                                rm: self.resource_manager.clone(),
                                                text_metrics: Rc::new(TextMetric {
                                                    rm: self.resource_manager.clone(),
                                                    gc: self.graphics_context.clone(),
                                                }),
                                                stats: self.fps_stats.clone(),
                                            },
                                        ),
                                        editor_callback,
                                        editor,
                                    },
                                ));
                            }
                            _ => {}
                        }
                    }
                    self.update_state(ApplicationState::Editor(editor));
                }
                ApplicationState::LoadTileSetMenu(mut menu) => {
                    if let Some(event) = menu.process_events(self.sub_scene_renderer.clone()) {
                        match event {
                            ApplicationEvent::TilesetLoaded(tiles) => {
                                log::info!("Loaded tiles {:?}", tiles);
                                (menu.editor_callback.call)(&mut menu.editor, Some(tiles));
                                return self.update_state(ApplicationState::Editor(menu.editor));
                            }
                            ApplicationEvent::TilesetLoadCanceled => {
                                log::info!("Load canceled");
                                return self.update_state(ApplicationState::NewLayerMenu(
                                    NewLayerMenuStateWithEditorMemory {
                                        menu: menu.new_layer_menu,
                                        editor_callback: menu.editor_callback,
                                        editor: menu.editor,
                                    },
                                ));
                            }
                            _ => {}
                        }
                    }
                    self.update_state(ApplicationState::LoadTileSetMenu(menu));
                }
                ApplicationState::NewLayerMenu(mut new_menu) => {
                    if let Some(event) = new_menu.process_events(self.sub_scene_renderer.clone()) {
                        match event {
                            ApplicationEvent::TilesetLoadCanceled => {
                                return self
                                    .update_state(ApplicationState::Editor(new_menu.editor));
                            }

                            ApplicationEvent::TilesetReuse(tiles) => {
                                (new_menu.editor_callback.call)(
                                    &mut new_menu.editor,
                                    Some(TryLoadTileSetResult::Reuse(tiles)),
                                );

                                return self
                                    .update_state(ApplicationState::Editor(new_menu.editor));
                            }

                            ApplicationEvent::LoadTilesetFromFile(file, loaded_tilesets) => {
                                match file {
                                    Some(file) => {
                                        let tex = match self
                                            .resource_manager
                                            .load_texture_from_bytes(&file.bytes, Sampling::Nearest)
                                        {
                                            Ok(tex) => tex,
                                            Err(e) => {
                                                log::error!("Failed to load texture: {}", e);
                                                new_menu.set_error(e.to_string());
                                                self.app_state =
                                                    Some(ApplicationState::NewLayerMenu(new_menu));
                                                return;
                                            }
                                        };

                                        return {
                                            self.update_state(ApplicationState::LoadTileSetMenu(
                                                LoadTileSetMenuStateWithEditorMemory {
                                                    editor_callback: new_menu.editor_callback,
                                                    new_layer_menu: new_menu.menu,
                                                    menu: LoadTileSetMenu::new(
                                                        ApplicationContext {
                                                            dispatcher: self.dispatcher.clone(),
                                                            platform: self.platform.clone(),
                                                            gv: self.graphics_context.clone(),
                                                            rm: self.resource_manager.clone(),
                                                            text_metrics: Rc::new(TextMetric {
                                                                rm: self.resource_manager.clone(),
                                                                gc: self.graphics_context.clone(),
                                                            }),
                                                            stats: self.fps_stats.clone(),
                                                        },
                                                        LoadedTexture {
                                                            suggested_name: file
                                                                .descriptor
                                                                .name
                                                                .clone(),
                                                            extension: file.descriptor.extension,
                                                            bytes: Rc::new(RefCell::new(
                                                                file.bytes,
                                                            )),
                                                            id: tex.id.clone(),
                                                            dimensions: tex.size,
                                                        },
                                                        loaded_tilesets,
                                                    )
                                                    .expect("Loading tileset failed"),
                                                    editor: new_menu.editor,
                                                },
                                            ))
                                        };
                                    }
                                    None => {}
                                }
                            }
                            ApplicationEvent::LoadTileset(loaded_tilesets) => {
                                log::info!("Start loading tileset");

                                let file = self.platform.pick_file(&["png", "jpg"]);
                                let dispatcher = self.dispatcher.clone();
                                // make "update state event instead of function"

                                self.platform.execute_async(async move {
                                    let file = file.await;

                                    dispatcher.send_event(
                                        StateLogicDeferredEvent::ApplicationEvent(
                                            ApplicationEvent::LoadTilesetFromFile(
                                                file,
                                                loaded_tilesets.clone(),
                                            ),
                                        ),
                                    );
                                });
                            }
                            _ => {}
                        }
                    }
                    self.update_state(ApplicationState::NewLayerMenu(new_menu));
                }
            }
        }
    }

    fn handle_key(&mut self, _event_loop: &ActiveEventLoop, event: &KeyEvent) {
        self.app_state.as_mut().unwrap().handle_key(event);
    }

    fn handle_mouse_position(&mut self, x: f32, y: f32) {
        self.mouse_position = (x, y);
        self.app_state.as_mut().unwrap().handle_mouse_position(x, y);
    }

    fn handle_mouse_button(
        &mut self,
        button: winit::event::MouseButton,
        state: winit::event::ElementState,
    ) {
        self.app_state
            .as_mut()
            .unwrap()
            .handle_mouse_button(self.mouse_position, button, state);
    }

    fn handle_mouse_wheel(&mut self, delta_x: f32, delta_y: f32) {
        self.app_state
            .as_mut()
            .unwrap()
            .handle_mouse_wheel(delta_x, delta_y);
    }

    fn resized(&mut self, width: u32, height: u32) {
        self.size = (width, height);
    }

    fn render_target(&self) -> S {
        self.resource_manager.update();
        self.fps_stats.borrow_mut().tick();

        let scene = if let Some(state) = self.app_state.as_ref() {
            state.render_target(
                (self.size.0 as f32, self.size.1 as f32),
                self.sub_scene_renderer.clone(),
            )
        } else {
            S::new((self.size.0 as f32, self.size.1 as f32))
        };

        self.resource_manager.cleanup(60, 10000);

        scene
    }
}
