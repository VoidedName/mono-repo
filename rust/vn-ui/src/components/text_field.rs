use crate::text::layout::TextLayout;
use crate::utils::ToArray;
use crate::{
    DynamicDimension, ElementId, ElementImpl, ElementSize, ElementWorld, EventHandler,
    InteractionState, Interpolatable, SizeConstraints, StateToParams, TextFieldAction, TextMetrics,
    UiContext,
};
use std::rc::Rc;
use vn_scene::{BoxPrimitiveData, Color, Rect, Scene, TextPrimitiveData, Transform};
use web_time::Instant;

#[derive(Clone, PartialEq, Interpolatable)]
pub struct TextVisuals {
    #[interpolate_snappy = "snap_middle"]
    pub text: String,
    #[interpolate_snappy = "snap_middle"]
    pub caret_position: Option<usize>,
    #[interpolate_snappy = "snap_middle"]
    pub font: String,
    pub font_size: f32,
    pub color: Color,
    #[interpolate_none_as_default]
    pub caret_width: Option<f32>,
    #[interpolate_none_as_default]
    pub caret_blink_duration: Option<f32>,
}

#[derive(Clone)]
pub struct TextFieldParams<Message: Clone> {
    pub visuals: TextVisuals,
    pub metrics: Rc<dyn TextMetrics>,
    pub interaction: InteractionState,
    pub text_field_action_handler: EventHandler<TextFieldAction, Message>,
}

impl<Message: Clone> Interpolatable for TextFieldParams<Message> {
    fn interpolate(&self, other: &Self, t: f32) -> Self {
        Self {
            visuals: self.visuals.interpolate(&other.visuals, t),
            metrics: other.metrics.clone(),
            interaction: other.interaction.clone(),
            text_field_action_handler: other.text_field_action_handler.clone(),
        }
    }
}

pub struct TextField<State, Message: Clone> {
    id: ElementId,
    params: StateToParams<State, TextFieldParams<Message>>,
    visuals: Option<TextVisuals>,
    layout: Option<TextLayout>,
    size: ElementSize,
    gained_focus_at: Option<Instant>,
    show_caret: bool,
    line_height: f32,
    last_max_width: Option<f32>,
    _phantom: std::marker::PhantomData<Message>,
}

