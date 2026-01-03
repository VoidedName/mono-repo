use crate::utils::ToArray;
use crate::{Element, ElementId, ElementImpl, ElementSize, SizeConstraints, UiContext};
use vn_scene::{Rect, Scene};

pub struct ExtendedHitbox {
    id: ElementId,
    element: Box<dyn Element>,
}

impl ExtendedHitbox {
    pub fn new(element: Box<dyn Element>, ctx: &mut UiContext) -> Self {
        let ui_id = ctx.event_manager.next_id();
        Self { id: ui_id, element }
    }
}

impl ElementImpl for ExtendedHitbox {
    fn id_impl(&self) -> ElementId {
        self.id
    }

    fn layout_impl(&mut self, ctx: &mut UiContext, constraints: SizeConstraints) -> ElementSize {
        self.element
            .layout(ctx, constraints)
            .clamp_to_constraints(constraints)
    }

    fn draw_impl(
        &mut self,
        ctx: &mut UiContext,
        origin: (f32, f32),
        size: ElementSize,
        canvas: &mut dyn Scene,
    ) {
        ctx.with_hitbox_hierarchy(
            self.id,
            canvas.current_layer_id(),
            Rect {
                position: origin.to_array(),
                size: size.to_array(),
            },
            |ctx| {
                self.element.draw(ctx, origin, size, canvas);
            },
        );
    }
}
