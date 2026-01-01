use crate::text::layout::TextLayout;
use crate::utils::ToArray;
use crate::{
    ElementId, ElementImpl, ElementSize, SizeConstraints, TextFieldParams, TextMetrics, UiContext,
};
use std::cell::RefCell;
use std::rc::Rc;
use vn_vttrpg_window::{BoxPrimitive, Scene, TextPrimitive};
use web_time::Instant;

pub trait TextFieldController {
    fn text(&self) -> String;
    fn caret_position(&self) -> Option<usize>;
    // I'm not entirely sure if this is the right place for this, but it's the easiest place to put it for now.
    // I need to somehow / somewhere report the text layout so that the logic can respond to it correctly.
    fn set_current_layout(&mut self, layout: TextLayout);
    fn current_layout(&self) -> Option<&TextLayout>;
}

pub struct StaticTextFieldController {
    text_layout: Option<TextLayout>,
    text: String,
}

impl StaticTextFieldController {
    pub fn new(text: String) -> Self {
        Self {
            text,
            text_layout: None,
        }
    }
}

impl TextFieldController for StaticTextFieldController {
    fn text(&self) -> String {
        self.text.clone()
    }
    fn caret_position(&self) -> Option<usize> {
        None
    }

    fn set_current_layout(&mut self, layout: TextLayout) {
        self.text_layout = Some(layout);
    }

    fn current_layout(&self) -> Option<&TextLayout> {
        self.text_layout.as_ref()
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
}

impl TextFieldController for DynamicTextFieldController {
    fn text(&self) -> String {
        (self.f)()
    }
    fn caret_position(&self) -> Option<usize> {
        None
    }

    fn set_current_layout(&mut self, layout: TextLayout) {
        self.text_layout = Some(layout);
    }
    fn current_layout(&self) -> Option<&TextLayout> {
        self.text_layout.as_ref()
    }
}

pub struct InputTextFieldController {
    text_f: Box<dyn Fn() -> String>,
    caret_f: Box<dyn Fn() -> Option<usize>>,
    text_layout: Option<TextLayout>,
}

impl InputTextFieldController {
    pub fn new(text_f: Box<dyn Fn() -> String>, caret_f: Box<dyn Fn() -> Option<usize>>) -> Self {
        Self {
            text_f,
            caret_f,
            text_layout: None,
        }
    }
}

impl TextFieldController for InputTextFieldController {
    fn text(&self) -> String {
        (self.text_f)()
    }
    fn caret_position(&self) -> Option<usize> {
        (self.caret_f)()
    }

    fn set_current_layout(&mut self, layout: TextLayout) {
        self.text_layout = Some(layout);
    }
    fn current_layout(&self) -> Option<&TextLayout> {
        self.text_layout.as_ref()
    }
}

pub struct TextField {
    id: ElementId,
    params: TextFieldParams,
    controller: Rc<RefCell<dyn TextFieldController>>,
    text: String,
    caret_position: Option<usize>,
    text_metrics: Rc<dyn TextMetrics>,
    size: ElementSize,
    gained_focus_at: Option<Instant>,
    show_caret: bool,
    caret_blink_duration: f32,
    line_height: f32,
    caret_width: f32,
    last_max_width: Option<f32>,
}

impl TextField {
    pub fn new<T: TextMetrics + 'static>(
        params: TextFieldParams,
        controller: Rc<RefCell<dyn TextFieldController>>,
        text_metrics: Rc<T>,
        ctx: &mut UiContext,
    ) -> Self {
        let text = controller.borrow().text();
        let caret_position = controller.borrow().caret_position();

        let caret_width = 2.0;
        let line_height = text_metrics.line_height(&params.font, params.font_size);

        Self {
            id: ctx.event_manager.next_id(),
            line_height,
            text,
            caret_position,
            params,
            controller,
            show_caret: false,
            caret_width,
            text_metrics,
            caret_blink_duration: 1.0,
            gained_focus_at: None,
            size: ElementSize::ZERO,
            last_max_width: None,
        }
    }

