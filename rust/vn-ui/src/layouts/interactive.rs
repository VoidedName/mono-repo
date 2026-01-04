use crate::{Element, ElementId, ElementImpl, ElementSize, SizeConstraints, UiContext};
use vn_scene::Scene;

pub struct Interactive {
    id: ElementId,
    child: Box<dyn Element>,
    interactive: bool,
}

pub struct InteractiveParams {
    pub is_interactive: bool,
}

impl Interactive {
    pub fn new(child: Box<dyn Element>, params: InteractiveParams, ctx: &mut UiContext) -> Self {
        Self {
            id: ctx.event_manager.next_id(),
            child,
            interactive: params.is_interactive,
        }
    }
}

impl ElementImpl for Interactive {
    fn id_impl(&self) -> ElementId {
        self.id
    }

    fn layout_impl(&mut self, ctx: &mut UiContext, constraints: SizeConstraints) -> ElementSize {
        self.child.layout(ctx, constraints)
    }

    fn draw_impl(
        &mut self,
        ctx: &mut UiContext,
        origin: (f32, f32),
        size: ElementSize,
        canvas: &mut dyn Scene,
    ) {
        ctx.with_interactivity(self.interactive, |ctx| {
            self.child.draw(ctx, origin, size, canvas);
        });
    }
}
