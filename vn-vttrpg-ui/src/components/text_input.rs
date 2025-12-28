use crate::utils::ToArray;
use crate::{
    DynamicString, ElementId, ElementImpl, ElementSize, LabelParams, LabelText, SizeConstraints,
    TextMetrics, UiContext,
};
use std::sync::Arc;
use vn_utils::string::CharIndex;
use vn_vttrpg_window::{BoxPrimitive, Scene, TextPrimitive};
use web_time::Instant;

pub struct TextInputParams {
    pub label: LabelParams,
    pub text: LabelText,
    pub caret_position: CaretSource,
}

pub enum CaretSource {
    Static(usize),
    Dynamic(Box<dyn Fn() -> usize>),
}

pub struct TextInput {
    id: ElementId,
    params: TextInputParams,
    text: String,
    caret_position: usize,
    text_metrics: Arc<dyn TextMetrics>,
    size: ElementSize,
    gained_focus_at: Option<Instant>,
    show_caret: bool,
    caret_blink_duration: f32,
    line_height: f32,
    caret_width: f32,
}

impl TextInput {
    pub fn new<T: TextMetrics + 'static>(
        params: TextInputParams,
        text_metrics: Arc<T>,
        ctx: &mut UiContext,
    ) -> Self {
        let text = match &params.text {
            LabelText::Static(text) => text.clone(),
            LabelText::Dynamic(DynamicString(text)) => text(),
        };
        let caret_position = match &params.caret_position {
            CaretSource::Static(pos) => *pos,
            CaretSource::Dynamic(f) => f(),
        };

        let (width, height) =
            text_metrics.size_of_text(&text, &params.label.font, params.label.font_size);

        let caret_width = 2.0;
        let line_height = text_metrics.line_height(&params.label.font, params.label.font_size);

        Self {
            id: ctx.event_manager.next_id(),
            line_height,
            text,
            caret_position,
            params,
            show_caret: false,
            caret_width,
            text_metrics,
            caret_blink_duration: 2.0,
            gained_focus_at: None,
            size: ElementSize {
                width: width + caret_width,
                height: height.max(line_height),
            },
        }
    }

    pub fn update_state(&mut self) {
        let mut changed = false;
        match &self.params.text {
            LabelText::Static(_) => {}
            LabelText::Dynamic(DynamicString(f)) => {
                let new_text = f();
                if new_text != self.text {
                    self.text = new_text;
                    changed = true;
                }
            }
        }
        match &self.params.caret_position {
            CaretSource::Static(_) => {}
            CaretSource::Dynamic(f) => {
                let new_caret_position = f();

                if self.caret_position != new_caret_position {
                    changed = true;
                }

                self.caret_position = new_caret_position;
            }
        }

        if changed {
            let (width, height) = self.text_metrics.size_of_text(
                &self.text,
                &self.params.label.font,
                self.params.label.font_size,
            );

            // Reset caret blink timer when changing
            if self.gained_focus_at.is_some() {
                self.gained_focus_at = Some(Instant::now());
            }

            self.size = ElementSize {
                width: width + self.caret_width,
                height: height.max(self.line_height),
            };
        }
    }
}

impl ElementImpl for TextInput {
    fn id_impl(&self) -> ElementId {
        self.id
    }

    fn layout_impl(&mut self, _ctx: &mut UiContext, constraints: SizeConstraints) -> ElementSize {
        self.update_state();

        let is_focused = _ctx.event_manager.is_focused(self.id);
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
                self.show_caret = start_at.elapsed().as_secs_f32() % self.caret_blink_duration
                    < self.caret_blink_duration / 2.0;
            }
        }

        let size = self.size.clamp_to_constraints(constraints);
        size
    }

    fn draw_impl(
        &mut self,
        ctx: &mut UiContext,
        origin: (f32, f32),
        size: ElementSize,
        scene: &mut Scene,
    ) {
        let caret_height = self.params.label.font_size;
        let caret_y_offset = self.line_height / 2.0 - caret_height / 2.0;

        ctx.with_hitbox_hierarchy(
            self.id,
            scene.current_layer_id(),
            vn_vttrpg_window::Rect {
                position: origin.to_array(),
                size: size.to_array(),
            },
            |_ctx| {
                let glyphs = self.text_metrics.get_glyphs(
                    &self.text,
                    &self.params.label.font,
                    self.params.label.font_size,
                );

                let mut text_builder = TextPrimitive::builder();
                text_builder = text_builder
                    .transform(|t| t.translation([origin.0 + self.caret_width / 2.0, origin.1]))
                    .tint(self.params.label.color)
                    .clip_area(|c| {
                        c.size(size.to_array())
                            .position([-self.caret_width / 2.0, 0.0])
                    });

                let mut current_x = 0.0;
                for glyph in glyphs {
                    text_builder = text_builder.add_glyph(vn_vttrpg_window::GlyphInstance {
                        texture: glyph.texture.clone(),
                        position: [current_x + glyph.x_bearing, glyph.y_offset],
                        size: [
                            glyph.texture.texture.width() as f32,
                            glyph.texture.texture.height() as f32,
                        ],
                    });
                    current_x += glyph.advance;
                }
                scene.add_text(text_builder.build());

                if self.show_caret {
                    scene.with_next_layer(|scene| {
                        // Calculate caret X position

                        // todo: compute all of this in the layout phase
                        let caret_x_offset = if self.caret_position == 0 {
                            0.0
                        } else {
                            let text_up_to_caret = if self.caret_position >= self.text.len() {
                                &self.text
                            } else {
                                // ensure we don't split at non-char boundary
                                let end = self
                                    .text
                                    .byte_pos_for_char_index(self.caret_position)
                                    .unwrap_or(self.text.len());
                                &self.text[..end]
                            };
                            self.text_metrics
                                .size_of_text(
                                    text_up_to_caret,
                                    &self.params.label.font,
                                    self.params.label.font_size,
                                )
                                .0
                        };

                        let caret_x = origin.0 + caret_x_offset;
                        let caret_y = origin.1 + caret_y_offset;

                        scene.add_box(
                            BoxPrimitive::builder()
                                .transform(|t| t.translation([caret_x, caret_y]))
                                .clip_area(|c| {
                                    c.size(size.to_array())
                                        .position([-caret_x_offset, -caret_y_offset])
                                })
                                .size([self.caret_width, caret_height])
                                .color(self.params.label.color)
                                .build(),
                        );
                    });
                }
            },
        );
    }
}
