use crate::text::layout::TextLayout;
use crate::utils::ToArray;
use crate::{
    ElementId, ElementImpl, ElementSize, InteractionState, SizeConstraints, StateToParams,
    TextFieldCallbacks, TextMetrics, UiContext,
};
use std::cell::RefCell;
use std::rc::Rc;
use vn_scene::{BoxPrimitiveData, Color, Rect, Scene, TextPrimitiveData, Transform};
use vn_ui_animation_macros::Interpolatable;
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

#[derive(Clone, Interpolatable)]
pub struct TextFieldParams {
    pub visuals: TextVisuals,
    #[interpolate_snappy = "snap_middle"]
    pub controller: Rc<RefCell<dyn TextFieldCallbacks>>,
    #[interpolate_snappy = "snap_middle"]
    pub metrics: Rc<dyn TextMetrics>,
    pub interaction: InteractionState,
}

pub struct DynamicString(pub Box<dyn Fn() -> String>);

pub enum TextFieldText {
    Static(String),
    Dynamic(DynamicString),
}

pub struct StaticTextFieldController {
    text_layout: Option<TextLayout>,
    pub text: String,
}

impl StaticTextFieldController {
    pub fn new(text: String) -> Self {
        Self {
            text,
            text_layout: None,
        }
    }

    pub fn current_layout(&self) -> Option<&TextLayout> {
        self.text_layout.as_ref()
    }
}

impl TextFieldCallbacks for StaticTextFieldController {
    fn text_layout_changed(&mut self, layout: &TextLayout) {
        self.text_layout = Some(layout.clone());
    }
}

pub struct DynamicTextFieldController {
    text_layout: Option<TextLayout>,
    f: Box<dyn Fn() -> String>,
}

impl DynamicTextFieldController {
    pub fn new(f: Box<dyn Fn() -> String>) -> Self {
        Self {
            f,
            text_layout: None,
        }
    }

    pub fn text(&self) -> String {
        (self.f)()
    }

    pub fn current_layout(&self) -> Option<&TextLayout> {
        self.text_layout.as_ref()
    }
}

impl TextFieldCallbacks for DynamicTextFieldController {
    fn text_layout_changed(&mut self, layout: &TextLayout) {
        self.text_layout = Some(layout.clone());
    }
}

pub struct InputTextFieldController {
    pub id: ElementId,
    pub text: String,
    pub caret: usize,
    pub intended_x: f32,
    pub last_move_was_vertical: bool,
    text_layout: Option<TextLayout>,
}

impl InputTextFieldController {
    pub fn new(id: ElementId) -> Self {
        Self {
            id,
            text: "".to_string(),
            caret: 0,
            intended_x: 0.0,
            last_move_was_vertical: false,
            text_layout: None,
        }
    }

    pub fn current_layout(&self) -> Option<&TextLayout> {
        self.text_layout.as_ref()
    }
}

impl TextFieldCallbacks for InputTextFieldController {
    fn text_layout_changed(&mut self, layout: &TextLayout) {
        self.text_layout = Some(layout.clone());
    }
}

pub trait InputTextFieldControllerExt {
    fn handle_key(&mut self, key_event: &winit::event::KeyEvent);
    fn handle_click(&mut self, x: f32, y: f32);
}

impl InputTextFieldControllerExt for InputTextFieldController {
    fn handle_key(&mut self, key_event: &winit::event::KeyEvent) {
        if key_event.state.is_pressed() {
            use vn_utils::string::{InsertAtCharIndex, RemoveAtCharIndex};
            use winit::keyboard::{Key, NamedKey};

            if !self.last_move_was_vertical {
                if let Some(layout) = &self.text_layout {
                    self.intended_x = layout.get_caret_x(self.caret);
                }
            }

            match &key_event.logical_key {
                Key::Character(s) => {
                    self.text.insert_str_at_char_index(self.caret, s);
                    self.caret += s.chars().count();
                    if let Some(layout) = &self.text_layout {
                        self.intended_x = layout.get_caret_x(self.caret);
                    }
                    self.last_move_was_vertical = false;
                }
                Key::Named(NamedKey::Space) => {
                    self.text.insert_at_char_index(self.caret, ' ');
                    self.caret += 1;
                    if let Some(layout) = &self.text_layout {
                        self.intended_x = layout.get_caret_x(self.caret);
                    }
                    self.last_move_was_vertical = false;
                }
                Key::Named(NamedKey::Backspace) => {
                    if self.caret > 0 && self.caret <= self.text.len() {
                        self.caret -= 1;
                        self.text.remove_at_char_index(self.caret);
                        if let Some(layout) = &self.text_layout {
                            self.intended_x = layout.get_caret_x(self.caret);
                        }
                    }
                    self.last_move_was_vertical = false;
                }
                Key::Named(NamedKey::Delete) => {
                    if self.caret < self.text.len() {
                        self.text.remove_at_char_index(self.caret);
                        if let Some(layout) = &self.text_layout {
                            self.intended_x = layout.get_caret_x(self.caret);
                        }
                    }
                    self.last_move_was_vertical = false;
                }
                Key::Named(NamedKey::ArrowLeft) => {
                    if self.caret > 0 {
                        self.caret -= 1;
                        if let Some(layout) = &self.text_layout {
                            self.intended_x = layout.get_caret_x(self.caret);
                        }
                    }
                    self.last_move_was_vertical = false;
                }
                Key::Named(NamedKey::ArrowRight) => {
                    if self.caret < self.text.len() {
                        self.caret += 1;
                        if let Some(layout) = &self.text_layout {
                            self.intended_x = layout.get_caret_x(self.caret);
                        }
                    }
                    self.last_move_was_vertical = false;
                }
                Key::Named(NamedKey::ArrowUp) => {
                    if let Some(layout) = &self.text_layout {
                        self.caret = layout.get_vertical_move(self.caret, -1, self.intended_x);
                    }
                    self.last_move_was_vertical = true;
                }
                Key::Named(NamedKey::ArrowDown) => {
                    if let Some(layout) = &self.text_layout {
                        self.caret = layout.get_vertical_move(self.caret, 1, self.intended_x);
                    }
                    self.last_move_was_vertical = true;
                }
                Key::Named(NamedKey::Enter) => {
                    self.text.insert_at_char_index(self.caret, '\n');
                    self.caret += 1;
                    if let Some(layout) = &self.text_layout {
                        self.intended_x = layout.get_caret_x(self.caret);
                    }
                    self.last_move_was_vertical = false;
                }
                _ => {}
            }
        }
    }

