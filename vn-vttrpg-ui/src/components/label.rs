use crate::{ConcreteSize, Element, SizeConstraints, UiContext};
use vn_vttrpg_window::{Color, Scene, TextPrimitive};

/// This keeps the UI agnostic to any specific graphics and resource management
pub trait TextMetrics {
    fn size_of_text(&self, text: &str, font: &str, font_size: f32) -> (f32, f32);
}

#[derive(Clone)]
pub struct LabelParams {
    pub text: String,
    pub font: String,
    pub font_size: f32,
    pub color: Color,
}

/// A UI element that renders a string of text.
pub struct Label {
    params: LabelParams,
    size: ConcreteSize,
}

impl Label {
    pub fn new<T: TextMetrics>(params: LabelParams, text_metrics: &T) -> Self {
        let size = text_metrics.size_of_text(&params.text, &params.font, params.font_size);

        Self {
            params,
            size: ConcreteSize {
                width: size.0,
                height: size.1,
            },
        }
    }
}

impl Element for Label {
    fn layout(&mut self, _ctx: &mut UiContext, constraints: SizeConstraints) -> ConcreteSize {
        self.size.clamp_to_constraints(constraints)
    }

    fn draw_impl(
        &mut self,
        _ctx: &mut UiContext,
        origin: (f32, f32),
        size: ConcreteSize,
        scene: &mut Scene,
    ) {
        scene.add_text(
            TextPrimitive::builder(self.params.text.clone(), self.params.font.clone())
                .transform(|t| t.translation([origin.0, origin.1]))
                // dunno if i should be squishing / stretching or clipping here...
                .size([self.size.width, self.size.height])
                .clip_area(|c| c.size([size.width, size.height]))
                .font_size(self.params.font_size)
                .tint(self.params.color)
                .build(),
        )
    }
}
