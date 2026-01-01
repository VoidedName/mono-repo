use crate::{
    Element, ElementId, ElementImpl, ElementSize, Padding, PaddingParams, SizeConstraints,
    UiContext,
};
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
    child: Padding,
    params: CardParams,
}

impl Card {
    pub fn new(child: Box<dyn Element>, params: CardParams, ctx: &mut UiContext) -> Self {
        Self {
            id: ctx.event_manager.next_id(),
            child: Padding::new(
                child,
                PaddingParams {
                    pad_left: params.border_size,
                    pad_right: params.border_size,
                    pad_top: params.border_size,
                    pad_bottom: params.border_size,
                },
                ctx,
            ),
            params,
        }
    }
}

impl ElementImpl for Card {
    fn id_impl(&self) -> ElementId {
        self.id
    }

    fn layout_impl(&mut self, ctx: &mut UiContext, constraints: SizeConstraints) -> ElementSize {
        self.child.layout(ctx, constraints).clamp_to_constraints(constraints)
    }

    fn draw_impl(
        &mut self,
        ctx: &mut UiContext,
        origin: (f32, f32),
        size: ElementSize,
        scene: &mut Scene,
    ) {
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

        self.child.draw(ctx, origin, size, scene);
    }
}
