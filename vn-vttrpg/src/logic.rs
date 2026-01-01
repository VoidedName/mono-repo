use std::cell::RefCell;
use std::rc::Rc;
use vn_utils::string::{InsertAtCharIndex, RemoveAtCharIndex};
use vn_vttrpg_ui::{
    Anchor, AnchorLocation, Card, CardParams, DynamicSize, DynamicTextFieldController, Element,
    ElementId, ElementSize, EventManager, Fill, Flex, InputTextFieldController, Padding,
    PaddingParams, SimpleLayoutCache, SizeConstraints, Stack, TextField, TextFieldController,
    TextFieldParams, TextFieldText, TextMetrics, UiContext,
};
use vn_vttrpg_window::graphics::GraphicsContext;
use vn_vttrpg_window::input::InputState;
use vn_vttrpg_window::resource_manager::ResourceManager;
use vn_vttrpg_window::scene_renderer::SceneRenderer;
use vn_vttrpg_window::{Color, StateLogic};
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
            height = height.max(glyph.texture.texture.height() as f32);
        }
        (width, height)
    }

    fn line_height(&self, font: &str, font_size: f32) -> f32 {
        self.rm.line_height(font, font_size)
    }

    fn get_glyphs(
        &self,
        text: &str,
        font: &str,
        font_size: f32,
    ) -> Vec<vn_vttrpg_window::text::Glyph> {
        self.rm.get_glyphs(&self.gc, text, font, font_size)
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
    pub resource_manager: Rc<ResourceManager>,
    pub graphics_context: Rc<GraphicsContext>,
    pub input: InputState,
    fps_stats: Rc<RefCell<FpsStats>>,
    size: (u32, u32),
    mouse_position: (f32, f32),
    ui: Option<RefCell<Box<dyn Element>>>,
    event_manager: Rc<RefCell<EventManager>>,
    input_text_id: ElementId,
    input_text: Rc<RefCell<String>>,
    input_caret: Rc<RefCell<usize>>,
    input_controller: Rc<RefCell<InputTextFieldController>>,
    intended_x: f32,
    last_move_was_vertical: bool,
    caret_width: f32,
}

impl StateLogic<SceneRenderer> for MainLogic {
    async fn new_from_graphics_context(
        graphics_context: Rc<GraphicsContext>,
        resource_manager: Rc<ResourceManager>,
    ) -> anyhow::Result<Self> {
        let diffuse_bytes = include_bytes!("vn_dk_white_square_better_n.png");
        resource_manager.load_texture_from_bytes("vn_dk_white_square", diffuse_bytes)?;

        let font_bytes =
            include_bytes!("../../vn-vttrpg-window/src/text/fonts/JetBrainsMono-Bold.ttf");
        resource_manager.load_font_from_bytes("jetbrains-bold", font_bytes)?;

        let input_text = Rc::new(RefCell::new("".to_string()));
        let input_caret = Rc::new(RefCell::new(0));
        let input_controller = Rc::new(RefCell::new(InputTextFieldController::new(
            Box::new({
                let text = input_text.clone();
                move || text.borrow().clone()
            }),
            Box::new({
                let caret = input_caret.clone();
                move || Some(*caret.borrow())
            }),
        )));

        Ok(Self {
            resource_manager,
            mouse_position: (0.0, 0.0),
            size: graphics_context.size(),
            graphics_context,
            input: InputState::new(),
            fps_stats: Rc::new(RefCell::new(FpsStats::new())),
            ui: None,
            event_manager: Rc::new(RefCell::new(EventManager::new())),
            input_text_id: ElementId(0),
            input_text,
            input_caret,
            input_controller,
            intended_x: 0.0,
            last_move_was_vertical: false,
            caret_width: 2.0,
        })
    }

