use crate::{Element, ElementId, ElementImpl, ElementSize, SizeConstraints, UiContext};

pub struct Fill {
    id: ElementId,
    element: Box<dyn Element>,
}

impl Fill {
    pub fn new(element: Box<dyn Element>, ctx: &mut UiContext) -> Self {
        Self {
            id: ctx.event_manager.next_id(),
            element,
        }
    }
}

impl ElementImpl for Fill {
    fn id_impl(&self) -> ElementId {
        self.id
    }

    fn layout_impl(&mut self, ctx: &mut UiContext, constraints: SizeConstraints) -> ElementSize {
        let child_size = self.element.layout(ctx, constraints);

        let width = match constraints.max_size.width {
            Some(w) => w,
            _ => child_size.width,
        };
        
        let height = match constraints.max_size.height {
            Some(h) => h,
            _ => child_size.height,
        };
        
        ElementSize { width, height }
    }
    
    fn draw_impl(&mut self, ctx: &mut UiContext, origin: (f32, f32), size: ElementSize, scene: &mut vn_vttrpg_window::Scene) {
        self.element.draw(ctx, origin, size, scene);
    }
}