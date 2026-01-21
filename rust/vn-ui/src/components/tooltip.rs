use crate::components::ExtendedHitbox;
use crate::utils::ToArray;
use crate::{
    DynamicSize, Element, ElementId, ElementImpl, ElementSize, ElementWorld, InteractionState,
    SizeConstraints, StateToParams, UiContext,
};
use vn_scene::{Rect, Scene};
use vn_ui_animation_macros::Interpolatable;
use web_time::{Duration, Instant};

#[derive(Clone, Copy, Debug, Default, Interpolatable)]
pub struct TooltipParams {
    #[interpolate_none_as_default]
    pub hover_delay: Option<Duration>,
    #[interpolate_none_as_default]
    pub hover_retain: Option<Duration>,
    pub interaction: InteractionState,
}

pub struct ToolTip<State: 'static> {
    id: ElementId,
    element: Box<dyn Element<State = State>>,
    tooltip: Box<dyn Element<State = State>>,
    params: StateToParams<State, TooltipParams>,
    show_tooltip: bool,
    tool_tip_size: ElementSize,
    hovered_last_at: Instant,
    hovered_start_at: Option<Instant>,
}

impl<State: 'static> ToolTip<State> {
    pub fn new(
        element: Box<dyn Element<State = State>>,
        tooltip: Box<dyn Element<State = State>>,
        params: StateToParams<State, TooltipParams>,
        world: &mut ElementWorld,
    ) -> Self {
        Self {
            tooltip: Box::new(ExtendedHitbox::new(tooltip, world)),
            id: world.next_id(),
            element,
            params,
            show_tooltip: false,
            tool_tip_size: ElementSize::ZERO,
            hovered_last_at: Instant::now(),
            hovered_start_at: None,
        }
    }
}

impl<State: 'static> ElementImpl for ToolTip<State> {
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
        let params = (self.params)(crate::StateToParamsArgs {
            state,
            id: self.id,
            ctx,
        });
        let is_hovered = params.interaction.is_hovered;
        let hover_delay = params.hover_delay.unwrap_or(Duration::from_secs_f32(0.1));
        let hover_retain = params.hover_retain.unwrap_or(Duration::from_secs_f32(0.1));

        match (self.show_tooltip, is_hovered, self.hovered_start_at) {
            // preparing to show tooltip
            (false, true, Some(start_at)) => {
                if Instant::now() - start_at > hover_delay {
                    self.show_tooltip = true;
                }
            }
            (false, true, None) => {
                self.hovered_start_at = Some(Instant::now());
            }
            (false, false, _) => {
                self.hovered_start_at = None;
            }
            // preparing to hide tooltip
            (true, false, _) => {
                if Instant::now() - self.hovered_last_at > hover_retain {
                    self.show_tooltip = false;
                }
            }
            (true, true, _) => {
                self.hovered_last_at = Instant::now();
            }
        }

        if self.show_tooltip {
            self.tool_tip_size = self.tooltip.layout(
                ctx,
                state,
                SizeConstraints {
                    min_size: ElementSize::ZERO,
                    max_size: DynamicSize {
                        width: Some(constraints.scene_size.0),
                        height: Some(constraints.scene_size.1),
                    },
                    scene_size: constraints.scene_size,
                },
            );
        }

        self.element
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
        let _params = (self.params)(crate::StateToParamsArgs {
            state,
            id: self.id,
            ctx,
        });
        ctx.with_hitbox_hierarchy(
            self.id,
            canvas.current_layer_id(),
            Rect {
                position: origin.to_array(),
                size: size.to_array(),
            },
            |ctx| {
                self.element.draw(ctx, state, origin, size, canvas);
                if self.show_tooltip {
                    // todo: to some more intelligent positioning of the tooltip
                    let tooltip_origin = (origin.0, origin.1 - self.tool_tip_size.height - 10.0);

                    canvas.with_next_layer(&mut |canvas| {
                        self.tooltip
                            .draw(ctx, state, tooltip_origin, self.tool_tip_size, canvas)
                    });
                }
            },
        );
    }
}

pub trait ToolTipExt: Element {
    fn tooltip(
        self,
        tooltip: Box<dyn Element<State = Self::State>>,
        params: StateToParams<Self::State, TooltipParams>,
        world: &mut ElementWorld,
    ) -> ToolTip<Self::State>;
}

impl<E: Element + 'static> ToolTipExt for E {
    fn tooltip(
        self,
        tooltip: Box<dyn Element<State = Self::State>>,
        params: StateToParams<Self::State, TooltipParams>,
        world: &mut ElementWorld,
    ) -> ToolTip<Self::State> {
        ToolTip::new(Box::new(self), tooltip, params, world)
    }
}
