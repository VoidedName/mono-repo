use crate::text::layout::TextLayout;
use crate::utils::ToArray;
use crate::{
    CaretSource, DynamicString, ElementId, ElementImpl, ElementSize, LabelText, SizeConstraints,
    TextInputParams, TextMetrics, UiContext,
};
use std::sync::Arc;
use vn_vttrpg_window::{logic, BoxPrimitive, Scene, TextPrimitive};
use web_time::Instant;
use vn_vttrpg_window::primitives::rect::RectBuilder;

pub struct TextAreaInput {
    id: ElementId,
    params: TextInputParams,
    text: String,
    caret_position: usize,
    text_metrics: Arc<dyn TextMetrics>,
    size: ElementSize,
    layout: TextLayout,
    gained_focus_at: Option<Instant>,
    show_caret: bool,
    caret_blink_duration: f32,
    line_height: f32,
    caret_width: f32,
}

impl TextAreaInput {
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

        let layout = TextLayout::layout(
            &text,
            &params.label.font,
            params.label.font_size,
            f32::INFINITY,
            text_metrics.as_ref(),
        );

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
                width: layout.total_width + caret_width,
                height: layout.total_height.max(line_height),
            },
            layout,
        }
    }

    pub fn update_state(&mut self, max_width: f32) {
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
            self.layout = TextLayout::layout(
                &self.text,
                &self.params.label.font,
                self.params.label.font_size,
                max_width - self.caret_width,
                self.text_metrics.as_ref(),
            );

            // Reset caret blink timer when changing
            if self.gained_focus_at.is_some() {
                self.gained_focus_at = Some(Instant::now());
            }

            let width = if max_width.is_finite() {
                max_width
            } else {
                self.layout.total_width + self.caret_width
            };

            self.size = ElementSize {
                width,
                height: self.layout.total_height.max(self.line_height),
            };
        }
    }
}

impl ElementImpl for TextAreaInput {
    fn id_impl(&self) -> ElementId {
        self.id
    }

    fn layout_impl(&mut self, _ctx: &mut UiContext, constraints: SizeConstraints) -> ElementSize {
        let max_width = constraints.max_size.width.unwrap_or(f32::INFINITY);
        self.update_state(max_width);

        // Ensure layout is up to date with max_width
        self.layout = TextLayout::layout(
            &self.text,
            &self.params.label.font,
            self.params.label.font_size,
            max_width - self.caret_width,
            self.text_metrics.as_ref(),
        );
        let width = if max_width.is_finite() {
            max_width
        } else {
            self.layout.total_width + self.caret_width
        };
        self.size = ElementSize {
            width,
            height: self.layout.total_height.max(self.line_height),
        };

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

        self.size.clamp_to_constraints(constraints)
    }

    fn draw_impl(
        &mut self,
        ctx: &mut UiContext,
        origin: (f32, f32),
        size: ElementSize,
        scene: &mut Scene,
    ) {
        let caret_height = self.params.label.font_size;
        let caret_y_extra_offset = self.line_height / 2.0 - caret_height / 2.0;

        ctx.with_hitbox_hierarchy(
            self.id,
            scene.current_layer_id(),
            vn_vttrpg_window::Rect {
                position: origin.to_array(),
                size: size.to_array(),
            },
            |_ctx| {
                for (i, line) in self.layout.lines.iter().enumerate() {
                    let mut text_builder = TextPrimitive::builder();
                    let line_y_offset = i as f32 * self.line_height;
                    text_builder = text_builder
                        .transform(|t| {
                            t.translation([origin.0 + self.caret_width / 2.0, origin.1 + line_y_offset])
                        })
                        .tint(self.params.label.color)
                        .clip_area(|c| {
                            c.size(size.to_array())
                                .position([-self.caret_width / 2.0, -line_y_offset])
                        });

                    let mut current_x = 0.0;
                    for glyph in &line.glyphs {
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
                }

                if self.show_caret {
                    scene.with_next_layer(|scene| {
                        let mut caret_x_offset = 0.0;
                        let mut caret_y_offset = 0.0;
                        
                        let mut found = false;
                        for (i, line) in self.layout.lines.iter().enumerate() {
                            if self.caret_position >= line.char_start && self.caret_position <= line.char_end {
                                caret_y_offset = i as f32 * self.line_height;
                                
                                // Calculate X offset within the line
                                
                                // Need to be careful with indices, they are char indices but we need byte indices for slicing if we use String
                                // Actually, let's just use the glyphs if they are 1-to-1 with chars (usually true for these simple fonts)
                                // Better: use text_metrics.size_of_text on a substring of the line
                                
                                let line_substring = if line.char_start == line.char_end {
                                    ""
                                } else {
                                    let start_byte = self.text.char_indices().nth(line.char_start).map(|(i, _)| i).unwrap_or(self.text.len());
                                    let end_byte = self.text.char_indices().nth(self.caret_position).map(|(i, _)| i).unwrap_or(self.text.len());
                                    &self.text[start_byte..end_byte]
                                };

                                caret_x_offset = self.text_metrics.size_of_text(
                                    line_substring,
                                    &self.params.label.font,
                                    self.params.label.font_size
                                ).0;
                                
                                found = true;
                                break;
                            }
                        }
                        
                        if !found && !self.layout.lines.is_empty() {
                            let last_line_idx = self.layout.lines.len() - 1;
                            caret_y_offset = last_line_idx as f32 * self.line_height;
                            caret_x_offset = self.layout.lines[last_line_idx].width;
                        }

                        let caret_x = origin.0 + caret_x_offset + self.caret_width / 2.0;
                        let caret_y = origin.1 + caret_y_offset + caret_y_extra_offset;

                        scene.add_box(
                            BoxPrimitive::builder()
                                .transform(|t| t.translation([caret_x, caret_y]))
                                .clip_area(|c| {
                                    c.size(size.to_array())
                                        .position([-caret_x_offset - self.caret_width / 2.0, -(caret_y_offset + caret_y_extra_offset)])
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