    pub fn update_state(&mut self, max_width: Option<f32>) -> bool {
        let mut changed = false;
        let new_text = self.controller.borrow().text();
        if new_text != self.text {
            self.text = new_text;
            changed = true;
        }

        let new_caret_position = self.controller.borrow().caret_position();
        if self.caret_position != new_caret_position {
            self.caret_position = new_caret_position;
            changed = true;
        }

        if max_width != self.last_max_width {
            self.last_max_width = max_width;
            changed = true;
        }

        if changed {
            let caret_space = self.caret_width;
            self.controller
                .borrow_mut()
                .set_current_layout(TextLayout::layout(
                    &self.text,
                    &self.params.font,
                    self.params.font_size,
                    max_width.map(|w| w - caret_space),
                    self.text_metrics.as_ref(),
                ));

            // Reset caret blink timer when changing
            if self.gained_focus_at.is_some() {
                self.gained_focus_at = Some(Instant::now());
            }

            self.size = self
                .controller
                .borrow()
                .current_layout()
                .map(|l| ElementSize {
                    width: l.total_width + caret_space,
                    height: l.total_height.max(self.line_height),
                })
                .unwrap_or(ElementSize::ZERO);
        }
        changed
    }

    pub fn caret_width(&self) -> f32 {
        if self.caret_position.is_some() {
            self.caret_width
        } else {
            0.0
        }
    }
}

impl ElementImpl for TextField {
    fn id_impl(&self) -> ElementId {
        self.id
    }

    fn layout_impl(&mut self, ctx: &mut UiContext, constraints: SizeConstraints) -> ElementSize {
        self.update_state(constraints.max_size.width);

        let is_focused = ctx.event_manager.is_focused(self.id);
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
        let caret_height = self.line_height * 0.8;
        let caret_y_extra_offset = self.line_height / 2.0 - caret_height / 2.0;
        let caret_space = if self.caret_position.is_some() {
            self.caret_width
        } else {
            0.0
        };

        ctx.with_hitbox_hierarchy(
            self.id,
            scene.current_layer_id(),
            vn_vttrpg_window::Rect {
                position: origin.to_array(),
                size: size.to_array(),
            },
            |_ctx| {
                if let Some(layout) = self.controller.borrow().current_layout() {
                    for (i, line) in layout.lines.iter().enumerate() {
                        let mut text_builder = TextPrimitive::builder();
                        let line_y_offset = i as f32 * self.line_height;
                        text_builder = text_builder
                            .transform(|t| {
                                t.translation([
                                    origin.0 + caret_space / 2.0,
                                    origin.1 + line_y_offset,
                                ])
                            })
                            .tint(self.params.color)
                            .clip_area(|c| {
                                c.size(size.to_array())
                                    .position([-caret_space / 2.0, -line_y_offset])
                            });

                        let mut current_x = 0.0;
                        for glyph in &line.glyphs {
                            text_builder =
                                text_builder.add_glyph(vn_vttrpg_window::GlyphInstance {
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
                        if let Some(caret_position) = self.caret_position {
                            scene.with_next_layer(|scene| {
                                let (caret_x_offset, caret_y_offset) =
                                    layout.get_caret_pos(caret_position);

                                let caret_x = origin.0 + caret_x_offset + self.caret_width / 2.0;
                                let caret_y = origin.1 + caret_y_offset + caret_y_extra_offset;

                                scene.add_box(
                                    BoxPrimitive::builder()
                                        .transform(|t| t.translation([caret_x, caret_y]))
                                        .clip_area(|c| {
                                            c.size(size.to_array()).position([
                                                -caret_x_offset - self.caret_width / 2.0,
                                                -(caret_y_offset + caret_y_extra_offset),
                                            ])
                                        })
                                        .size([self.caret_width, caret_height])
                                        .color(self.params.color)
                                        .build(),
                                );
                            });
                        }
                    }
                }
            },
        );
    }
}
