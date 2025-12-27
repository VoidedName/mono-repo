use crate::{Element, ElementId, ElementSize, SizeConstraints, UiContext};
use vn_utils::UpdateOption;
use vn_vttrpg_window::{BoxPrimitive, Color, Scene};

#[derive(Clone, Copy)]
pub struct CardParams {
    pub background_color: Color,
    pub border_size: f32,
    pub border_color: Color,
    pub corner_radius: f32,
}

pub struct Card {
    id: ElementId,
    child: Box<dyn Element>,
    child_size: ElementSize,
    params: CardParams,
}

impl Card {
    pub fn new(child: Box<dyn Element>, params: CardParams, ctx: &mut UiContext) -> Self {
        Self {
            id: ctx.event_manager.next_id(),
            child,
            child_size: ElementSize::ZERO,
            params,
        }
    }
}

impl Element for Card {
    fn id(&self) -> ElementId {
        self.id
    }

    fn layout_impl(&mut self, ctx: &mut UiContext, constraints: SizeConstraints) -> ElementSize {
        let mut child_constraints = constraints;
        let margin = self.params.border_size * 2.0;

        child_constraints
            .max_size
            .width
            .update(|w| w.max(margin) - margin);
        child_constraints
            .max_size
            .height
            .update(|h| h.max(margin) - margin);

        child_constraints.min_size.width = child_constraints.min_size.width.max(margin) - margin;
        child_constraints.min_size.height = child_constraints.min_size.height.max(margin) - margin;

        self.child_size = self.child.layout(ctx, child_constraints);

        ElementSize {
            width: self.child_size.width + margin,
            height: self.child_size.height + margin,
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
        let margin = self.params.border_size * 2.0;

        scene.add_box(
            BoxPrimitive::builder()
                .transform(|t| t.translation([origin.0, origin.1]))
                .color(self.params.background_color)
                .border_color(self.params.border_color)
                .corner_radius(self.params.corner_radius)
                .border_thickness(self.params.border_size)
                .size([size.width, size.height])
                .build(),
        );

        self.child.draw(
            ctx,
            (
                origin.0 + self.params.border_size,
                origin.1 + self.params.border_size,
            ),
            ElementSize {
                width: size.width.max(margin) - margin,
                height: size.height.max(margin) - margin,
            },
            scene,
        );
    }
}
