use crate::logic::TextMetric;
use crate::logic::game_state::LoadTileSetMenuEvent;
use std::cell::RefCell;
use std::rc::Rc;
use vn_scene::Color;
use vn_ui::*;

pub struct Input<State: 'static, Event: Clone + 'static> {
    pub id: ElementId,
    pub element: Box<dyn Element<State = State, Message = Event>>,
}

#[derive(Clone, Debug)]
pub struct TextFieldState {
    pub id: ElementId,
    pub text: String,
    pub caret: Option<usize>,
}

pub fn input<State: 'static, Event: Clone + 'static, F>(
    text: F,
    place_holder: Option<impl ToString>,
    font: impl ToString,
    font_size: f32,
    metrics: Rc<TextMetric>,
    handler: EventHandler<TextFieldAction, Event>,
    world: Rc<RefCell<ElementWorld>>,
) -> Input<State, Event>
where
    F: Fn(&State) -> TextFieldState + 'static,
{
    let input = TextField::new(
        {
            let font = font.to_string();
            let place_holder = place_holder.map(|x| x.to_string());
            params! { args =>
                let text = text(args.state);
                let is_focused = args.ctx.event_manager.borrow().is_focused(args.id);
                TextFieldParams {
                    visuals: TextVisuals {
                        color: if text.text.is_empty() && !is_focused { Color::WHITE.with_alpha(0.3) } else { Color::WHITE },
                        text: if text.text.is_empty() && let Some(text) = place_holder.as_ref() && !is_focused { text.clone() } else { text.text },
                        caret_position: text.caret,
                        font: font.clone(),
                        font_size,
                        caret_width: Some(2.0),
                        caret_blink_duration: Some(1.0),
                    },
                    metrics: metrics.clone(),
                    interaction: InteractionState {
                        is_focused,
                        is_hovered: is_focused,
                    },
                    text_field_action_handler: handler.clone(),
                }
            }
        },
        world.clone(),
    );

    let input_id = input.id().clone();

    let input = input
        .padding(params!(PaddingParams::uniform(5.0)), world.clone())
        .card({
                  let input_id = input_id.clone();
                  params!(args =>
                    let is_hovered = args.ctx.event_manager.borrow().is_hovered(input_id);
                    CardParams {
                        background_color: if is_hovered { Color::WHITE.with_alpha(0.15) } else { Color::WHITE.with_alpha(0.1) },
                        border_color: if is_hovered { Color::WHITE } else { Color::WHITE.with_alpha(0.5) },
                        corner_radius: 5.0,
                        border_size: 2.0,
                    })
              },
              world.clone(),
        );

    Input {
        id: input_id,
        element: Box::new(input),
    }
}

pub fn label<State: 'static, Event: Clone + 'static, F>(
    text: F,
    font: impl ToString,
    font_size: f32,
    color: Color,
    metrics: Rc<TextMetric>,
    world: Rc<RefCell<ElementWorld>>,
) -> Box<dyn Element<State = State, Message = Event>>
where
    F: Fn(&State) -> String + 'static,
{
    Box::new(TextField::new(
        {
            let font = font.to_string();
            params! { args =>
                TextFieldParams {
                    visuals: TextVisuals {
                        color,
                        text: text(args.state),
                        caret_position: None,
                        font: font.clone(),
                        font_size,
                        caret_width: Some(2.0),
                        caret_blink_duration: Some(1.0),
                    },
                    metrics: metrics.clone(),
                    interaction: InteractionState::default(),
                    text_field_action_handler: EventHandler::none(),
                }
            }
        },
        world.clone(),
    ))
}

