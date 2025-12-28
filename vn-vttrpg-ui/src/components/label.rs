use crate::utils::ToArray;
use crate::{ElementId, ElementImpl, ElementSize, SizeConstraints, UiContext};
use std::sync::Arc;
use vn_vttrpg_window::{Color, Glyph, Scene, TextPrimitive};

/// This keeps the UI agnostic to any specific graphics and resource management
pub trait TextMetrics {
    fn size_of_text(&self, text: &str, font: &str, font_size: f32) -> (f32, f32);
    fn line_height(&self, font: &str, font_size: f32) -> f32;
    fn get_glyphs(&self, text: &str, font: &str, font_size: f32) -> Vec<Glyph>;
}

pub struct LabelParams {
    pub text: LabelText,
    pub font: String,
    pub font_size: f32,
    pub color: Color,
}

/// A UI element that renders a string of text.
pub struct Label {
    id: ElementId,
    params: LabelParams,
    text: String,
    text_metrics: Arc<dyn TextMetrics>,
    size: ElementSize,
}

pub struct DynamicString(pub Box<dyn Fn() -> String>);

pub enum LabelText {
    Static(String),
    Dynamic(DynamicString),
}

impl Label {
    pub fn new<T: TextMetrics + 'static>(
        params: LabelParams,
        text_metrics: Arc<T>,
        ctx: &mut UiContext,
    ) -> Self {
        let text = match &params.text {
            LabelText::Static(text) => text.clone(),
            LabelText::Dynamic(DynamicString(text)) => text(),
        };

        let (width, height) = text_metrics.size_of_text(&text, &params.font, params.font_size);

        Self {
            id: ctx.event_manager.next_id(),
            text,
            params,
            text_metrics,
            size: ElementSize { width, height },
        }
    }

    pub fn update_text(&mut self) {
        match &self.params.text {
            LabelText::Static(_) => {}
            LabelText::Dynamic(DynamicString(text)) => {
                let new_text = text();
                if new_text != self.text {
                    self.text = new_text;
                    let (width, height) = self.text_metrics.size_of_text(
                        &self.text,
                        &self.params.font,
                        self.params.font_size,
                    );
                    self.size = ElementSize { width, height };
                }
            }
        }
    }
}

impl ElementImpl for Label {
    fn id_impl(&self) -> ElementId {
        self.id
    }

    fn layout_impl(&mut self, _ctx: &mut UiContext, constraints: SizeConstraints) -> ElementSize {
        self.update_text();
        self.size.clamp_to_constraints(constraints)
    }

    fn draw_impl(
        &mut self,
        _ctx: &mut UiContext,
        origin: (f32, f32),
        size: ElementSize,
        scene: &mut Scene,
    ) {
        let glyphs =
            self.text_metrics
                .get_glyphs(&self.text, &self.params.font, self.params.font_size);

        let mut builder = TextPrimitive::builder();
        builder = builder
            .transform(|t| t.translation(origin.to_array()))
            .tint(self.params.color)
            .clip_area(|c| c.size(size.to_array()));

        let mut current_x = 0.0;
        for glyph in glyphs {
            builder = builder.add_glyph(vn_vttrpg_window::GlyphInstance {
                texture: glyph.texture.clone(),
                position: [current_x, glyph.y_offset],
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
