use crate::{Element, ElementId, ElementImpl, ElementSize, UiContext};
use vn_utils::UpdateOption;
use vn_vttrpg_window::Scene;

pub struct PaddingParams {
    pub pad_left: f32,
    pub pad_right: f32,
    pub pad_top: f32,
    pub pad_bottom: f32,
}

pub struct Padding {
    id: ElementId,
    child: Box<dyn Element>,
    params: PaddingParams,
    child_size: ElementSize,
}

impl Padding {
    pub fn new(child: Box<dyn Element>, params: PaddingParams, ctx: &mut UiContext) -> Self {
        Self {
            id: ctx.event_manager.next_id(),
            child,
            params,
            child_size: ElementSize::ZERO,
        }
    }
}

impl ElementImpl for Padding {
    fn id_impl(&self) -> crate::ElementId {
        self.id
    }

    fn layout_impl(
        &mut self,
        ctx: &mut UiContext,
        constraints: crate::SizeConstraints,
    ) -> ElementSize {
        let mut child_constraints = constraints;
        let x_padding = self.params.pad_left + self.params.pad_right;
        let y_padding = self.params.pad_top + self.params.pad_bottom;

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
        scene: &mut Scene,
    ) {
        let x_padding = self.params.pad_left + self.params.pad_right;
        let y_padding = self.params.pad_top + self.params.pad_bottom;

        self.child.draw(
            ctx,
            (
                origin.0 + self.params.pad_left,
                origin.1 + self.params.pad_top,
            ),
            ElementSize {
                width: size.width.max(x_padding) - x_padding,
                height: size.height.max(y_padding) - y_padding,
            },
            scene,
        );
    }
}
