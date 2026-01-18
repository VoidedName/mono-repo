use std::cell::RefCell;
use std::pin::Pin;
use std::rc::Rc;
use thiserror::Error;
use vn_ui::{
    Anchor, AnchorLocation, AnchorParams, Card, CardParams, DynamicSize,
    DynamicTextFieldController, Element, ElementSize, EventManager, Fill,
    InputTextFieldController, InputTextFieldControllerExt, Interactive, InteractiveParams,
    Padding, PaddingParams, SimpleLayoutCache, SizeConstraints, Stack, TextField,
    TextFieldCallbacks, TextFieldParams, TextMetrics, TextVisuals, UiContext,
};
use vn_wgpu_window::graphics::GraphicsContext;
use vn_wgpu_window::input::InputState;
use vn_wgpu_window::resource_manager::ResourceManager;
use vn_wgpu_window::scene_renderer::SceneRenderer;
use vn_wgpu_window::{Color, StateLogic};
use web_time::Instant;
use winit::event::KeyEvent;
use winit::event_loop::ActiveEventLoop;

struct TextMetric {
    rm: Rc<ResourceManager>,
    gc: Rc<GraphicsContext>,
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
                uv_rect: vn_scene::Rect {
                    position: [0.0, 0.0],
                    size: [1.0, 1.0],
                },
            })
            .collect()
    }
}

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

            // log::info!("FPS:{:>8.2}", fps);

            *key_frame_time = Some(Instant::now());
            *self.frame_count.borrow_mut() = 0;
        }
    }

    fn current_fps(&self) -> Option<f32> {
        self.current_fps.borrow().clone()
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
}

pub struct MainLogic {
    pub resource_manager: Rc<ResourceManager>,
    pub graphics_context: Rc<GraphicsContext>,
    pub input: InputState,
    fps_stats: Rc<RefCell<FpsStats>>,
    size: (u32, u32),
    mouse_position: (f32, f32),
    ui: RefCell<Box<dyn Element<State = ()>>>,
    event_manager: Rc<RefCell<EventManager>>,
    input_controller: Rc<RefCell<InputTextFieldController>>,
    file_loader: Rc<Box<dyn PlatformHooks>>,
}

impl MainLogic {
    pub(crate) async fn new(
        file_loader: Rc<Box<dyn PlatformHooks>>,
        graphics_context: Rc<GraphicsContext>,
        resource_manager: Rc<ResourceManager>,
    ) -> anyhow::Result<Self> {
        let font_bytes = file_loader
            .load_file("fonts/JetBrainsMono-Bold.ttf".to_string())
            .await?;
        resource_manager.load_font_from_bytes("jetbrains-bold", &font_bytes)?;

        resource_manager.set_glyph_size_increment(12.0);

        let event_manager = Rc::new(RefCell::new(EventManager::new()));
        let input_controller = Rc::new(RefCell::new(InputTextFieldController::new(
            event_manager.borrow_mut().next_id(),
        )));

        let fps_stats = Rc::new(RefCell::new(FpsStats::new()));

        Ok(Self {
            ui: RefCell::new(Box::new(Self::build_ui(
                graphics_context.clone(),
                resource_manager.clone(),
                event_manager.clone(),
                input_controller.clone(),
                fps_stats.clone(),
            ))),
            resource_manager,
            mouse_position: (0.0, 0.0),
            size: graphics_context.size(),
            graphics_context,
            input: InputState::new(),
            fps_stats,
            event_manager,
            input_controller,
            file_loader,
        })
    }
}

impl StateLogic<SceneRenderer> for MainLogic {
    fn handle_key(&mut self, event_loop: &ActiveEventLoop, event: &KeyEvent) {
        self.input.handle_key(event);

        let mut event_manager = self.event_manager.borrow_mut();
        let events = event_manager.handle_key(event);

        for (id, interaction_event) in events {
            if id == self.input_controller.borrow().id {
                if let vn_ui::InteractionEvent::Keyboard(key_event) = interaction_event {
                    self.input_controller.borrow_mut().handle_key(&key_event);
                }
            }
        }

        use winit::keyboard::{KeyCode, PhysicalKey};
        match (event.physical_key, event.state.is_pressed()) {
            (PhysicalKey::Code(KeyCode::Escape), true) => event_loop.exit(),
            _ => {}
        }
    }

    fn handle_mouse_position(&mut self, x: f32, y: f32) {
        self.mouse_position = (x, y);
    }

    fn handle_mouse_button(
        &mut self,
        button: winit::event::MouseButton,
        state: winit::event::ElementState,
    ) {
        use vn_ui::MouseButton;
        let button = match button {
            winit::event::MouseButton::Left => MouseButton::Left,
            winit::event::MouseButton::Right => MouseButton::Right,
            winit::event::MouseButton::Middle => MouseButton::Middle,
            _ => return,
        };

        let mut event_manager = self.event_manager.borrow_mut();
        let events = match state {
            winit::event::ElementState::Pressed => event_manager.handle_mouse_down(
                self.mouse_position.0,
                self.mouse_position.1,
                button,
            ),
            winit::event::ElementState::Released => {
                event_manager.handle_mouse_up(self.mouse_position.0, self.mouse_position.1, button)
            }
        };

        for (id, event) in events {
            match event {
                vn_ui::InteractionEvent::Click { x, y, .. } => {
                    if id == self.input_controller.borrow().id {
                        self.input_controller.borrow_mut().handle_click(x, y);
                    }
                }
                _ => {}
            }
        }
    }

