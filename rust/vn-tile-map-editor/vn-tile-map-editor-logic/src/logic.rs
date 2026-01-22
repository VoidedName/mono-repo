use crate::logic::game_state::{GameState, GameStateEx, Editor};
use std::cell::RefCell;
use std::pin::Pin;
use std::rc::Rc;
use std::future::Future;
use thiserror::Error;
use vn_ui::*;
use vn_wgpu_window::StateLogic;
use vn_wgpu_window::graphics::GraphicsContext;
use vn_wgpu_window::resource_manager::ResourceManager;
use vn_wgpu_window::scene_renderer::SceneRenderer;
use web_time::Instant;
use winit::event::KeyEvent;
use winit::event_loop::ActiveEventLoop;

pub mod game_state;

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
                uv_rect: Rect {
                    position: [0.0, 0.0],
                    size: [1.0, 1.0],
                },
            })
            .collect()
    }
}

struct FpsStats {
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

pub trait PlatformHooks {
    fn load_file(
        &self,
        path: String,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<Vec<u8>, FileLoadingError>>>>;

    fn exit(&self);
}

pub struct MainLogic {
    pub resource_manager: Rc<ResourceManager>,
    pub graphics_context: Rc<GraphicsContext>,
    fps_stats: Rc<RefCell<FpsStats>>,
    size: (u32, u32),
    mouse_position: (f32, f32),
    platform: Rc<Box<dyn PlatformHooks>>,
    game_state: GameState,
}

impl MainLogic {
    pub(crate) async fn new(
        platform: Rc<Box<dyn PlatformHooks>>,
        graphics_context: Rc<GraphicsContext>,
        resource_manager: Rc<ResourceManager>,
    ) -> anyhow::Result<Self> {
        let font_bytes = platform
            .load_file("fonts/JetBrainsMono-Bold.ttf".to_string())
            .await?;

        resource_manager.load_font_from_bytes("jetbrains-bold", &font_bytes)?;
        resource_manager.set_glyph_size_increment(12.0);

        let fps_stats = Rc::new(RefCell::new(FpsStats::new()));

        let game_state = GameState::Editor(
            Editor::new(
                platform.clone(),
                graphics_context.clone(),
                resource_manager.clone(),
            )
            .await?,
        );

        Ok(Self {
            resource_manager,
            mouse_position: (0.0, 0.0),
            size: graphics_context.size(),
            graphics_context,
            fps_stats,
            platform,
            game_state,
        })
    }
}

impl StateLogic<SceneRenderer> for MainLogic {
    fn process_events(&mut self) {
        let _ = match &mut self.game_state {
            GameState::Editor(editor) => editor.process_events(),
        };
    }

    fn handle_key(&mut self, _event_loop: &ActiveEventLoop, event: &KeyEvent) {
        self.game_state.handle_key(event);
    }

    fn handle_mouse_position(&mut self, x: f32, y: f32) {
        self.mouse_position = (x, y);
        self.game_state.handle_mouse_position(x, y);
    }

    fn handle_mouse_button(
        &mut self,
        button: winit::event::MouseButton,
        state: winit::event::ElementState,
    ) {
        self.game_state
            .handle_mouse_button(self.mouse_position, button, state);
    }

    fn handle_mouse_wheel(&mut self, delta_x: f32, delta_y: f32) {
        self.game_state.handle_mouse_wheel(delta_x, delta_y);
    }

    fn resized(&mut self, width: u32, height: u32) {
        self.size = (width, height);
    }

    fn render_target(&self) -> vn_wgpu_window::scene::WgpuScene {
        self.resource_manager.update();
        self.fps_stats.borrow_mut().tick();

        let scene = self
            .game_state
            .render_target((self.size.0 as f32, self.size.1 as f32));

        self.resource_manager.cleanup(60, 10000);

        scene
    }
}
