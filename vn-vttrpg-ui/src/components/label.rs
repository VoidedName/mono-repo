use crate::utils::ToArray;
use crate::{ConcreteSize, Element, ElementId, SizeConstraints, UiContext};
use std::sync::Arc;
use vn_vttrpg_window::{Color, Scene, TextPrimitive};

/// This keeps the UI agnostic to any specific graphics and resource management
pub trait TextMetrics {
    fn size_of_text(&self, text: &str, font: &str, font_size: f32) -> (f32, f32);
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
    size: ConcreteSize,
}

pub enum LabelText {
    Static(String),
    Dynamic(Arc<Box<dyn Fn() -> String>>),
}

impl Label {
    pub fn new<T: TextMetrics + 'static>(params: LabelParams, text_metrics: Arc<T>, ctx: &mut UiContext) -> Self {
        let text = match &params.text {
            LabelText::Static(text) => text.clone(),
            LabelText::Dynamic(text) => text(),
        };

        let (width, height) = text_metrics.size_of_text(&text, &params.font, params.font_size);

        Self {
            id: ctx.event_manager.next_id(),
            text,
            params,
            text_metrics,
            size: ConcreteSize { width, height },
        }
    }

    pub fn update_text(&mut self) {
        match &self.params.text {
            LabelText::Static(_) => {}
            LabelText::Dynamic(text) => {
                let new_text = text();
                if new_text != self.text {
                    self.text = new_text;
                    let (width, height) = self.text_metrics.size_of_text(
                        &self.text,
                        &self.params.font,
                        self.params.font_size,
                    );
                    self.size = ConcreteSize { width, height };
                }
            }
        }
    }
}

impl Element for Label {
    fn id(&self) -> ElementId {
        self.id
    }
    
    fn layout_impl(&mut self, _ctx: &mut UiContext, constraints: SizeConstraints) -> ConcreteSize {
        self.update_text();
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
            TextPrimitive::builder(self.text.clone(), self.params.font.clone())
                .transform(|t| t.translation(origin.to_array()))
                // dunno if i should be squishing / stretching or clipping here...
                .size(self.size.to_array())
                .clip_area(|c| c.size(size.to_array()))
                .font_size(self.params.font_size)
                .tint(self.params.color)
                .build(),
        )
    }
}
