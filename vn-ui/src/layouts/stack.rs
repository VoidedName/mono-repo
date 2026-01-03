use crate::{
    DynamicSize, Element, ElementId, ElementImpl, ElementSize, SizeConstraints, UiContext,
};
use vn_window::Scene;

pub struct Stack {
    id: ElementId,
    children: Vec<Box<dyn Element>>,
    children_size: Vec<ElementSize>,
}

impl Stack {
    pub fn new(children: Vec<Box<dyn Element>>, ctx: &mut UiContext) -> Self {
        Stack {
            id: ctx.event_manager.next_id(),
            children_size: vec![ElementSize::ZERO; children.len()],
            children,
        }
    }
}

impl ElementImpl for Stack {
    fn id_impl(&self) -> ElementId {
        self.id
    }

    fn layout_impl(&mut self, ctx: &mut UiContext, constraints: SizeConstraints) -> ElementSize {
        let mut max_width: f32 = 0.0;
        let mut max_height: f32 = 0.0;

        for (idx, child) in &mut self.children.iter_mut().enumerate() {
            let child_size = child.layout(ctx, constraints);

            max_width = max_width.max(child_size.width);
            max_height = max_height.max(child_size.height);

            self.children_size[idx] = child_size;
        }

        ElementSize {
            width: constraints.max_size.width.unwrap_or(max_width),
            height: constraints.max_size.height.unwrap_or(max_height),
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
        let mut first_drawn = false;

        let mut draw_child =
            |child: &mut Box<dyn Element>, child_size: ElementSize, scene: &mut Scene| {
                child.draw(
                    ctx,
                    (origin.0, origin.1),
                    child_size.clamp_to_constraints(SizeConstraints {
                        min_size: ElementSize::ZERO,
                        max_size: DynamicSize {
                            width: Some(size.width),
                            height: Some(size.height),
                        },
                        scene_size: scene.scene_size(),
                    }),
                    scene,
                );
            };

        for (idx, child) in self.children.iter_mut().enumerate() {
            match first_drawn {
                true => {
                    scene.with_top_layer(|scene| draw_child(child, self.children_size[idx], scene))
                }
                false => {
                    draw_child(child, self.children_size[idx], scene);
                    first_drawn = true;
                }
            }
        }
    }
}
