use crate::{Element, ElementId, ElementImpl, ElementSize, SizeConstraints, UiContext};
use std::rc::Rc;
use vn_scene::Scene;
use vn_ui_animation::AnimationController;
use vn_ui_animation_macros::Interpolatable;
use vn_utils::option::UpdateOption;
use web_time::Instant;

#[derive(Clone, Copy, Debug, Interpolatable)]
pub struct PaddingParams {
    pub pad_left: f32,
    pub pad_right: f32,
    pub pad_top: f32,
    pub pad_bottom: f32,
}

impl PaddingParams {
    pub fn uniform(value: f32) -> Self {
        Self {
            pad_left: value,
            pad_right: value,
            pad_top: value,
            pad_bottom: value,
        }
    }
}

pub struct Padding {
    id: ElementId,
    child: Box<dyn Element>,
    controller: Rc<AnimationController<PaddingParams>>,
    layout_time: Instant,
    child_size: ElementSize,
}

impl Padding {
    pub fn new(
        child: Box<dyn Element>,
        controller: Rc<AnimationController<PaddingParams>>,
        ctx: &mut UiContext,
    ) -> Self {
        Self {
            id: ctx.event_manager.next_id(),
            child,
            controller,
            layout_time: Instant::now(),
            child_size: ElementSize::ZERO,
        }
    }
}

impl ElementImpl for Padding {
    fn id_impl(&self) -> ElementId {
        self.id
    }

    fn layout_impl(&mut self, ctx: &mut UiContext, constraints: SizeConstraints) -> ElementSize {
        self.layout_time = Instant::now();
        let params = self.controller.value(self.layout_time);

        let mut child_constraints = constraints;
        let x_padding = params.pad_left + params.pad_right;
        let y_padding = params.pad_top + params.pad_bottom;

        child_constraints
            .max_size
            .width
            .update(|w| w.max(x_padding) - x_padding);
        child_constraints
            .max_size
            .height
            .update(|h| h.max(y_padding) - y_padding);

        child_constraints.min_size.width =
            child_constraints.min_size.width.max(x_padding) - x_padding;
        child_constraints.min_size.height =
            child_constraints.min_size.height.max(y_padding) - y_padding;

        self.child_size = self.child.layout(ctx, child_constraints);

        ElementSize {
            width: self.child_size.width + x_padding,
            height: self.child_size.height + y_padding,
        }
        .clamp_to_constraints(constraints)
    }

    fn draw_impl(
        &mut self,
        ctx: &mut UiContext,
        origin: (f32, f32),
        size: ElementSize,
        canvas: &mut dyn Scene,
    ) {
        let params = self.controller.value(self.layout_time);

        let x_padding = params.pad_left + params.pad_right;
        let y_padding = params.pad_top + params.pad_bottom;

        self.child.draw(
            ctx,
            (origin.0 + params.pad_left, origin.1 + params.pad_top),
            ElementSize {
                width: size.width.max(x_padding) - x_padding,
                height: size.height.max(y_padding) - y_padding,
            },
            canvas,
        );
    }
}
