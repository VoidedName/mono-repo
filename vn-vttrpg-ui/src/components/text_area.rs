use crate::text::layout::TextLayout;
use crate::utils::ToArray;
use crate::{
    DynamicString, ElementId, ElementImpl, ElementSize, LabelParams, LabelText, SizeConstraints,
    TextMetrics, UiContext,
};
use std::sync::Arc;
use vn_vttrpg_window::{Scene, TextPrimitive};

/// A UI element that renders multiple lines of text with autowrapping and newline support.
pub struct TextArea {
    id: ElementId,
    params: LabelParams,
    text: String,
    text_metrics: Arc<dyn TextMetrics>,
    size: ElementSize,
    layout: TextLayout,
}

impl TextArea {
    pub fn new<T: TextMetrics + 'static>(
        params: LabelParams,
        text_metrics: Arc<T>,
        ctx: &mut UiContext,
    ) -> Self {
        let text = match &params.text {
            LabelText::Static(text) => text.clone(),
            LabelText::Dynamic(DynamicString(text)) => text(),
        };

        // Initial layout with no width constraint (will wrap based on available width in layout_impl)
        let layout = TextLayout::layout(
            &text,
            &params.font,
            params.font_size,
            f32::INFINITY,
            text_metrics.as_ref(),
        );

        Self {
            id: ctx.event_manager.next_id(),
            text,
            params,
            text_metrics,
            size: ElementSize {
                width: layout.total_width,
                height: layout.total_height,
            },
            layout,
        }
    }

    pub fn update_text(&mut self, max_width: f32) -> bool {
        let mut text_changed = false;
        match &self.params.text {
            LabelText::Static(_) => {}
            LabelText::Dynamic(DynamicString(text)) => {
                let new_text = text();
                if new_text != self.text {
                    self.text = new_text;
                    text_changed = true;
                }
            }
        }

        if text_changed {
            self.layout = TextLayout::layout(
                &self.text,
                &self.params.font,
                self.params.font_size,
                max_width,
                self.text_metrics.as_ref(),
            );
            let width = if max_width.is_finite() {
                max_width
            } else {
                self.layout.total_width
            };
            self.size = ElementSize {
                width,
                height: self.layout.total_height,
            };
        }
        text_changed
    }
}

impl ElementImpl for TextArea {
    fn id_impl(&self) -> ElementId {
        self.id
    }

    fn layout_impl(&mut self, _ctx: &mut UiContext, constraints: SizeConstraints) -> ElementSize {
        let max_width = constraints.max_size.width.unwrap_or(f32::INFINITY);

        if !self.update_text(max_width) {
            // If text didn't change, we might still need to re-layout if max_width changed
            // We can check if current layout width is greater than new max_width
            // or if it was wrapped and now we have more space.
            // For simplicity, let's just re-layout if max_width is different from what we'd expect.
            // Actually, comparing floats for equality is bad, but here we just want to know if we need a refresh.
            
            self.layout = TextLayout::layout(
                &self.text,
                &self.params.font,
                self.params.font_size,
                max_width,
                self.text_metrics.as_ref(),
            );
            let width = if max_width.is_finite() {
                max_width
            } else {
                self.layout.total_width
            };
            self.size = ElementSize {
                width,
                height: self.layout.total_height,
            };
        }

        self.size.clamp_to_constraints(constraints)
    }

    fn draw_impl(
        &mut self,
        _ctx: &mut UiContext,
        origin: (f32, f32),
        size: ElementSize,
        scene: &mut Scene,
    ) {
        let line_height = self
            .text_metrics
            .line_height(&self.params.font, self.params.font_size);

        for (i, line) in self.layout.lines.iter().enumerate() {
            let mut builder = TextPrimitive::builder();
            let y_offset = i as f32 * line_height;
            builder = builder
                .transform(|t| t.translation([origin.0, origin.1 + y_offset]))
                .tint(self.params.color)
                .clip_area(|c| {
                    c.size(size.to_array())
                        .position([0.0, -y_offset])
                });

            let mut current_x = 0.0;
            for glyph in &line.glyphs {
                builder = builder.add_glyph(vn_vttrpg_window::GlyphInstance {
                    texture: glyph.texture.clone(),
                    position: [current_x + glyph.x_bearing, glyph.y_offset],
                    size: [
                        glyph.texture.texture.width() as f32,
                        glyph.texture.texture.height() as f32,
                    ],
                });
                current_x += glyph.advance;
            }

            scene.add_text(builder.build());
        }
    }
}