    fn handle_key(&mut self, event_loop: &ActiveEventLoop, event: &KeyEvent) {
        self.input.handle_key(event);

        let mut event_manager = self.event_manager.borrow_mut();
        let events = event_manager.handle_key(event);

        for (id, interaction_event) in events {
            if id == self.input_text_id {
                if let vn_vttrpg_ui::InteractionEvent::Keyboard(key_event) = interaction_event {
                    if key_event.state.is_pressed() {
                        use winit::keyboard::{Key, NamedKey};

                        let mut text = self.input_text.borrow_mut();
                        let mut caret = self.input_caret.borrow_mut();

                        let controller = self.input_controller.borrow();
                        let layout = controller.current_layout();

                        if !self.last_move_was_vertical {
                            if let Some(layout) = layout {
                                self.intended_x = layout.get_caret_x(*caret);
                            }
                        }

                        match &key_event.logical_key {
                            Key::Character(s) => {
                                text.insert_str_at_char_index(*caret, s);
                                *caret += s.chars().count();
                                if let Some(layout) = layout {
                                    self.intended_x = layout.get_caret_x(*caret);
                                }
                                self.last_move_was_vertical = false;
                            }
                            Key::Named(NamedKey::Space) => {
                                text.insert_at_char_index(*caret, ' ');
                                *caret += 1;
                                if let Some(layout) = layout {
                                    self.intended_x = layout.get_caret_x(*caret);
                                }
                                self.last_move_was_vertical = false;
                            }
                            Key::Named(NamedKey::Backspace) => {
                                if *caret > 0 && *caret <= text.len() {
                                    *caret -= 1;
                                    text.remove_at_char_index(*caret);
                                    if let Some(layout) = layout {
                                        self.intended_x = layout.get_caret_x(*caret);
                                    }
                                }
                                self.last_move_was_vertical = false;
                            }
                            Key::Named(NamedKey::Delete) => {
                                if *caret < text.len() {
                                    text.remove_at_char_index(*caret);
                                    if let Some(layout) = layout {
                                        self.intended_x = layout.get_caret_x(*caret);
                                    }
                                }
                                self.last_move_was_vertical = false;
                            }
                            Key::Named(NamedKey::ArrowLeft) => {
                                if *caret > 0 {
                                    *caret -= 1;
                                    if let Some(layout) = layout {
                                        self.intended_x = layout.get_caret_x(*caret);
                                    }
                                }
                                self.last_move_was_vertical = false;
                            }
                            Key::Named(NamedKey::ArrowRight) => {
                                if *caret < text.len() {
                                    *caret += 1;
                                    if let Some(layout) = layout {
                                        self.intended_x = layout.get_caret_x(*caret);
                                    }
                                }
                                self.last_move_was_vertical = false;
                            }
                            Key::Named(NamedKey::ArrowUp) => {
                                if let Some(layout) = layout {
                                    *caret = layout.get_vertical_move(*caret, -1, self.intended_x);
                                }
                                self.last_move_was_vertical = true;
                            }
                            Key::Named(NamedKey::ArrowDown) => {
                                if let Some(layout) = layout {
                                    *caret = layout.get_vertical_move(*caret, 1, self.intended_x);
                                }
                                self.last_move_was_vertical = true;
                            }
                            Key::Named(NamedKey::Enter) => {
                                text.insert_at_char_index(*caret, '\n');
                                *caret += 1;
                                if let Some(layout) = layout {
                                    self.intended_x = layout.get_caret_x(*caret);
                                }
                                self.last_move_was_vertical = false;
                            }
                            _ => {}
                        }
                    }
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
        use vn_vttrpg_ui::MouseButton;
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
                vn_vttrpg_ui::InteractionEvent::Click { x, y, .. } => {
                    if id == self.input_text_id {
                        let controller = self.input_controller.borrow();
                        let layout = controller.current_layout();
                        if let Some(layout) = layout.as_ref() {
                            // Subtract caret_width / 2.0 because drawing adds it (caret_space / 2.0)
                            // Actually, caret_space is self.caret_width if caret_position.is_some()
                            if let Some(c_pos) = layout.hit_test(x - self.caret_width / 2.0, y) {
                                let mut caret = self.input_caret.borrow_mut();
                                *caret = c_pos;
                                self.intended_x = layout.get_caret_x(*caret);
                                self.last_move_was_vertical = false;
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }

    fn resized(&mut self, width: u32, height: u32) {
        self.size = (width, height);

        let text_metric = Rc::new(TextMetric {
            rm: self.resource_manager.clone(),
            gc: self.graphics_context.clone(),
        });

        let mut event_manager = self.event_manager.borrow_mut();
        let mut ui_ctx = UiContext {
            event_manager: &mut event_manager,
            parent_id: None,
            layout_cache: Box::new(SimpleLayoutCache::new()),
        };

        use vn_vttrpg_ui::AnchorParams;

        let test_input = TextField::new(
            TextFieldParams {
                font: "jetbrains-bold".to_string(),
                font_size: 12.0,
                color: Color::RED,
            },
            self.input_controller.clone(),
            text_metric.clone(),
            &mut ui_ctx,
        );

        self.input_text_id = test_input.id();
        self.caret_width = test_input.caret_width();

        let test_input = Fill::new(Box::new(test_input), &mut ui_ctx);

        let test_input = Padding::new(
            Box::new(test_input),
            PaddingParams {
                pad_left: 10.0,
                pad_right: 10.0,
                pad_top: 5.0,
                pad_bottom: 5.0,
            },
            &mut ui_ctx,
        );

        let test_input = Card::new(
            Box::new(test_input),
            CardParams {
                background_color: Color::BLACK.with_alpha(0.5),
                border_size: 2.0,
                border_color: Color::WHITE,
                corner_radius: 5.0,
            },
            &mut ui_ctx,
        );

        let fps_stats = self.fps_stats.clone();

        let fps = TextField::new(
            TextFieldParams {
                font: "jetbrains-bold".to_string(),
                font_size: 18.0,
                color: Color::WHITE.with_alpha(0.5),
            },
            Rc::new(RefCell::new(DynamicTextFieldController::new(Box::new(
                move || {
                    format!(
                        "FPS: {:8>.2}",
                        fps_stats.borrow().current_fps().unwrap_or(0.0)
                    )
                },
            )))),
            text_metric.clone(),
            &mut ui_ctx,
        );

        let fps = Anchor::new(
            Box::new(fps),
            AnchorParams {
                location: AnchorLocation::TopRight,
            },
            &mut ui_ctx,
        );

        let ui = Anchor::new(
            Box::new(test_input),
            AnchorParams {
                location: AnchorLocation::CENTER,
            },
            &mut ui_ctx,
        );

        let ui = Stack::new(vec![Box::new(ui), Box::new(fps)], &mut ui_ctx);

        self.ui = Some(RefCell::new(Box::new(ui)));
    }

    fn render_target(&self) -> vn_vttrpg_window::scene::Scene {
        self.fps_stats.borrow_mut().tick();
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
                layout_cache: Box::new(SimpleLayoutCache::new()),
            };

            ui.layout(
                &mut ctx,
                SizeConstraints {
                    min_size: ElementSize {
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
                ElementSize {
                    width: self.size.0 as f32,
                    height: self.size.1 as f32,
                },
                &mut scene,
            );
        }

        scene
    }
}
