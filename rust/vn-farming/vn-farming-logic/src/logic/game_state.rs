use crate::logic::{PlatformHooks, TextMetric};
use std::cell::RefCell;
use std::rc::Rc;
use vn_scene::{Color, KeyCode, PhysicalKey};
use vn_ui::{
    AnchorExt, AnchorLocation, AnchorParams, ButtonExt, ButtonParams, Element, ElementId,
    ElementWorld, Flex, InteractionEvent, InteractionEventKind, InteractionState, InteractiveExt,
    InteractiveParams, PaddingExt, PaddingParams, StaticTextFieldController, TextField,
    TextFieldParams, TextVisuals,
};
use vn_wgpu_window::GraphicsContext;
use vn_wgpu_window::resource_manager::ResourceManager;

/// Start menu has the buttons
///
/// Start
/// Load
/// Settings
/// Exit

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StartMenuButton {
    Start,
    Load,
    Settings,
    Exit,
}

pub enum MenuEvent {
    StartGame,
    LoadGame,
    Settings,
    Exit,
}

pub struct StartMenu {
    pub ui: RefCell<Box<dyn Element<State = StartMenu>>>,
    pub focused_button: Rc<RefCell<Option<StartMenuButton>>>,
    pub button_ids: Rc<RefCell<Vec<(StartMenuButton, ElementId)>>>,
}

const MENU_FONT: &str = "menu-font";

impl StartMenu {
    pub async fn new(
        platform: Rc<Box<dyn PlatformHooks>>,
        gtx: Rc<GraphicsContext>,
        rm: Rc<ResourceManager>,
    ) -> anyhow::Result<Self> {
        let menu_font = platform
            .load_file("fonts/JetBrainsMono-Regular.ttf".to_string())
            .await?;
        rm.load_font_from_bytes(MENU_FONT, &menu_font)?;

        let mut world = ElementWorld::new();
        let focused_button = Rc::new(RefCell::new(Some(StartMenuButton::Start)));
        let button_ids = Rc::new(RefCell::new(Vec::new()));

        let mut buttons: Vec<Box<dyn Element<State = StartMenu>>> = Vec::new();

        for btn_type in [
            StartMenuButton::Start,
            StartMenuButton::Load,
            StartMenuButton::Settings,
            StartMenuButton::Exit,
        ] {
            let label = format!("{:?}", btn_type);
            let metrics = Rc::new(TextMetric {
                rm: rm.clone(),
                gc: gtx.clone(),
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
                    controller: Rc::new(RefCell::new(StaticTextFieldController::new())),
                    metrics: metrics.clone(),
                    interaction: Default::default(),
                }),
                &mut world,
            )
            .anchor(
                Box::new(|_| AnchorParams {
                    location: AnchorLocation::CENTER,
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
                    let is_focused = *local_focused_button.borrow() == Some(btn_type);
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
                    }
                }),
                &mut world,
            );

            button_ids.borrow_mut().push((btn_type, button.id()));
            buttons.push(Box::new(
                button.padding(Box::new(|_| PaddingParams::uniform(8.0)), &mut world),
            ));
        }

        let ui = Flex::new_column(buttons, &mut world).anchor(
            Box::new(|_| AnchorParams {
                location: AnchorLocation::CENTER,
            }),
            &mut world,
        );

        Ok(Self {
            ui: RefCell::new(Box::new(ui)),
            focused_button,
            button_ids,
        })
    }

    pub fn handle_event(&self, id: ElementId, event: InteractionEvent) -> Option<MenuEvent> {
        match event.kind {
            InteractionEventKind::Click { .. } => {
                if let Some(btn) = self
                    .button_ids
                    .borrow()
                    .iter()
                    .find(|(_, b_id)| *b_id == id)
                    .map(|(btn, _)| btn)
                {
                    log::info!("Button clicked: {:?}", btn);
                    Some(match btn {
                        StartMenuButton::Start => MenuEvent::StartGame,
                        StartMenuButton::Load => MenuEvent::LoadGame,
                        StartMenuButton::Settings => MenuEvent::Settings,
                        StartMenuButton::Exit => MenuEvent::Exit,
                    })
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
                    *self.focused_button.borrow_mut() = Some(*btn);
                }
                None
            }
            InteractionEventKind::Keyboard(key_event) => self.handle_keyboard(key_event),
            _ => None,
        }
    }

    pub fn handle_event_no_target(&self, event: InteractionEvent) -> Option<MenuEvent> {
        match event.kind {
            InteractionEventKind::Keyboard(key_event) => self.handle_keyboard(key_event),
            _ => None,
        }
    }

    fn handle_keyboard(&self, key_event: vn_scene::KeyEvent) -> Option<MenuEvent> {
        if !key_event.state.is_pressed() {
            return None;
        }

        match key_event.physical_key {
            PhysicalKey::Code(KeyCode::ArrowUp) => {
                let current = *self.focused_button.borrow();
                let next = match current {
                    Some(StartMenuButton::Start) => StartMenuButton::Start,
                    Some(StartMenuButton::Load) => StartMenuButton::Start,
                    Some(StartMenuButton::Settings) => StartMenuButton::Load,
                    Some(StartMenuButton::Exit) => StartMenuButton::Settings,
                    None => StartMenuButton::Start,
                };
                *self.focused_button.borrow_mut() = Some(next);
                None
            }
            PhysicalKey::Code(KeyCode::ArrowDown) => {
                let current = *self.focused_button.borrow();
                let next = match current {
                    Some(StartMenuButton::Start) => StartMenuButton::Load,
                    Some(StartMenuButton::Load) => StartMenuButton::Settings,
                    Some(StartMenuButton::Settings) => StartMenuButton::Exit,
                    Some(StartMenuButton::Exit) => StartMenuButton::Exit,
                    None => StartMenuButton::Start,
                };
                *self.focused_button.borrow_mut() = Some(next);
                None
            }
            PhysicalKey::Code(KeyCode::Enter) => {
                if let Some(btn) = *self.focused_button.borrow() {
                    log::info!("Button clicked via Enter: {:?}", btn);

                    Some(match btn {
                        StartMenuButton::Start => MenuEvent::StartGame,
                        StartMenuButton::Load => MenuEvent::LoadGame,
                        StartMenuButton::Settings => MenuEvent::Settings,
                        StartMenuButton::Exit => MenuEvent::Exit,
                    })
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

pub enum GameState {
    StartMenu(StartMenu),
}