    fn resized(&mut self, width: u32, height: u32) {
        self.size = (width, height);
    }

    fn render_target(&self) -> vn_wgpu_window::scene::WgpuScene {
        self.resource_manager.update();

        self.fps_stats.borrow_mut().tick();
        let mut scene =
            vn_wgpu_window::scene::WgpuScene::new((self.size.0 as f32, self.size.1 as f32));

        let mut ui = self.ui.borrow_mut();

        let mut event_manager = self.event_manager.borrow_mut();
        event_manager.handle_mouse_move(self.mouse_position.0, self.mouse_position.1);
        event_manager.clear_hitboxes();

        let mut ctx = UiContext {
            event_manager: &mut event_manager,
            parent_id: None,
            layout_cache: Box::new(SimpleLayoutCache::new()),
            interactive: true,
            now: Instant::now(),
        };

        ui.layout(
            &mut ctx,
            &(),
            SizeConstraints {
                min_size: ElementSize {
                    width: 0.0,
                    height: 0.0,
                },
                max_size: DynamicSize {
                    width: Some(self.size.0 as f32),
                    height: Some(self.size.1 as f32),
                },
                scene_size: (self.size.0 as f32, self.size.1 as f32),
            },
        );

        ui.draw(
            &mut ctx,
            &(),
            (0.0, 0.0),
            ElementSize {
                width: self.size.0 as f32,
                height: self.size.1 as f32,
            },
            &mut scene,
        );

        self.resource_manager.cleanup(60, 10000);

        scene
    }
}

impl MainLogic {
    fn build_ui(
        graphics_context: Rc<GraphicsContext>,
        resource_manager: Rc<ResourceManager>,
        event_manager: Rc<RefCell<EventManager>>,
        input_controller: Rc<RefCell<InputTextFieldController>>,
        fps_stats: Rc<RefCell<FpsStats>>,
    ) -> impl Element<State = ()> {
        let text_metric = Rc::new(TextMetric {
            rm: resource_manager.clone(),
            gc: graphics_context.clone(),
        });

        let mut event_manager = event_manager.borrow_mut();
        let mut ui_ctx = UiContext {
            event_manager: &mut event_manager,
            parent_id: None,
            layout_cache: Box::new(SimpleLayoutCache::new()),
            interactive: true,
            now: Instant::now(),
        };

        let text_input = TextField::new(
            {
                let input_controller = input_controller.clone();
                let text_metric = text_metric.clone();
                Box::new(move |_, _| {
                    let input = input_controller.borrow();
                    TextFieldParams {
                        visuals: TextVisuals {
                            text: input.text.clone(),
                            caret_position: Some(input.caret),
                            font: "jetbrains-bold".to_string(),
                            font_size: 36.0,
                            color: Color::RED,
                            caret_width: None,
                            caret_blink_duration: None,
                        },
                        controller: input_controller.clone(),
                        metrics: text_metric.clone(),
                    }
                })
            },
            &mut ui_ctx,
        );

        input_controller.borrow_mut().id = text_input.id();

        let text_input = Fill::new(Box::new(text_input), &mut ui_ctx);

        let test_input = Padding::new(
            Box::new(text_input),
            Box::new(|_, _| PaddingParams::uniform(0.0)),
            &mut ui_ctx,
        );

        let test_input = Card::new(
            Box::new(test_input),
            Box::new(|_, _| CardParams {
                background_color: Color::TRANSPARENT,
                border_size: 2.0,
                border_color: Color::TRANSPARENT,
                corner_radius: 5.0,
            }),
            &mut ui_ctx,
        );

        let fps_controller_typed = Rc::new(RefCell::new(DynamicTextFieldController::new(Box::new({
            let fps_stats = fps_stats.clone();
            move || {
                format!(
                    "FPS: {:>6.2}",
                    fps_stats.borrow().current_fps().unwrap_or(0.0)
                )
            }
        }))));

        let fps_controller: Rc<RefCell<dyn TextFieldCallbacks>> = fps_controller_typed.clone();

        let fps = TextField::new(
            {
                let fps_controller_typed = fps_controller_typed.clone();
                let fps_controller = fps_controller.clone();
                let text_metric = text_metric.clone();
                Box::new(move |_, _| {
                    let text = fps_controller_typed.borrow().text();
                    TextFieldParams {
                        visuals: TextVisuals {
                            text,
                            caret_position: None,
                            font: "jetbrains-bold".to_string(),
                            font_size: 18.0,
                            color: Color::WHITE.with_alpha(0.5),
                            caret_width: None,
                            caret_blink_duration: None,
                        },
                        controller: fps_controller.clone(),
                        metrics: text_metric.clone(),
                    }
                })
            },
            &mut ui_ctx,
        );

        let fps = Anchor::new(
            Box::new(fps),
            Box::new(|_, _| AnchorParams {
                location: AnchorLocation::TopRight,
            }),
            &mut ui_ctx,
        );

        let fps = Interactive::new(
            Box::new(fps),
            Box::new(|_, _| InteractiveParams {
                is_interactive: true,
            }),
            &mut ui_ctx,
        );

        let ui = Anchor::new(
            Box::new(test_input),
            Box::new(|_, _| AnchorParams {
                location: AnchorLocation::CENTER,
            }),
            &mut ui_ctx,
        );

        let ui = Stack::new(vec![Box::new(ui), Box::new(fps)], &mut ui_ctx);

        ui
    }
}

