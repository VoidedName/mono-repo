use crate::utils::ToArray;
use crate::{
    Element, ElementId, ElementImpl, ElementSize, ElementWorld, InteractionState, SizeConstraints,
    StateToParams, ToolTip, TooltipParams, UiContext,
};
use vn_scene::{BoxPrimitiveData, Color, Rect, Scene, Transform};
use vn_ui_animation_macros::Interpolatable;

#[derive(Clone, Copy, Interpolatable)]
pub struct ButtonParams {
    pub background: Color,
    pub border_color: Color,
    pub border_width: f32,
    pub corner_radius: f32,
    pub interaction: InteractionState,
}

pub struct Button<State> {
    id: ElementId,
    child: Box<dyn Element<State = State>>,
    params: StateToParams<State, ButtonParams>,
}

impl<State> Button<State> {
    pub fn new(
        child: Box<dyn Element<State = State>>,
        params: StateToParams<State, ButtonParams>,
        world: &mut ElementWorld,
    ) -> Self {
        Self {
            id: world.next_id(),
            child,
            params,
        }
    }
}

impl<State> ElementImpl for Button<State> {
    type State = State;

    fn id_impl(&self) -> ElementId {
        self.id
    }

    fn layout_impl(
        &mut self,
        ctx: &mut UiContext,
        state: &Self::State,
        constraints: SizeConstraints,
    ) -> ElementSize {
        let _params = (self.params)(crate::StateToParamsArgs {
            state,
            id: self.id,
            ctx,
        });

        self.child
            .layout(ctx, state, constraints)
            .clamp_to_constraints(constraints)
    }

    fn draw_impl(
        &mut self,
        ctx: &mut UiContext,
        state: &Self::State,
        origin: (f32, f32),
        size: ElementSize,
        canvas: &mut dyn Scene,
    ) {
        let params = (self.params)(crate::StateToParamsArgs {
            state,
            id: self.id,
            ctx,
        });

        let mut background = params.background;
        let mut border_color = params.border_color;

        if params.interaction.is_hovered {
            background = background.lighten(0.1);
        }

        if params.interaction.is_focused {
            background = background.darken(0.1);
            border_color = Color::WHITE;
        }

        ctx.with_hitbox_hierarchy(
            self.id,
            canvas.current_layer_id(),
            Rect {
                position: origin.to_array(),
                size: size.to_array(),
            },
            |ctx| {
                canvas.add_box(BoxPrimitiveData {
                    transform: Transform {
                        translation: [origin.0, origin.1],
                        ..Transform::DEFAULT
                    },
                    size: [size.width, size.height],
                    color: background,
                    border_color,
                    border_thickness: params.border_width,
                    border_radius: params.corner_radius,
                    clip_rect: Rect::NO_CLIP,
                });

                let margin = params.border_width * 2.0;
                self.child.draw(
                    ctx,
                    state,
                    (
                        origin.0 + params.border_width,
                        origin.1 + params.border_width,
                    ),
                    ElementSize {
                        width: size.width.max(margin) - margin,
                        height: size.height.max(margin) - margin,
                    },
                    canvas,
                );
            },
        );
    }
}

pub trait ButtonExt: Element {
    fn button(
        self,
        params: StateToParams<Self::State, ButtonParams>,
        world: &mut ElementWorld,
    ) -> Button<Self::State>;
}

impl<E: Element + 'static> ButtonExt for E {
    fn button(
        self,
        params: StateToParams<Self::State, ButtonParams>,
        world: &mut ElementWorld,
    ) -> Button<Self::State> {
        Button::new(Box::new(self), params, world)
    }
}
