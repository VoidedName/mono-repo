use crate::logic::game_state::{GameStateEx, MENU_FONT};
use crate::logic::{PlatformHooks, TextMetric};
use crate::map::{Map, MapParams, TileMap};
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Instant;
use vn_scene::{Color, Rect};
use vn_ui::{
    AnchorExt, AnchorLocation, AnchorParams, ButtonExt, ButtonParams, DynamicDimension,
    DynamicSize, Element, ElementId, ElementSize, ElementWorld, EventHandler, EventManager, Flex,
    FlexExt, InteractionEventKind, InteractionState, InteractiveExt, InteractiveParams, PaddingExt,
    PaddingParams, SimpleLayoutCache, SizeConstraints, StackExt, TextField, TextFieldParams,
    TextVisuals, UiContext,
};
use vn_wgpu_window::resource_manager::{ResourceManager, Sampling};
use vn_wgpu_window::{GraphicsContext, WgpuScene};
use winit::event::{ElementState, KeyEvent, MouseButton};
use winit::keyboard::{KeyCode, PhysicalKey};

/// Start menu has the buttons
///
/// Start
/// Load
/// Settings
/// Exit
#[derive(Debug, Clone, Copy, PartialEq)]
enum StartMenuButton {
    Start,
    Load,
    Settings,
    Exit,
}

#[derive(Debug, Clone, Copy)]
pub enum StartMenuEvent {
    StartGame,
    LoadGame,
    Settings,
    Exit,
}

impl StartMenuButton {
    fn to_menu_event(&self) -> StartMenuEvent {
        match self {
            StartMenuButton::Start => StartMenuEvent::StartGame,
            StartMenuButton::Load => StartMenuEvent::LoadGame,
            StartMenuButton::Settings => StartMenuEvent::Settings,
            StartMenuButton::Exit => StartMenuEvent::Exit,
        }
    }

    fn next(&self) -> Self {
        match self {
            StartMenuButton::Start => StartMenuButton::Load,
            StartMenuButton::Load => StartMenuButton::Settings,
            StartMenuButton::Settings => StartMenuButton::Exit,
            StartMenuButton::Exit => StartMenuButton::Exit,
        }
    }

    fn previous(&self) -> Self {
        match self {
            StartMenuButton::Start => StartMenuButton::Start,
            StartMenuButton::Load => StartMenuButton::Start,
            StartMenuButton::Settings => StartMenuButton::Load,
            StartMenuButton::Exit => StartMenuButton::Settings,
        }
    }
}

pub struct StartMenu {
    ui: RefCell<Box<dyn Element<State = StartMenu, Message = StartMenuEvent>>>,
    focused_button: Rc<RefCell<StartMenuButton>>,
    button_ids: Rc<RefCell<Vec<(StartMenuButton, ElementId)>>>,
    event_manager: Rc<RefCell<EventManager>>,
}

impl StartMenu {
    pub async fn new(
        platform: Rc<Box<dyn PlatformHooks>>,
        gc: Rc<GraphicsContext>,
        rm: Rc<ResourceManager>,
    ) -> anyhow::Result<Self> {
        let menu_font = platform
            .load_file("fonts/JetBrainsMono-Regular.ttf".to_string())
            .await?;
        rm.load_font_from_bytes(MENU_FONT, &menu_font)?;

        let mut world = ElementWorld::new();
        let focused_button = Rc::new(RefCell::new(StartMenuButton::Start));
        let button_ids = Rc::new(RefCell::new(Vec::new()));

        let mut buttons: Vec<Box<dyn Element<State = StartMenu, Message = StartMenuEvent>>> =
            Vec::new();

        for btn_type in [
            StartMenuButton::Start,
            StartMenuButton::Load,
            StartMenuButton::Settings,
            StartMenuButton::Exit,
        ] {
            let label = format!("{:?}", btn_type);
            let metrics = Rc::new(TextMetric {
                rm: rm.clone(),
                gc: gc.clone(),
            });
            let local_focused_button = focused_button.clone();

            let button = TextField::new(
                Box::new(move |_| TextFieldParams {
                    visuals: TextVisuals {
                        text: label.clone(),
                        caret_position: None,
                        font: MENU_FONT.to_string(),
                        font_size: 32.0,
                        color: Color::WHITE,
                        caret_width: None,
                        caret_blink_duration: None,
                    },
                    metrics: metrics.clone(),
                    interaction: Default::default(),
                    text_field_action_handler: EventHandler::none(),
                }),
                &mut world,
            )
            .anchor(
                Box::new(|_| AnchorParams {
                    location: AnchorLocation::Center,
                }),
                &mut world,
            )
            .interactive(
                Box::new(|_| InteractiveParams {
                    is_interactive: false,
                }),
                &mut world,
            )
            .button(
                Box::new(move |args| {
                    let is_focused = *local_focused_button.borrow() == btn_type;
                    ButtonParams {
                        background: Color::BLACK.with_alpha(0.5),
                        border_color: if is_focused {
                            Color::RED
                        } else {
                            Color::TRANSPARENT
                        },
                        border_width: 2.0,
                        corner_radius: 4.0,
                        interaction: InteractionState {
                            is_hovered: args.ctx.event_manager.borrow().is_hovered(args.id),
                            is_focused,
                        },
                        on_click: if args.ctx.event_manager.borrow().is_hovered(args.id) {
                            Some(btn_type.to_menu_event())
                        } else {
                            None
                        },
                    }
                }),
                &mut world,
            );

            button_ids.borrow_mut().push((btn_type, button.id()));
            buttons.push(Box::new(
                button.padding(Box::new(|_| PaddingParams::uniform(8.0)), &mut world),
            ) as Box<dyn Element<State = StartMenu, Message = StartMenuEvent>>);
        }

        let ui = Flex::new_column_unweighted(buttons, true, &mut world).anchor(
            Box::new(|_| AnchorParams {
                location: AnchorLocation::Center,
            }),
            &mut world,
        );

        Ok(Self {
            ui: RefCell::new(Box::new(ui)),
            focused_button,
            button_ids,
            event_manager: Rc::new(RefCell::new(EventManager::new())),
        })
    }

