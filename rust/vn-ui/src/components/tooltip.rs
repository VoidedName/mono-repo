use crate::components::ExtendedHitbox;
use crate::utils::ToArray;
use crate::{
    DynamicDimension, DynamicSize, Element, ElementId, ElementImpl, ElementSize, ElementWorld,
    InteractionState, SizeConstraints, StateToParams, UiContext, into_box_impl,
};
use std::cell::RefCell;
use std::rc::Rc;
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

pub struct ToolTip<State: 'static, Message: 'static> {
    id: ElementId,
    element: Box<dyn Element<State = State, Message = Message>>,
    tooltip: Box<dyn Element<State = State, Message = Message>>,
    params: StateToParams<State, TooltipParams>,
    show_tooltip: bool,
    tool_tip_size: ElementSize,
    hovered_last_at: Instant,
    hovered_start_at: Option<Instant>,
}

impl<State: 'static, Message: 'static> ToolTip<State, Message> {
    pub fn new<P: Into<StateToParams<State, TooltipParams>>>(
        element: impl Into<Box<dyn Element<State = State, Message = Message>>>,
        tooltip: impl Into<Box<dyn Element<State = State, Message = Message>>>,
        params: P,
        world: Rc<RefCell<ElementWorld>>,
    ) -> Self {
        Self {
            tooltip: Box::new(ExtendedHitbox::new(tooltip, world.clone())),
            id: world.borrow_mut().next_id(),
            element: element.into(),
            params: params.into(),
            show_tooltip: false,
            tool_tip_size: ElementSize::ZERO,
            hovered_last_at: Instant::now(),
            hovered_start_at: None,
        }
    }
}

impl<State: 'static, Message: 'static> ElementImpl for ToolTip<State, Message> {
    type State = State;
    type Message = Message;

    fn id_impl(&self) -> ElementId {
        self.id
    }

    fn layout_impl(
        &mut self,
        ctx: &mut UiContext,
        state: &Self::State,
        constraints: SizeConstraints,
    ) -> ElementSize {
        let params = self.params.call(crate::StateToParamsArgs {
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
                        width: DynamicDimension::Limit(constraints.scene_size.0),
                        height: DynamicDimension::Limit(constraints.scene_size.1),
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
        let _params = self.params.call(crate::StateToParamsArgs {
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

                    ctx.with_clipping(
                        Rect {
                            position: [origin.0, origin.1 - self.tool_tip_size.height - 10.0],
                            size: [self.tool_tip_size.width, self.tool_tip_size.height],
                        },
                        |ctx| {
                            let tooltip_origin =
                                (origin.0, origin.1 - self.tool_tip_size.height - 10.0);

                            canvas.with_next_layer(&mut |canvas| {
                                self.tooltip.draw(
                                    ctx,
                                    state,
                                    tooltip_origin,
                                    self.tool_tip_size,
                                    canvas,
                                )
                            });
                        },
                    )
                }
            },
        );
    }

    fn handle_event_impl(
        &mut self,
        ctx: &mut UiContext,
        state: &Self::State,
        event: &crate::InteractionEvent,
    ) -> Vec<Self::Message> {
        let mut messages = self.element.handle_event(ctx, state, event);
        if self.show_tooltip {
            messages.extend(self.tooltip.handle_event(ctx, state, event));
        }
        messages
    }
}

pub trait ToolTipExt<State, Message> {
    fn tooltip<P: Into<StateToParams<State, TooltipParams>>>(
        self,
        tooltip: impl Into<Box<dyn Element<State = State, Message = Message>>>,
        params: P,
        world: Rc<RefCell<ElementWorld>>,
    ) -> ToolTip<State, Message>;
}

impl<State, Message, E: Into<Box<dyn Element<State = State, Message = Message>>> + 'static>
    ToolTipExt<State, Message> for E
{
    fn tooltip<P: Into<StateToParams<State, TooltipParams>>>(
        self,
        tooltip: impl Into<Box<dyn Element<State = State, Message = Message>>>,
        params: P,
        world: Rc<RefCell<ElementWorld>>,
    ) -> ToolTip<State, Message> {
        ToolTip::new(self, tooltip, params, world)
    }
}

into_box_impl!(ToolTip);
