use crate::{ConcreteSize, Element, ElementId, SizeConstraints, UiContext};
use vn_vttrpg_window::{Rect, Scene};
use crate::utils::ToArray;

pub struct ExtendedHitbox {
    ui_id: ElementId,
    element: Box<dyn Element>,
}

impl ExtendedHitbox {
    pub fn new(element: Box<dyn Element>, ctx: &mut UiContext) -> Self {
        let ui_id = ctx.event_manager.next_id();
        Self { ui_id, element }
    }
}

impl Element for ExtendedHitbox {
    fn layout(&mut self, ctx: &mut UiContext, constraints: SizeConstraints) -> ConcreteSize {
        self.element.layout(ctx, constraints)
    }

    fn draw_impl(
        &mut self,
        ctx: &mut UiContext,
        origin: (f32, f32),
        size: ConcreteSize,
        scene: &mut Scene,
    ) {
        ctx.with_hitbox_hierarchy(
            self.ui_id,
            scene.current_layer_id(),
            Rect {
                position: origin.to_array(),
                size: size.to_array(),
            },
            |ctx| {
                self.element.draw(ctx, origin, size, scene);
            },
        );
    }
}
