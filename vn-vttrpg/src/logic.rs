use std::cell::RefCell;
use std::sync::Arc;
use vn_vttrpg_ui::{
    Anchor, AnchorLocation, Button, Card, CardParams, ConcreteSize, DynamicSize, Element,
    EventManager, Flex, Label, SizeConstraints, TextMetrics, ToolTip, TooltipParams, UiContext,
};
use vn_vttrpg_window::graphics::GraphicsContext;
use vn_vttrpg_window::input::InputState;
use vn_vttrpg_window::resource_manager::ResourceManager;
use vn_vttrpg_window::scene_renderer::SceneRenderer;
use vn_vttrpg_window::{Color, StateLogic};
use web_time::Duration;
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
    mouse_position: (f32, f32),
    ui: Option<RefCell<Box<dyn Element>>>,
    event_manager: Arc<RefCell<EventManager>>,
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
            mouse_position: (0.0, 0.0),
            size: graphics_context.size(),
            graphics_context,
            input: InputState::new(),
            fps_stats: FpsStats::new(),
            ui: None,
            event_manager: Arc::new(RefCell::new(EventManager::new())),
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

    fn handle_mouse_position(&mut self, x: f32, y: f32) {
        self.mouse_position = (x, y);
    }

    fn resized(&mut self, width: u32, height: u32) {
        self.size = (width, height);

        struct TextMetric {
            rm: Arc<ResourceManager>,
            gc: Arc<GraphicsContext>,
        }

        let text_metric = TextMetric {
            rm: self.resource_manager.clone(),
            gc: self.graphics_context.clone(),
        };

        impl TextMetrics for TextMetric {
            fn size_of_text(&self, text: &str, font: &str, font_size: f32) -> (f32, f32) {
                let txt = self
                    .rm
                    .get_or_render_text(&self.gc, text, &font, font_size)
                    .unwrap();
                (txt.texture.width() as f32, txt.texture.height() as f32)
            }
        }

        use vn_vttrpg_ui::{AnchorParams, ButtonParams, LabelParams};

        let start = Label::new(
            LabelParams {
                text: "Start".to_string(),
                font: "jetbrains-bold".to_string(),
                font_size: 48.0,
                color: Color::WHITE,
            },
            &text_metric,
        );

        let options = Label::new(
            LabelParams {
                text: "Options".to_string(),
                font: "jetbrains-bold".to_string(),
                font_size: 48.0,
                color: Color::WHITE,
            },
            &text_metric,
        );

        let exit = Label::new(
            LabelParams {
                text: "Exit".to_string(),
                font: "jetbrains-bold".to_string(),
                font_size: 48.0,
                color: Color::WHITE,
            },
            &text_metric,
        );

        let mut event_manager = self.event_manager.borrow_mut();
        let mut ui_ctx = UiContext {
            event_manager: &mut event_manager,
            parent_id: None,
        };

        let start = Button::new(
            Box::new(start),
            ButtonParams {
                background: Color::GREEN,
                border_color: Color::WHITE,
                border_width: 2.0,
                corner_radius: 10.0,
            },
            &mut ui_ctx,
        );
        let options = Button::new(
            Box::new(options),
            ButtonParams {
                background: Color::BLUE,
                border_color: Color::WHITE,
                border_width: 2.0,
                corner_radius: 10.0,
            },
            &mut ui_ctx,
        );
        let exit = Button::new(
            Box::new(exit),
            ButtonParams {
                background: Color::RED,
                border_color: Color::WHITE,
                border_width: 2.0,
                corner_radius: 10.0,
            },
            &mut ui_ctx,
        );

        let tooltip1 = Label::new(
            LabelParams {
                text: "Start this thing".to_string(),
                font: "jetbrains-bold".to_string(),
                font_size: 24.0,
                color: Color::WHITE,
            },
            &text_metric,
        );

        let tooltip1 = Card::new(
            Box::new(tooltip1),
            CardParams {
                background_color: Color::BLACK,
                border_size: 2.0,
                border_color: Color::WHITE,
                corner_radius: 10.0,
            },
        );

        let tooltip2 = Label::new(
            LabelParams {
                text: "Tooltip of a tooltip for some reason".to_string(),
                font: "jetbrains-bold".to_string(),
                font_size: 24.0,
                color: Color::WHITE,
            },
            &text_metric,
        );
        let tooltip2 = Card::new(
            Box::new(tooltip2),
            CardParams {
                background_color: Color::BLACK,
                border_size: 2.0,
                border_color: Color::WHITE,
                corner_radius: 10.0,
            },
        );

        let tooltip = ToolTip::new(
            Box::new(tooltip1),
            Box::new(tooltip2),
            TooltipParams {
                hover_delay: Some(Duration::from_secs_f32(0.1)),
                hover_retain: Some(Duration::from_secs_f32(0.25)),
            },
            &mut ui_ctx,
        );

        let start = ToolTip::new(
            Box::new(start),
            Box::new(tooltip),
            TooltipParams {
                hover_delay: Some(Duration::from_secs_f32(0.1)),
                hover_retain: Some(Duration::from_secs_f32(0.25)),
            },
            &mut ui_ctx,
        );

        let start = Anchor::new(
            Box::new(start),
            AnchorParams {
                location: AnchorLocation::CENTER,
            },
        );

        let tooltip = Label::new(
            LabelParams {
                text: "Open the options".to_string(),
                font: "jetbrains-bold".to_string(),
                font_size: 24.0,
                color: Color::WHITE,
            },
            &text_metric,
        );

        let tooltip = Card::new(
            Box::new(tooltip),
            CardParams {
                background_color: Color::BLACK,
                border_size: 2.0,
                border_color: Color::WHITE,
                corner_radius: 10.0,
            },
        );
        
        let options = ToolTip::new(Box::new(options), Box::new(tooltip), TooltipParams {
            hover_delay: Some(Duration::from_secs_f32(0.1)),
            hover_retain: Some(Duration::from_secs_f32(0.25)),
        }, &mut ui_ctx);
        let options = Anchor::new(
            Box::new(options),
            AnchorParams {
                location: AnchorLocation::CENTER,
            },
        );

        let exit = Anchor::new(
            Box::new(exit),
            AnchorParams {
                location: AnchorLocation::CENTER,
            },
        );

        let menu = Flex::new_column(vec![Box::new(start), Box::new(options), Box::new(exit)]);

        let mut ui = Anchor::new(
            Box::new(menu),
            AnchorParams {
                location: AnchorLocation::CENTER,
            },
        );

        self.ui = Some(RefCell::new(Box::new(ui)));
    }

    fn render_target(&self) -> vn_vttrpg_window::scene::Scene {
        self.fps_stats.tick();

        let t = match self.fps_stats.current_fps() {
            Some(fps) => {
                format!("FPS:{:>8.2}", fps)
            }
            None => "Initializing...".to_string(),
        };

        let _text = self
            .resource_manager
            .get_or_render_text(&self.graphics_context, &t, "jetbrains-bold", 48.0)
            .unwrap();

        let mut scene =
            vn_vttrpg_window::scene::Scene::new((self.size.0 as f32, self.size.1 as f32));

        if let Some(ui) = &self.ui {
            let mut ui = ui.borrow_mut();

            let mut event_manager = self.event_manager.borrow_mut();
            event_manager.handle_mouse_move(self.mouse_position.0, self.mouse_position.1);
            event_manager.clear_hitboxes();

            let mut ctx = UiContext {
                event_manager: &mut event_manager,
                parent_id: None,
            };

            ui.layout(
                &mut ctx,
                SizeConstraints {
                    min_size: ConcreteSize {
                        width: 0.0,
                        height: 0.0,
                    },
                    max_size: DynamicSize {
                        width: Some(self.size.0 as f32),
                        height: Some(self.size.1 as f32),
                    },
                    scene_size: scene.scene_size(),
                },
            );

            ui.draw(
                &mut ctx,
                (0.0, 0.0),
                ConcreteSize {
                    width: self.size.0 as f32,
                    height: self.size.1 as f32,
                },
                &mut scene,
            );
        }

        scene
    }
}
