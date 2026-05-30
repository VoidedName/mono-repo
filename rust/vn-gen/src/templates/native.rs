use anyhow::Result;
use std::fs;
use std::path::Path;

/// Generates the Native (WGPU/vn-ui) target crate.
pub fn create(root: &Path, name: &str) -> Result<()> {
    let crate_name = format!("{}-native", name);
    let path = root.join(&crate_name);
    fs::create_dir_all(path.join("src"))?;

    let cargo_toml = format!(
        r#"[package]
name = "{crate_name}"
version = "0.1.0"
edition = "2024"

[dependencies]
{name}-core = {{ path = "../{name}-core" }}
anyhow = {{ workspace = true }}
winit = {{ workspace = true }}
wgpu = {{ workspace = true }}
env_logger = {{ workspace = true }}
log = {{ workspace = true }}
pollster = {{ workspace = true }}
web-time = {{ workspace = true }}
vn-ui = {{ path = "../../vn-ui" }}
vn-scene = {{ path = "../../vn-scene" }}
vn-wgpu-window = {{ path = "../../vn-wgpu-window" }}
"#
    );
    fs::write(path.join("Cargo.toml"), cargo_toml)?;

    let crate_safe_name = name.replace('-', "_");
    let main_rs = format!(
        r#"use anyhow::Result;
use {crate_safe_name}_core::{{Counter, PlatformHooks}};
use std::rc::Rc;
use std::cell::RefCell;
use web_time::Instant;
use winit::event::{{ElementState, MouseButton}};
use vn_scene::{{Color, ConstructableScene, GenericScene, Rect}};
use vn_ui::{{
    params, Button, ButtonParams, Flex, FlexChild, FlexDirection, FlexParams, 
    Element, ElementWorld, EventHandler, InteractionState, Padding, PaddingParams, 
    TextField, TextFieldParams, TextMetrics, TextVisuals, 
    InteractionEventKind, SimpleLayoutCache, SizeConstraints, UiContext, DynamicSize,
    DynamicDimension, ElementSize, EventManager, ChildElement, InteractiveExt,
}};
use vn_wgpu_window::resource_manager::ResourceManager;
use vn_wgpu_window::graphics::GraphicsContext;
use vn_wgpu_window::{{SceneRenderer, Renderer, StateLogic, init_with_logic}};

struct NativeHooks;
impl PlatformHooks for NativeHooks {{}}

pub struct TextMetric {{
    pub rm: Rc<ResourceManager>,
    pub gc: Rc<GraphicsContext>,
}}

impl TextMetrics for TextMetric {{
    fn size_of_text(&self, text: &str, font: &str, font_size: f32) -> (f32, f32) {{
        let glyphs = self.rm.get_glyphs(&self.gc, text, font, font_size);
        let mut width = 0.0;
        let mut height: f32 = 0.0;

        if let Some(first) = glyphs.first() {{
            width += first.x_bearing;
        }}

        for glyph in glyphs {{
            width += glyph.advance;
            height = height.max(glyph.size.1);
        }}
        (width, height)
    }}

    fn line_height(&self, font: &str, font_size: f32) -> f32 {{
        self.rm.line_height(font, font_size)
    }}

    fn get_glyphs(&self, text: &str, font: &str, font_size: f32) -> Vec<vn_scene::GlyphData> {{
        let glyphs = self.rm.get_glyphs(&self.gc, text, font, font_size);
        glyphs
            .into_iter()
            .map(|g| vn_scene::GlyphData {{
                texture_id: g.texture.clone(),
                advance: g.advance,
                x_bearing: g.x_bearing,
                y_offset: g.y_offset,
                size: [g.size.0, g.size.1],
                uv_rect: g.uv_rect,
            }})
            .collect()
    }}
}}

#[derive(Clone, Debug)]
enum Message {{
    Increment,
    Decrement,
    Reset,
}}

struct AppState {{
    counter: Counter,
    hooks: NativeHooks,
    ui_root: RefCell<Box<dyn Element<State = AppState, Message = Message>>>,
    #[allow(dead_code)]
    world: Rc<RefCell<ElementWorld>>,
    event_manager: Rc<RefCell<EventManager>>,
    sub_scene_renderer: Rc<RefCell<SceneRenderer<GenericScene>>>,
    mouse_position: (f32, f32),
    size: (u32, u32),
}}

impl AppState {{
    fn new(gc: Rc<GraphicsContext>, rm: Rc<ResourceManager>) -> Self {{
        let world = Rc::new(RefCell::new(ElementWorld::new()));
        let event_manager = Rc::new(RefCell::new(EventManager::new()));
        let metrics: Rc<dyn TextMetrics> = Rc::new(TextMetric {{ rm: rm.clone(), gc: gc.clone() }});
        let sub_scene_renderer = Rc::new(RefCell::new(<SceneRenderer<GenericScene> as Renderer>::new(gc.clone(), rm.clone())));
        
        let counter_text = TextField::new(
            {{
                let metrics = metrics.clone();
                params! {{
                    args<AppState>,
                    TextFieldParams {{
                        visuals: TextVisuals {{
                            text: format!("Count: {{}}", args.state.counter.count()),
                            caret_position: None,
                            font: "default".to_string(),
                            font_size: 48.0,
                            color: Color {{ r: 1.0, g: 1.0, b: 0.0, a: 1.0 }},
                            caret_width: None,
                            caret_blink_duration: None,
                        }},
                        metrics: metrics.clone(),
                        interaction: InteractionState {{
                            is_hovered: false,
                            is_focused: false,
                        }},
                        text_field_action_handler: EventHandler::none(),
                    }}
                }}
            }},
            world.clone(),
        );

        let inc_button = Button::new(
            {{
                let metrics = metrics.clone();
                let world = world.clone();
                let child: ChildElement<_, _> = TextField::new(
                    {{
                        let metrics = metrics.clone();
                        params! {{
                            args<AppState>,
                            TextFieldParams {{
                                visuals: TextVisuals {{
                                    text: " + ".to_string(),
                                    caret_position: None,
                                    font: "default".to_string(),
                                    font_size: 32.0,
                                    color: Color {{ r: 1.0, g: 1.0, b: 1.0, a: 1.0 }},
                                    caret_width: None,
                                    caret_blink_duration: None,
                                }},
                                metrics: metrics.clone(),
                                interaction: InteractionState {{
                                    is_hovered: false,
                                    is_focused: false,
                                }},
                                text_field_action_handler: EventHandler::none(),
                            }}
                        }}
                    }},
                    world.clone(),
                )
                .interactive_set(false, world.clone())
                .into();

                params! {{
                    args<AppState>,
                    ButtonParams {{
                        background: if args.ctx.event_manager.borrow().is_hovered(args.id) {{
                            Color {{ r: 0.3, g: 0.8, b: 0.3, a: 1.0 }}
                        }} else {{
                            Color {{ r: 0.2, g: 0.6, b: 0.2, a: 1.0 }}
                        }},
                        border_color: Color {{ r: 1.0, g: 1.0, b: 1.0, a: 1.0 }},
                        border_width: 2.0,
                        corner_radius: 8.0,
                        interaction: InteractionState::default(),
                        child: child.clone(),
                        on_click: EventHandler::from(Message::Increment),
                    }}
                }}
            }},
            world.clone(),
        );

        let dec_button = Button::new(
            {{
                let metrics = metrics.clone();
                let world = world.clone();
                let child: ChildElement<_, _> = TextField::new(
                    {{
                        let metrics = metrics.clone();
                        params! {{
                            args<AppState>,
                            TextFieldParams {{
                                visuals: TextVisuals {{
                                    text: " - ".to_string(),
                                    caret_position: None,
                                    font: "default".to_string(),
                                    font_size: 32.0,
                                    color: Color {{ r: 1.0, g: 1.0, b: 1.0, a: 1.0 }},
                                    caret_width: None,
                                    caret_blink_duration: None,
                                }},
                                metrics: metrics.clone(),
                                interaction: InteractionState {{
                                    is_hovered: false,
                                    is_focused: false,
                                }},
                                text_field_action_handler: EventHandler::none(),
                            }}
                        }}
                    }},
                    world.clone(),
                )
                .interactive_set(false, world.clone())
                .into();

                params! {{
                    args<AppState>,
                    ButtonParams {{
                        background: if args.ctx.event_manager.borrow().is_hovered(args.id) {{
                            Color {{ r: 0.8, g: 0.3, b: 0.3, a: 1.0 }}
                        }} else {{
                            Color {{ r: 0.6, g: 0.2, b: 0.2, a: 1.0 }}
                        }},
                        border_color: Color {{ r: 1.0, g: 1.0, b: 1.0, a: 1.0 }},
                        border_width: 2.0,
                        corner_radius: 8.0,
                        interaction: InteractionState::default(),
                        child: child.clone(),
                        on_click: EventHandler::from(Message::Decrement),
                    }}
                }}
            }},
            world.clone(),
        );

        let reset_button = Button::new(
            {{
                let metrics = metrics.clone();
                let world = world.clone();
                let child: ChildElement<_, _> = TextField::new(
                    {{
                        let metrics = metrics.clone();
                        params! {{
                            args<AppState>,
                            TextFieldParams {{
                                visuals: TextVisuals {{
                                    text: " Reset ".to_string(),
                                    caret_position: None,
                                    font: "default".to_string(),
                                    font_size: 32.0,
                                    color: Color {{ r: 1.0, g: 1.0, b: 1.0, a: 1.0 }},
                                    caret_width: None,
                                    caret_blink_duration: None,
                                }},
                                metrics: metrics.clone(),
                                interaction: InteractionState {{
                                    is_hovered: false,
                                    is_focused: false,
                                }},
                                text_field_action_handler: EventHandler::none(),
                            }}
                        }}
                    }},
                    world.clone(),
                )
                .interactive_set(false, world.clone())
                .into();

                params! {{
                    args<AppState>,
                    ButtonParams {{
                        background: if args.ctx.event_manager.borrow().is_hovered(args.id) {{
                            Color {{ r: 0.5, g: 0.5, b: 0.5, a: 1.0 }}
                        }} else {{
                            Color {{ r: 0.3, g: 0.3, b: 0.3, a: 1.0 }}
                        }},
                        border_color: Color {{ r: 1.0, g: 1.0, b: 1.0, a: 1.0 }},
                        border_width: 2.0,
                        corner_radius: 8.0,
                        interaction: InteractionState::default(),
                        child: child.clone(),
                        on_click: EventHandler::from(Message::Reset),
                    }}
                }}
            }},
            world.clone(),
        );

        let buttons_row = Flex::new(
            {{
                let inc_button = FlexChild::new(inc_button).into_rc_refcell();
                let dec_button = FlexChild::new(dec_button).into_rc_refcell();
                let reset_button = FlexChild::new(reset_button).into_rc_refcell();
                params! {{
                    FlexParams {{
                        direction: FlexDirection::Row,
                        force_orthogonal_same_size: false,
                        children: vec![
                            inc_button.clone(),
                            dec_button.clone(),
                            reset_button.clone(),
                        ],
                    }}
                }}
            }},
            world.clone(),
        );

        let content_column = Flex::new(
            {{
                let counter_text = FlexChild::new(counter_text).into_rc_refcell();
                let buttons_row = FlexChild::new(buttons_row).into_rc_refcell();
                params! {{
                    FlexParams {{
                        direction: FlexDirection::Column,
                        force_orthogonal_same_size: false,
                        children: vec![
                            counter_text.clone(),
                            buttons_row.clone(),
                        ],
                    }}
                }}
            }},
            world.clone(),
        );

        let ui_root = Padding::new(
            content_column,
            params! {{
                PaddingParams {{
                    pad_top: 20.0,
                    pad_bottom: 20.0,
                    pad_left: 20.0,
                    pad_right: 20.0,
                }}
            }},
            world.clone(),
        );

        Self {{
            counter: Counter::new(),
            hooks: NativeHooks,
            ui_root: RefCell::new(Box::new(ui_root)),
            world,
            event_manager,
            sub_scene_renderer,
            mouse_position: (0.0, 0.0),
            size: (800, 600),
        }}
    }}
}}

impl StateLogic<vn_wgpu_window::scene_renderer::SceneRenderer<GenericScene>> for AppState {{
    type Event = Message;

    fn handle_event(&mut self, event: Self::Event) {{
        match event {{
            Message::Increment => self.counter.increment(&self.hooks),
            Message::Decrement => self.counter.decrement(&self.hooks),
            Message::Reset => self.counter.reset(&self.hooks),
        }}
    }}

    fn handle_key(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop, event: &winit::event::KeyEvent) {{
        self.event_manager.borrow_mut().queue_event(InteractionEventKind::Keyboard(event.clone()));
    }}

    fn handle_mouse_position(&mut self, x: f32, y: f32) {{
        self.mouse_position = (x, y);
        self.event_manager.borrow_mut().queue_event(InteractionEventKind::MouseMove {{
            x,
            y,
            local_x: x,
            local_y: y,
        }});
    }}

    fn handle_mouse_wheel(&mut self, _delta_x: f32, delta_y: f32) {{
        self.event_manager.borrow_mut().queue_event(InteractionEventKind::MouseScroll {{ y: delta_y }});
    }}

    fn resized(&mut self, width: u32, height: u32) {{
        self.size = (width, height);
    }}

    fn handle_mouse_button(
        &mut self,
        button: winit::event::MouseButton,
        state: winit::event::ElementState,
    ) {{
        use vn_ui::MouseButton as UiMouseButton;
        let button = match button {{
            MouseButton::Left => UiMouseButton::Left,
            MouseButton::Right => UiMouseButton::Right,
            MouseButton::Middle => UiMouseButton::Middle,
            _ => return,
        }};

        let kind = match state {{
            ElementState::Pressed => InteractionEventKind::MouseDown {{
                button,
                x: self.mouse_position.0,
                y: self.mouse_position.1,
                local_x: self.mouse_position.0,
                local_y: self.mouse_position.1,
            }},
            ElementState::Released => InteractionEventKind::MouseUp {{
                button,
                x: self.mouse_position.0,
                y: self.mouse_position.1,
                local_x: self.mouse_position.0,
                local_y: self.mouse_position.1,
            }},
        }};
        self.event_manager.borrow_mut().queue_event(kind);
    }}

    fn update(&mut self) {{
        let events = self.event_manager.borrow_mut().process_events();
        
        let mut ctx = UiContext {{
            event_manager: self.event_manager.clone(),
            parent_id: None,
            layout_cache: Box::new(SimpleLayoutCache::new()),
            interactive: true,
            clip_rect: Rect::NO_CLIP,
            now: Instant::now(),
            scene_renderer: self.sub_scene_renderer.clone(),
        }};

        for event in events {{
            let messages = self.ui_root.borrow_mut().handle_event(&mut ctx, self, &event);
            for msg in messages {{
                self.handle_event(msg);
            }}
        }}
    }}

    fn render_target(&self) -> GenericScene {{
        let size = (self.size.0 as f32, self.size.1 as f32);
        let mut scene = GenericScene::new(size);

        self.event_manager.borrow_mut().clear_hitboxes();

        let mut ctx = UiContext {{
            event_manager: self.event_manager.clone(),
            parent_id: None,
            layout_cache: Box::new(SimpleLayoutCache::new()),
            interactive: true,
            clip_rect: Rect::NO_CLIP,
            now: Instant::now(),
            scene_renderer: self.sub_scene_renderer.clone(),
        }};

        self.ui_root.borrow_mut().layout(
            &mut ctx,
            self,
            SizeConstraints {{
                min_size: ElementSize {{
                    width: 0.0,
                    height: 0.0,
                }},
                max_size: DynamicSize {{
                    width: DynamicDimension::Limit(size.0),
                    height: DynamicDimension::Limit(size.1),
                }},
                scene_size: size,
            }},
        );

        self.ui_root.borrow_mut().draw(
            &mut ctx,
            self,
            (0.0, 0.0),
            ElementSize {{
                width: size.0,
                height: size.1,
            }},
            &mut scene,
        );

        scene
    }}
}}

fn main() -> Result<()> {{
    env_logger::init();
    
    init_with_logic(
        "{name} Native".to_string(),
        (800.0, 600.0),
        |_dispatcher, gc, rm| async {{
            Ok(AppState::new(gc, rm))
        }},
    )?;

    Ok(())
}}
"#
    );
    fs::write(path.join("src").join("main.rs"), main_rs)?;
    Ok(())
}
