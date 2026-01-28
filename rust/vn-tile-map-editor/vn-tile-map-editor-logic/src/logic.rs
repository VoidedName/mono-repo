use crate::logic::game_state::{
    ApplicationState, ApplicationStateEx, Editor, LoadTileSetMenu,
    LoadTileSetMenuStateWithEditorMemory, LoadedTexture, NewLayerMenu,
    NewLayerMenuStateWithEditorMemory, TryLoadTileSetResult,
};
use std::cell::RefCell;
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;
use thiserror::Error;
use vn_ui::*;
use vn_wgpu_window::StateLogic;
use vn_wgpu_window::graphics::GraphicsContext;
use vn_wgpu_window::resource_manager::{ResourceManager, Sampling};
use vn_wgpu_window::scene_renderer::SceneRenderer;
use web_time::Instant;
use winit::event::KeyEvent;
use winit::event_loop::ActiveEventLoop;

pub mod game_state;
pub mod grid;
pub use grid::*;

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

        if elapsed >= 0.1 {
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

pub struct File {
    pub name: String,
    pub bytes: Vec<u8>,
}

pub trait PlatformHooks {
    fn load_asset(
        &self,
        path: String,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<Vec<u8>, FileLoadingError>>>>;

    fn load_file(
        &self,
        path: String,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<Vec<u8>, FileLoadingError>>>>;

    fn exit(&self);

    fn pick_file(&self, extensions: &[&str]) -> Option<File>;
}

pub struct EditorCallback<Msg> {
    pub call: Box<dyn Fn(&mut Editor, Msg)>,
}

pub enum ApplicationEvent {
    TilesetLoaded(TryLoadTileSetResult),
    TilesetReuse(String),
    TilesetLoadCanceled,
    LoadTileset(Vec<String>),
    NewLayer(Vec<String>, EditorCallback<Option<TryLoadTileSetResult>>),
}

pub struct MainLogic {
    pub resource_manager: Rc<ResourceManager>,
    pub graphics_context: Rc<GraphicsContext>,
    fps_stats: Rc<RefCell<FpsStats>>,
    size: (u32, u32),
    mouse_position: (f32, f32),
    #[allow(unused)]
    platform: Rc<Box<dyn PlatformHooks>>,
    app_state: Option<ApplicationState>,
}

pub struct ApplicationContext {
    #[allow(unused)]
    platform: Rc<Box<dyn PlatformHooks>>,
    #[allow(unused)]
    gv: Rc<GraphicsContext>,
    #[allow(unused)]
    rm: Rc<ResourceManager>,
    #[allow(unused)]
    text_metrics: Rc<TextMetric>,
    #[allow(unused)]
    stats: Rc<RefCell<FpsStats>>,
}

impl MainLogic {
    pub(crate) async fn new(
        platform: Rc<Box<dyn PlatformHooks>>,
        graphics_context: Rc<GraphicsContext>,
        resource_manager: Rc<ResourceManager>,
    ) -> anyhow::Result<Self> {
        let font_bytes = platform
            .load_asset("fonts/JetBrainsMono-Bold.ttf".to_string())
            .await?;

        resource_manager.load_font_from_bytes("jetbrains-bold", &font_bytes)?;
        resource_manager.set_glyph_size_increment(4.0);

        let fps_stats = Rc::new(RefCell::new(FpsStats::new()));

        let game_state = ApplicationState::Editor(
            Editor::new(ApplicationContext {
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
            resource_manager,
            mouse_position: (0.0, 0.0),
            size: graphics_context.size(),
            graphics_context,
            fps_stats,
            platform,
            app_state: Some(game_state),
        })
    }
}

impl StateLogic<SceneRenderer> for MainLogic {
    fn process_events(&mut self) {
        self.app_state = Some(match self.app_state.take().unwrap() {
            ApplicationState::Editor(mut editor) => {
                if let Some(event) = editor.process_events() {
                    match event {
                        ApplicationEvent::NewLayer(already_loaded, editor_callback) => {
                            ApplicationState::NewLayerMenu(NewLayerMenuStateWithEditorMemory {
                                menu: NewLayerMenu::new(
                                    already_loaded,
                                    ApplicationContext {
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
                            })
                        }
                        _ => ApplicationState::Editor(editor),
                    }
                } else {
                    ApplicationState::Editor(editor)
                }
            }
            ApplicationState::LoadTileSetMenu(mut menu) => {
                if let Some(event) = menu.process_events() {
                    match event {
                        ApplicationEvent::TilesetLoaded(tiles) => {
                            log::info!("Loaded tiles {:?}", tiles);
                            (menu.editor_callback.call)(&mut menu.editor, Some(tiles));
                            ApplicationState::Editor(menu.editor)
                        }
                        ApplicationEvent::TilesetLoadCanceled => {
                            log::info!("Load canceled");
                            (menu.editor_callback.call)(&mut menu.editor, None);
                            ApplicationState::Editor(menu.editor)
                        }
                        _ => ApplicationState::LoadTileSetMenu(menu),
                    }
                } else {
                    ApplicationState::LoadTileSetMenu(menu)
                }
            }
            ApplicationState::NewLayerMenu(mut new_menu) => {
                if let Some(event) = new_menu.process_events() {
                    match event {
                        ApplicationEvent::LoadTileset(loaded_tilesets) => {
                            log::info!("Start loading tileset");

                            let file = self.platform.pick_file(&[
                                // "png", "jpg"
                            ]);
                            match file {
                                Some(file) => {
                                    let tex = match self
                                        .resource_manager
                                        .load_texture_from_bytes(&file.bytes, Sampling::Nearest) {
                                        Ok(tex) => tex,
                                        Err(e) => {
                                            log::error!("Failed to load texture: {}", e);
                                            new_menu.set_error(e.to_string());
                                            self.app_state = Some(ApplicationState::NewLayerMenu(
                                                new_menu,
                                            ));
                                            return;
                                        }
                                    };

                                    ApplicationState::LoadTileSetMenu(pollster::block_on(async {
                                        LoadTileSetMenuStateWithEditorMemory {
                                            editor_callback: new_menu.editor_callback,
                                            menu: LoadTileSetMenu::new(
                                                ApplicationContext {
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
                                                    suggested_name: file.name,
                                                    id: tex.id.clone(),
                                                    dimensions: tex.size,
                                                },
                                                loaded_tilesets,
                                            )
                                            .await
                                            .expect("Loading tileset failed"),
                                            editor: new_menu.editor,
                                        }
                                    }))
                                }
                                None => ApplicationState::NewLayerMenu(new_menu),
                            }
                        }
                        ApplicationEvent::TilesetLoadCanceled => {
                            ApplicationState::Editor(new_menu.editor)
                        }
                        ApplicationEvent::TilesetReuse(tiles) => {
                            (new_menu.editor_callback.call)(
                                &mut new_menu.editor,
                                Some(TryLoadTileSetResult::Reuse(tiles)),
                            );
                            ApplicationState::Editor(new_menu.editor)
                        }
                        _ => ApplicationState::NewLayerMenu(new_menu),
                    }
                } else {
                    ApplicationState::NewLayerMenu(new_menu)
                }
            }
        });
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

    fn render_target(&self) -> vn_wgpu_window::scene::WgpuScene {
        self.resource_manager.update();
        self.fps_stats.borrow_mut().tick();

        let scene = self
            .app_state
            .as_ref()
            .unwrap()
            .render_target((self.size.0 as f32, self.size.1 as f32));

        self.resource_manager.cleanup(60, 10000);

        scene
    }
}