impl<State, Message: Clone> TextField<State, Message> {
    pub fn new(
        params: StateToParams<State, TextFieldParams<Message>>,
        world: &mut ElementWorld,
    ) -> Self {
        Self {
            id: world.next_id(),
            line_height: 0.0,
            visuals: None,
            layout: None,
            params,
            show_caret: false,
            gained_focus_at: None,
            size: ElementSize::ZERO,
            last_max_width: None,
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn update_state(
        &mut self,
        state: &State,
        max_width: DynamicDimension,
        ctx: &UiContext,
    ) -> bool {
        let params = (self.params)(crate::StateToParamsArgs {
            state,
            id: self.id,
            ctx,
        });
        let mut changed = self.visuals.as_ref() != Some(&params.visuals);

        let max_width_opt = match max_width {
            DynamicDimension::Hint(_) => None,
            DynamicDimension::Limit(v) => Some(v),
        };

        if max_width_opt != self.last_max_width {
            self.last_max_width = max_width_opt;
            changed = true;
        }

        if changed {
            self.line_height = params
                .metrics
                .line_height(&params.visuals.font, params.visuals.font_size);
            let caret_space = params.visuals.caret_width.unwrap_or(2.0);
            let layout = TextLayout::layout(
                &params.visuals.text,
                &params.visuals.font,
                params.visuals.font_size,
                max_width.map(|w| w - caret_space).to_option(),
                params.metrics.as_ref(),
            );

            self.size = ElementSize {
                width: layout.total_width + caret_space,
                height: layout.total_height.max(self.line_height),
            };
            self.layout = Some(layout);
            self.visuals = Some(params.visuals.clone());
        }
        changed
    }
}

impl<State, Message: Clone> ElementImpl for TextField<State, Message> {
    type State = State;
    type Message = Message;

    fn id_impl(&self) -> ElementId {
        self.id
    }

    fn layout_impl(
        &mut self,
        ctx: &mut UiContext,
        state: &Self::State,
        constraints: SizeConstraints,
    ) -> ElementSize {
        self.update_state(state, constraints.max_size.width, ctx);

        let params = (self.params)(crate::StateToParamsArgs {
            state,
            id: self.id,
            ctx,
        });

        let is_focused = params.interaction.is_focused;
        let caret_blink_duration = self
            .visuals
            .as_ref()
            .and_then(|v| v.caret_blink_duration)
            .unwrap_or(1.0);

        match (is_focused, self.gained_focus_at) {
            (false, _) => {
                self.gained_focus_at = None;
                self.show_caret = false;
            }
            (true, None) => {
                self.gained_focus_at = Some(Instant::now());
                self.show_caret = true;
            }
            (true, Some(start_at)) => {
                let elapsed = start_at.elapsed().as_secs_f32();
                self.show_caret = elapsed % caret_blink_duration < caret_blink_duration / 2.0;
            }
        }

        self.size.clamp_to_constraints(constraints)
    }

    fn draw_impl(
        &mut self,
        ctx: &mut UiContext,
        state: &Self::State,
        origin: (f32, f32),
        size: ElementSize,
        scene: &mut dyn Scene,
    ) {
        let params = (self.params)(crate::StateToParamsArgs {
            state,
            id: self.id,
            ctx,
        });
        let visuals = &params.visuals;

        let caret_height = self.line_height * 0.8;
        let caret_y_extra_offset = self.line_height / 2.0 - caret_height / 2.0;
        let caret_width = self
            .visuals
            .as_ref()
            .map(|v| v.caret_width)
            .flatten()
            .unwrap_or(2.0);

        let caret_space = caret_width;

        ctx.with_hitbox_hierarchy(
            self.id,
            scene.current_layer_id(),
            Rect {
                position: origin.to_array(),
                size: size.to_array(),
            },
            |ctx| {
                let clip_rect = ctx.clip_rect;
                if let Some(layout) = &self.layout {
                    for (i, line) in layout.lines.iter().enumerate() {
                        let line_y_offset = i as f32 * self.line_height;

                        let mut glyphs = Vec::new();
                        let mut current_x = 0.0;
                        for glyph in &line.glyphs {
                            glyphs.push(vn_scene::GlyphInstanceData {
                                texture_id: glyph.texture_id.clone(),
                                position: [current_x + glyph.x_bearing, glyph.y_offset],
                                size: glyph.size,
                                uv_rect: glyph.uv_rect,
                            });
                            current_x += glyph.advance;
                        }

                        scene.add_text(TextPrimitiveData {
                            transform: Transform {
                                translation: [
                                    origin.0 + caret_space / 2.0,
                                    origin.1 + line_y_offset,
                                ],
                                ..Transform::DEFAULT
                            },
                            tint: visuals.color,
                            glyphs,
                            clip_rect,
                        });
                    }

                    if self.show_caret {
                        if let Some(caret_position) = visuals.caret_position {
                            scene.with_next_layer(&mut |canvas| {
                                let (caret_x_offset, caret_y_offset) =
                                    layout.get_caret_pos(caret_position);

                                let caret_x = origin.0 + caret_x_offset + caret_width / 2.0;
                                let caret_y = origin.1 + caret_y_offset + caret_y_extra_offset;

                                canvas.add_box(BoxPrimitiveData {
                                    transform: Transform {
                                        translation: [caret_x, caret_y],
                                        ..Transform::DEFAULT
                                    },
                                    size: [caret_width, caret_height],
                                    color: visuals.color,
                                    border_color: Color::TRANSPARENT,
                                    border_thickness: 0.0,
                                    border_radius: 0.0,
                                    clip_rect,
                                });
                            });
                        }
                    }
                }
            },
        );
    }

    fn handle_event_impl(
        &mut self,
        ctx: &mut UiContext,
        state: &Self::State,
        event: &crate::InteractionEvent,
    ) -> Vec<Self::Message> {
        let params = (self.params)(crate::StateToParamsArgs {
            state,
            id: self.id,
            ctx,
        });

        if event.target != Some(self.id) {
            return Vec::new();
        }

        let messages = params.text_field_action_handler.handle(self.id, event, || match &event.kind {
            crate::InteractionEventKind::Click { local_x, local_y, .. } => {
                if let Some(layout) = &self.layout {
                    if let Some(c_pos) = layout.hit_test(*local_x, *local_y) {
                        return vec![TextFieldAction::CaretMove(c_pos)];
                    }
                }
                vec![]
            }
            crate::InteractionEventKind::Keyboard(key_event) => {
                if key_event.state.is_pressed() {
                    use vn_utils::string::{InsertAtCharIndex, RemoveAtCharIndex};
                    use winit::keyboard::{Key, NamedKey};

                    let current_text = &params.visuals.text;
                    let caret = params.visuals.caret_position.unwrap_or(0);

                    match &key_event.logical_key {
                        Key::Character(s) => {
                            let mut new_text = current_text.clone();
                            new_text.insert_str_at_char_index(caret, s);
                            vec![
                                TextFieldAction::TextChange(new_text),
                                TextFieldAction::CaretMove(caret + s.chars().count()),
                            ]
                        }
                        Key::Named(NamedKey::Space) => {
                            let mut new_text = current_text.clone();
                            new_text.insert_at_char_index(caret, ' ');
                            vec![
                                TextFieldAction::TextChange(new_text),
                                TextFieldAction::CaretMove(caret + 1),
                            ]
                        }
                        Key::Named(NamedKey::Backspace) => {
                            if caret > 0 && caret <= current_text.chars().count() {
                                let mut new_text = current_text.clone();
                                new_text.remove_at_char_index(caret - 1);
                                vec![
                                    TextFieldAction::TextChange(new_text),
                                    TextFieldAction::CaretMove(caret - 1),
                                ]
                            } else {
                                vec![]
                            }
                        }
                        Key::Named(NamedKey::Delete) => {
                            if caret < current_text.chars().count() {
                                let mut new_text = current_text.clone();
                                new_text.remove_at_char_index(caret);
                                vec![TextFieldAction::TextChange(new_text)]
                            } else {
                                vec![]
                            }
                        }
                        Key::Named(NamedKey::ArrowLeft) => {
                            if caret > 0 {
                                vec![TextFieldAction::CaretMove(caret - 1)]
                            } else {
                                vec![]
                            }
                        }
                        Key::Named(NamedKey::ArrowRight) => {
                            if caret < current_text.chars().count() {
                                vec![TextFieldAction::CaretMove(caret + 1)]
                            } else {
                                vec![]
                            }
                        }
                        Key::Named(NamedKey::ArrowUp) => {
                            if let Some(layout) = &self.layout {
                                let intended_x = layout.get_caret_x(caret);
                                let new_caret = layout.get_vertical_move(caret, -1, intended_x);
                                vec![TextFieldAction::CaretMove(new_caret)]
                            } else {
                                vec![]
                            }
                        }
                        Key::Named(NamedKey::ArrowDown) => {
                            if let Some(layout) = &self.layout {
                                let intended_x = layout.get_caret_x(caret);
                                let new_caret = layout.get_vertical_move(caret, 1, intended_x);
                                vec![TextFieldAction::CaretMove(new_caret)]
                            } else {
                                vec![]
                            }
                        }
                        Key::Named(NamedKey::Enter) => {
                            let mut new_text = current_text.clone();
                            new_text.insert_at_char_index(caret, '\n');
                            vec![
                                TextFieldAction::TextChange(new_text),
                                TextFieldAction::CaretMove(caret + 1),
                            ]
                        }
                        _ => vec![],
                    }
                } else {
                    vec![]
                }
            }
            _ => vec![],
        });

        messages
    }
}