pub fn labelled_input<State: 'static, Event: Clone + 'static, F>(
    text: F,
    label: impl ToString,
    font: impl ToString,
    font_size: f32,
    metrics: Rc<TextMetric>,
    handler: EventHandler<TextFieldAction, Event>,
    world: Rc<RefCell<ElementWorld>>,
) -> Input<State, Event>
where
    F: Fn(&State) -> TextFieldState + 'static,
{
    let font = font.to_string();
    let label = label.to_string();
    let mut input = input(
        text,
        Some(" "),
        font.clone(),
        font_size,
        metrics.clone(),
        handler,
        world.clone(),
    );

    let label = TextField::new(
        {
            params! { args =>
                TextFieldParams {
                    visuals: TextVisuals {
                        color: Color::WHITE.with_alpha(0.5),
                        text: label.clone(),
                        caret_position: None,
                        font: font.clone(),
                        font_size,
                        caret_width: Some(2.0),
                        caret_blink_duration: Some(1.0),
                    },
                    metrics: metrics.clone(),
                    interaction: InteractionState::default(),
                    text_field_action_handler: EventHandler::none(),
                }
            }
        },
        world.clone(),
    )
    .anchor(
        params!(AnchorParams {
            location: AnchorLocation::Left
        }),
        world.clone(),
    );

    let flex = Flex::new(
        {
            let flex_children = vec![
                FlexChild::new(label).into_rc_refcell(),
                FlexChild::new(input.element).into_rc_refcell(),
            ];

            params!(FlexParams {
                direction: FlexDirection::Row,
                force_orthogonal_same_size: true,
                children: flex_children.clone()
            })
        },
        world.clone(),
    );

    input.element = Box::new(flex);

    input
}

pub fn suppress_enter_key() -> fn(ElementId, &InteractionEvent) -> (Vec<LoadTileSetMenuEvent>, bool)
{
    |_, event| match event.kind {
        InteractionEventKind::Keyboard(KeyEvent {
            logical_key: winit::keyboard::Key::Named(winit::keyboard::NamedKey::Enter),
            ..
        }) => (vec![], false),
        _ => (vec![], true),
    }
}

pub fn btn<State: 'static, Event: Clone + 'static, F>(
    text: impl ToString,
    font: impl ToString,
    font_size: f32,
    disabled: F,
    color: impl Fn(&State) -> Color + 'static,
    metrics: Rc<TextMetric>,
    handler: EventHandler<ButtonAction, Event>,
    world: Rc<RefCell<ElementWorld>>,
) -> Box<dyn Element<State = State, Message = Event>>
where
    F: Fn(&State) -> bool + 'static + Clone,
{
    let btn = TextField::new(
        {
            let text = text.to_string();
            let font = font.to_string();
            let disabled = disabled.clone();
            params! {args =>
                TextFieldParams {
                    visuals: TextVisuals {
                        text: text.clone(),
                        caret_position: None,
                        font: font.clone(),
                        font_size,
                        color: if disabled(args.state) { color(args.state).with_alpha(0.5) } else { color(args.state) },
                        caret_width: None,
                        caret_blink_duration: None,
                    },
                    metrics: metrics.clone(),
                    interaction: Default::default(),
                    text_field_action_handler: EventHandler::none(),
                }
            }
        },
        world.clone(),
    )
        .padding(params!(PaddingParams::uniform(5.0)), world.clone())
        .interactive_set(false, world.clone())
        .button({
                    let disabled = disabled.clone();
                    params! { args =>
            let is_hovered = args.ctx.event_manager.borrow().is_hovered(args.id);
            ButtonParams {
                background: if is_hovered && !disabled(args.state) { Color::WHITE.with_alpha(0.15) } else { Color::WHITE.with_alpha(0.1) },
                border_color: if is_hovered && !disabled(args.state) { Color::WHITE } else { Color::WHITE.with_alpha(0.5) },
                border_width: 2.0,
                corner_radius: 5.0,
                interaction: Default::default(),
                on_click: handler.clone(),
            }
                }
                },
                world.clone(),
        ).interactive({
                          let disabled = disabled.clone();
                          params!(args => InteractiveParams {is_interactive: !disabled(args.state)})
                      }, world);

    Box::new(btn)
}

pub struct ListParams<State: 'static, Message: 'static> {
    pub len: usize,
    pub child: Box<
        dyn Fn(&State, usize, Rc<RefCell<ElementWorld>>) -> Rc<RefCell<FlexChild<State, Message>>>
            + 'static,
    >,
}

