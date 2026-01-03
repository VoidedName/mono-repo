use crate::{
    Element, ElementId, ElementImpl, ElementSize, Padding, PaddingParams, SizeConstraints,
    UiContext,
};
use std::rc::Rc;
use vn_ui_animation::{AnimationController, Interpolatable};
use vn_ui_animation_macros::Interpolatable;
use vn_window::{BoxPrimitive, Color, Scene};

#[derive(Clone, Copy, Interpolatable)]
pub struct CardParams {
    pub background_color: Color,
    pub border_size: f32,
    pub border_color: Color,
    pub corner_radius: f32,
}

pub struct Card {
    id: ElementId,
    child: Padding,
    padding_controller: Rc<AnimationController<PaddingParams>>,
    controller: Rc<AnimationController<CardParams>>,
    layout_time: web_time::Instant,
}

impl Card {
    pub fn new(
        child: Box<dyn Element>,
        controller: Rc<AnimationController<CardParams>>,
        ctx: &mut UiContext,
    ) -> Self {
        let border_size = controller.value(web_time::Instant::now()).border_size;

        let padding_controller = PaddingParams::uniform(border_size)
            .into_controller()
            .into_rc();

        Self {
            id: ctx.event_manager.next_id(),
            child: Padding::new(child, padding_controller.clone(), ctx),
            padding_controller,
            controller,
            layout_time: web_time::Instant::now(),
        }
    }
}

impl ElementImpl for Card {
    fn id_impl(&self) -> ElementId {
        self.id
    }

    fn layout_impl(&mut self, ctx: &mut UiContext, constraints: SizeConstraints) -> ElementSize {
        self.layout_time = web_time::Instant::now();
        let params = self.controller.value(self.layout_time);

        self.padding_controller.update_state(|state| {
            state.target_value = PaddingParams::uniform(params.border_size);
        });

        self.child
            .layout(ctx, constraints)
            .clamp_to_constraints(constraints)
    }

    fn draw_impl(
        &mut self,
        ctx: &mut UiContext,
        origin: (f32, f32),
        size: ElementSize,
        scene: &mut Scene,
    ) {
        let params = self.controller.value(self.layout_time);

        scene.add_box(
            BoxPrimitive::builder()
                .transform(|t| t.translation([origin.0, origin.1]))
                .color(params.background_color)
                .border_color(params.border_color)
                .corner_radius(params.corner_radius)
                .border_thickness(params.border_size)
                .size([size.width, size.height])
                .build(),
        );

        self.child.draw(ctx, origin, size, scene);
    }
}
