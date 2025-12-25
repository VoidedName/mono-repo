use std::cell::RefCell;
use std::sync::Arc;
use vn_vttrpg_ui::{Anchor, Border, Element, Label, Size, SizeConstraints};
use vn_vttrpg_window::StateLogic;
use vn_vttrpg_window::graphics::GraphicsContext;
use vn_vttrpg_window::input::InputState;
use vn_vttrpg_window::resource_manager::ResourceManager;
use vn_vttrpg_window::scene_renderer::SceneRenderer;
use web_time::Instant;
use winit::event::KeyEvent;
use winit::event_loop::ActiveEventLoop;

/// Tracks and calculates frames per second.
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

    /// Ticks the tracker, updating the FPS value if enough time has passed.
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

            log::info!("FPS:{:>8.2}", fps);

            *key_frame_time = Some(Instant::now());
            *self.frame_count.borrow_mut() = 0;
        }
    }

    fn current_fps(&self) -> Option<f32> {
        self.current_fps.borrow().clone()
    }
}

pub struct MainLogic {
    pub resource_manager: Arc<ResourceManager>,
    pub graphics_context: Arc<GraphicsContext>,
    pub input: InputState,
    fps_stats: FpsStats,
    size: (u32, u32),
}

impl StateLogic<SceneRenderer> for MainLogic {
    async fn new_from_graphics_context(
        graphics_context: Arc<GraphicsContext>,
        resource_manager: Arc<ResourceManager>,
    ) -> anyhow::Result<Self> {
        let diffuse_bytes = include_bytes!("vn_dk_white_square_better_n.png");
        resource_manager.load_texture_from_bytes("vn_dk_white_square", diffuse_bytes)?;

        let font_bytes =
            include_bytes!("../../vn-vttrpg-window/src/text/fonts/JetBrainsMono-Bold.ttf");
        resource_manager.load_font_from_bytes("jetbrains-bold", font_bytes)?;

        Ok(Self {
            resource_manager,
            size: graphics_context.size(),
            graphics_context,
            input: InputState::new(),
            fps_stats: FpsStats::new(),
        })
    }

    fn handle_key(&mut self, event_loop: &ActiveEventLoop, event: &KeyEvent) {
        self.input.handle_key(event);

        use winit::keyboard::{KeyCode, PhysicalKey};
        match (event.physical_key, event.state.is_pressed()) {
            (PhysicalKey::Code(KeyCode::Escape), true) => event_loop.exit(),
            _ => {
                // log::info!("Key: {:?} State: {:?}", event.physical_key, event.state);
            }
        }
    }

    fn resized(&mut self, width: u32, height: u32) {
        self.size = (width, height);
    }

    fn render_target(&self) -> vn_vttrpg_window::scene::Scene {
        self.fps_stats.tick();

        let t = match self.fps_stats.current_fps() {
            Some(fps) => {
                format!("FPS:{:>8.2}", fps)
            }
            None => "Initializing...".to_string(),
        };

        let text = self
            .resource_manager
            .get_or_render_text(&self.graphics_context, &t, "jetbrains-bold", 48.0)
            .unwrap();

        let ui = Label {
            text: t,
            font: "jetbrains-bold".to_string(),
            font_size: 48.0,
            size: Size {
                width: text.texture.width() as f32,
                height: text.texture.height() as f32,
            },
            color: vn_vttrpg_window::Color::WHITE.with_alpha(0.5),
        };

        let ui = Border::new(
            Box::new(ui),
            2.5,
            5.0,
            vn_vttrpg_window::Color::RED.with_alpha(0.5),
        );

        let mut ui = Anchor::new(Box::new(ui), vn_vttrpg_ui::AnchorLocation::TopRight);

        let mut scene = vn_vttrpg_window::scene::Scene::new();

        ui.layout(SizeConstraints {
            min_size: Size {
                width: 0.0,
                height: 0.0,
            },
            max_size: Size {
                width: self.size.0 as f32,
                height: self.size.1 as f32,
            },
        });

        ui.draw(
            (0.0, 0.0),
            Size {
                width: self.size.0 as f32,
                height: self.size.1 as f32,
            },
            &mut scene,
        );

        scene
    }
}