pub fn list<State: 'static, Message: 'static, F>(
    list_params: F,
    direction: FlexDirection,
    force_orthogonal_same_size: bool,
    world: Rc<RefCell<ElementWorld>>,
) -> Box<dyn Element<State = State, Message = Message>>
where
    F: Fn(&State) -> ListParams<State, Message> + 'static + Clone,
{
    Box::new(Flex::new(
        {
            let world = world.clone();
            params!(args<State> =>
            let params = list_params(args.state);
                FlexParams {
                    direction,
                    force_orthogonal_same_size,
                    children: (0..params.len).map(|idx| (params.child)(args.state, idx, world.clone())).collect(),
            })
        },
        world.clone(),
    ))
}

pub fn empty_texture() -> &'static [u8] {
    /// the bytes of a 1x1 png with one transparent pixel...
    const BYTES: [u8; 564] = [
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44,
        0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1F,
        0x15, 0xC4, 0x89, 0x00, 0x00, 0x01, 0x85, 0x69, 0x43, 0x43, 0x50, 0x49, 0x43, 0x43, 0x20,
        0x70, 0x72, 0x6F, 0x66, 0x69, 0x6C, 0x65, 0x00, 0x00, 0x28, 0x91, 0x7D, 0x91, 0xBD, 0x4B,
        0xC3, 0x50, 0x14, 0xC5, 0x4F, 0x53, 0x4B, 0x45, 0x2A, 0x0E, 0x76, 0x90, 0xE2, 0x90, 0xA1,
        0x3A, 0x59, 0x10, 0x15, 0x71, 0x94, 0x56, 0x2C, 0x82, 0x85, 0xD2, 0x56, 0x68, 0xD5, 0xC1,
        0xE4, 0xA5, 0x5F, 0xD0, 0xA4, 0x21, 0x49, 0x71, 0x71, 0x14, 0x5C, 0x0B, 0x0E, 0x7E, 0x2C,
        0x56, 0x1D, 0x5C, 0x9C, 0x75, 0x75, 0x70, 0x15, 0x04, 0xC1, 0x0F, 0x10, 0xFF, 0x00, 0x71,
        0x52, 0x74, 0x91, 0x12, 0xEF, 0x4B, 0x0A, 0x2D, 0x62, 0xBC, 0xF0, 0x92, 0x1F, 0xE7, 0xDD,
        0x73, 0x78, 0xEF, 0x3E, 0x40, 0x68, 0xD5, 0x98, 0x6A, 0xF6, 0x4D, 0x02, 0xAA, 0x66, 0x19,
        0x99, 0x64, 0x5C, 0xCC, 0x17, 0x56, 0xC5, 0xE0, 0x2B, 0x7C, 0x88, 0x20, 0x44, 0xDF, 0x80,
        0xC4, 0x4C, 0x3D, 0x95, 0x5D, 0xCC, 0xC1, 0xB3, 0xBE, 0xEE, 0xA9, 0x8F, 0xEA, 0x2E, 0xC6,
        0xB3, 0xBC, 0xFB, 0xFE, 0xAC, 0x41, 0xA5, 0x68, 0x32, 0xC0, 0x27, 0x12, 0xCF, 0x33, 0xDD,
        0xB0, 0x88, 0x37, 0x88, 0x67, 0x37, 0x2D, 0x9D, 0xF3, 0x3E, 0x71, 0x98, 0x55, 0x24, 0x85,
        0xF8, 0x9C, 0x78, 0xC2, 0xA0, 0x03, 0x12, 0x3F, 0x72, 0x5D, 0x76, 0xF9, 0x8D, 0x73, 0xD9,
        0x61, 0x81, 0x67, 0x86, 0x8D, 0x5C, 0x26, 0x41, 0x1C, 0x26, 0x16, 0xCB, 0x3D, 0x2C, 0xF7,
        0x30, 0xAB, 0x18, 0x2A, 0xF1, 0x0C, 0x71, 0x54, 0x51, 0x35, 0xCA, 0x17, 0xF2, 0x2E, 0x2B,
        0x9C, 0xB7, 0x38, 0xAB, 0xB5, 0x06, 0xEB, 0x9C, 0x93, 0xDF, 0x30, 0x54, 0xD4, 0x56, 0xB2,
        0x5C, 0xA7, 0x35, 0x8A, 0x24, 0x96, 0x90, 0x42, 0x1A, 0x22, 0x64, 0x34, 0x50, 0x45, 0x0D,
        0x16, 0x62, 0xF4, 0xD7, 0x48, 0x31, 0x91, 0xA1, 0xFD, 0xB8, 0x87, 0x3F, 0xE2, 0xF8, 0xD3,
        0xE4, 0x92, 0xC9, 0x55, 0x05, 0x23, 0xC7, 0x02, 0xEA, 0x50, 0x21, 0x39, 0x7E, 0xF0, 0x37,
        0xF8, 0x3D, 0x5B, 0xB3, 0x34, 0x3D, 0xE5, 0x26, 0x85, 0xE2, 0x40, 0xE0, 0xC5, 0xB6, 0x3F,
        0xC6, 0x80, 0xE0, 0x2E, 0xD0, 0x6E, 0xDA, 0xF6, 0xF7, 0xB1, 0x6D, 0xB7, 0x4F, 0x00, 0xFF,
        0x33, 0x70, 0xA5, 0x75, 0xFD, 0xF5, 0x16, 0x30, 0xF7, 0x49, 0x7A, 0xB3, 0xAB, 0x45, 0x8F,
        0x80, 0xA1, 0x6D, 0xE0, 0xE2, 0xBA, 0xAB, 0xC9, 0x7B, 0xC0, 0xE5, 0x0E, 0x30, 0xF2, 0xA4,
        0x4B, 0x86, 0xE4, 0x48, 0x7E, 0x5A, 0x42, 0xA9, 0x04, 0xBC, 0x9F, 0xD1, 0x33, 0x15, 0x80,
        0xE1, 0x5B, 0x60, 0x60, 0xCD, 0x9D, 0x5B, 0x67, 0x1F, 0xA7, 0x0F, 0x40, 0x8E, 0x66, 0xB5,
        0x7C, 0x03, 0x1C, 0x1C, 0x02, 0xE3, 0x65, 0xCA, 0x5E, 0xF7, 0xB8, 0x77, 0x7F, 0xEF, 0xDC,
        0xFE, 0xED, 0xE9, 0xCC, 0xEF, 0x07, 0x0A, 0x29, 0x72, 0x7D, 0xFA, 0x9A, 0x37, 0x96, 0x00,
        0x00, 0x00, 0x06, 0x62, 0x4B, 0x47, 0x44, 0x00, 0xFF, 0x00, 0xFF, 0x00, 0xFF, 0xA0, 0xBD,
        0xA7, 0x93, 0x00, 0x00, 0x00, 0x09, 0x70, 0x48, 0x59, 0x73, 0x00, 0x00, 0x2E, 0x23, 0x00,
        0x00, 0x2E, 0x23, 0x01, 0x78, 0xA5, 0x3F, 0x76, 0x00, 0x00, 0x00, 0x07, 0x74, 0x49, 0x4D,
        0x45, 0x07, 0xEA, 0x01, 0x1A, 0x0C, 0x01, 0x32, 0xA1, 0x19, 0x95, 0xAA, 0x00, 0x00, 0x00,
        0x19, 0x74, 0x45, 0x58, 0x74, 0x43, 0x6F, 0x6D, 0x6D, 0x65, 0x6E, 0x74, 0x00, 0x43, 0x72,
        0x65, 0x61, 0x74, 0x65, 0x64, 0x20, 0x77, 0x69, 0x74, 0x68, 0x20, 0x47, 0x49, 0x4D, 0x50,
        0x57, 0x81, 0x0E, 0x17, 0x00, 0x00, 0x00, 0x0B, 0x49, 0x44, 0x41, 0x54, 0x08, 0xD7, 0x63,
        0x60, 0x00, 0x02, 0x00, 0x00, 0x05, 0x00, 0x01, 0xE2, 0x26, 0x05, 0x9B, 0x00, 0x00, 0x00,
        0x00, 0x49, 0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
    ];

    &BYTES
}