    fn handle_click(&mut self, x: f32, y: f32) {
        let c_pos = self
            .current_layout()
            .and_then(|layout: &TextLayout| layout.hit_test(x, y));

        if let Some(c_pos) = c_pos {
            self.caret = c_pos;
            if let Some(layout) = self.current_layout() {
                self.intended_x = layout.get_caret_x(self.caret);
            }
            self.last_move_was_vertical = false;
        }
    }
}

pub struct TextField<State> {
    id: ElementId,
    params: StateToParams<State, TextFieldParams>,
    visuals: Option<TextVisuals>,
    layout: Option<TextLayout>,
    size: ElementSize,
    gained_focus_at: Option<Instant>,
    show_caret: bool,
    line_height: f32,
    last_max_width: Option<f32>,
}

impl<State> TextField<State> {
    pub fn new(params: StateToParams<State, TextFieldParams>, ctx: &mut UiContext) -> Self {
        Self {
            id: ctx.event_manager.borrow_mut().next_id(),
            line_height: 0.0,
            visuals: None,
            layout: None,
            params,
            show_caret: false,
            gained_focus_at: None,
            size: ElementSize::ZERO,
            last_max_width: None,
        }
    }

    pub fn update_state(&mut self, state: &State, now: &Instant, max_width: Option<f32>) -> bool {
        let params = (self.params)(state, now, self.id);
        let mut changed = self.visuals.as_ref() != Some(&params.visuals);

        if max_width != self.last_max_width {
            self.last_max_width = max_width;
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
                max_width.map(|w| w - caret_space),
                params.metrics.as_ref(),
            );
            params.controller.borrow_mut().text_layout_changed(&layout);

            self.size = ElementSize {
                width: layout.total_width + caret_space,
                height: layout.total_height.max(self.line_height),
            };
            self.layout = Some(layout);
            self.visuals = Some(params.visuals.clone());

            // Reset caret blink timer when changing
            if self.gained_focus_at.is_some() {
                self.gained_focus_at = Some(Instant::now());
            }
        }
        changed
    }

    pub fn caret_width(&self) -> f32 {
        if let Some(visuals) = &self.visuals {
            if visuals.caret_position.is_some() {
                return visuals.caret_width.unwrap_or(2.0);
            }
        }
        0.0
    }
}

impl<State> ElementImpl for TextField<State> {
    type State = State;

    fn id_impl(&self) -> ElementId {
        self.id
    }

    fn layout_impl(
        &mut self,
        ctx: &mut UiContext,
        state: &Self::State,
        constraints: SizeConstraints,
    ) -> ElementSize {
        self.update_state(state, &ctx.now, constraints.max_size.width);

        let params = (self.params)(state, &ctx.now, self.id);
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
                self.show_caret = start_at.elapsed().as_secs_f32() % caret_blink_duration
                    < caret_blink_duration / 2.0;
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
        canvas: &mut dyn Scene,
    ) {
        let params = (self.params)(state, &ctx.now, self.id);
        let visuals = &params.visuals;

        let caret_height = self.line_height * 0.8;
        let caret_y_extra_offset = self.line_height / 2.0 - caret_height / 2.0;
        let caret_width = self.caret_width();
        let caret_space = if visuals.caret_position.is_some() {
            caret_width
        } else {
            0.0
        };

        ctx.with_hitbox_hierarchy(
            self.id,
            canvas.current_layer_id(),
            Rect {
                position: origin.to_array(),
                size: size.to_array(),
            },
            |_ctx| {
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

                        canvas.add_text(TextPrimitiveData {
                            transform: Transform {
                                translation: [
                                    origin.0 + caret_space / 2.0,
                                    origin.1 + line_y_offset,
                                ],
                                ..Transform::DEFAULT
                            },
                            tint: visuals.color,
                            glyphs,
                            clip_rect: Rect {
                                position: origin.to_array(),
                                size: size.to_array(),
                            },
                        });
                    }

                    if self.show_caret {
                        if let Some(caret_position) = visuals.caret_position {
                            canvas.with_next_layer(&mut |canvas| {
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
                                    clip_rect: Rect {
                                        position: origin.to_array(),
                                        size: size.to_array(),
                                    },
                                });
                            });
                        }
                    }
                }
            },
        );
    }
}