    fn handle_event(&self, id: ElementId, event: InteractionEventKind) -> Option<StartMenuEvent> {
        match event {
            InteractionEventKind::Click { .. } => {
                if let Some(btn) = self
                    .button_ids
                    .borrow()
                    .iter()
                    .find(|(_, b_id)| *b_id == id)
                    .map(|(btn, _)| btn)
                {
                    log::info!("Button clicked: {:?}", btn);
                    Some(btn.to_menu_event())
                } else {
                    None
                }
            }
            InteractionEventKind::MouseEnter => {
                if let Some(btn) = self
                    .button_ids
                    .borrow()
                    .iter()
                    .find(|(_, b_id)| *b_id == id)
                    .map(|(btn, _)| btn)
                {
                    *self.focused_button.borrow_mut() = *btn;
                }
                None
            }
            InteractionEventKind::Keyboard(key_event) => self.handle_keyboard(key_event),
            _ => None,
        }
    }

    fn handle_event_no_target(&self, event: InteractionEventKind) -> Option<StartMenuEvent> {
        match event {
            InteractionEventKind::Keyboard(key_event) => self.handle_keyboard(key_event),
            _ => None,
        }
    }

    fn handle_keyboard(&self, key_event: KeyEvent) -> Option<StartMenuEvent> {
        if !key_event.state.is_pressed() {
            return None;
        }

        match key_event.physical_key {
            PhysicalKey::Code(KeyCode::ArrowUp) => {
                let mut current = self.focused_button.borrow_mut();
                *current = current.previous();
                None
            }
            PhysicalKey::Code(KeyCode::ArrowDown) => {
                let mut current = self.focused_button.borrow_mut();
                *current = current.next();
                None
            }
            PhysicalKey::Code(KeyCode::Enter) => {
                let btn = self.focused_button.borrow();
                log::info!("Button clicked via Enter: {:?}", btn);

                Some(btn.to_menu_event())
            }
            _ => None,
        }
    }


}

impl GameStateEx for StartMenu {
    type Event = StartMenuEvent;

    fn process_events(&mut self) -> Option<Self::Event> {
        let events = self.event_manager.borrow_mut().process_events();
        for event in events {
            let menu_event = match event.target {
                Some(target) => self.handle_event(target, event.kind),
                None => self.handle_event_no_target(event.kind),
            };

            if menu_event.is_some() {
                return menu_event;
            }
        }

        None
    }

    fn render_target(&self, size: (f32, f32)) -> WgpuScene {
        let mut scene = WgpuScene::new((size.0, size.1));

        let event_manager = self.event_manager.clone();
        event_manager.borrow_mut().clear_hitboxes();

        let mut ctx = UiContext {
            event_manager,
            parent_id: None,
            layout_cache: Box::new(SimpleLayoutCache::new()),
            interactive: true,
            clip_rect: Rect::NO_CLIP,
            now: Instant::now(),
        };

        self.ui.borrow_mut().layout(
            &mut ctx,
            self,
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

        self.ui.borrow_mut().draw(
            &mut ctx,
            self,
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
        self.event_manager
            .borrow_mut()
            .queue_event(InteractionEventKind::Keyboard(event.clone()));
    }

    fn handle_mouse_position(&mut self, x: f32, y: f32) {
        self.event_manager
            .borrow_mut()
            .queue_event(InteractionEventKind::MouseMove { x, y });
    }

    fn handle_mouse_button(
        &mut self,
        mouse_position: (f32, f32),
        button: MouseButton,
        state: ElementState,
    ) {
        use vn_ui::MouseButton;
        let button = match button {
            winit::event::MouseButton::Left => MouseButton::Left,
            winit::event::MouseButton::Right => MouseButton::Right,
            winit::event::MouseButton::Middle => MouseButton::Middle,
            _ => return,
        };

        let kind = match state {
            ElementState::Pressed => InteractionEventKind::MouseDown {
                button,
                x: mouse_position.0,
                y: mouse_position.1,
            },
            ElementState::Released => InteractionEventKind::MouseUp {
                button,
                x: mouse_position.0,
                y: mouse_position.1,
            },
        };
        self.event_manager.borrow_mut().queue_event(kind);
    }
}
