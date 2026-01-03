use crate::components::ExtendedHitbox;
use crate::utils::ToArray;
use crate::{
    DynamicSize, Element, ElementId, ElementImpl, ElementSize, SizeConstraints, UiContext,
};
use vn_scene::{Rect, Scene};
use vn_ui_animation_macros::Interpolatable;
use web_time::{Duration, Instant};

#[derive(Clone, Copy, Debug, Default, Interpolatable)]
pub struct TooltipParams {
    pub hover_delay: Option<Duration>,
    pub hover_retain: Option<Duration>,
}

pub struct ToolTip {
    id: ElementId,
    element: Box<dyn Element>,
    tooltip: Box<dyn Element>,
    show_tooltip: bool,
    tool_tip_size: ElementSize,
    hovered_last_at: Instant,
    hovered_start_at: Option<Instant>,
    hover_delay: Duration,
    hover_retain: Duration,
}

impl ToolTip {
    pub fn new(
        element: Box<dyn Element>,
        tooltip: Box<dyn Element>,
        params: TooltipParams,
        ctx: &mut UiContext,
    ) -> Self {
        let hover_delay = params.hover_delay.unwrap_or(Duration::from_secs_f32(0.1));
        let hover_retain = params.hover_retain.unwrap_or(Duration::from_secs_f32(0.1));

        Self {
            id: ctx.event_manager.next_id(),
            element,
            tooltip: Box::new(ExtendedHitbox::new(tooltip, ctx)),
            show_tooltip: false,
            tool_tip_size: ElementSize::ZERO,
            hovered_last_at: Instant::now() - hover_retain - hover_retain,
            hovered_start_at: None,
            hover_delay,
            hover_retain,
        }
    }
}

impl ElementImpl for ToolTip {
    fn id_impl(&self) -> ElementId {
        self.id
    }

    fn layout_impl(&mut self, ctx: &mut UiContext, constraints: SizeConstraints) -> ElementSize {
        let is_hovered = ctx.event_manager.is_hovered(self.id);

        match (self.show_tooltip, is_hovered, self.hovered_start_at) {
            // preparing to show tooltip
            (false, true, Some(start_at)) => {
                if Instant::now() - start_at > self.hover_delay {
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
                if Instant::now() - self.hovered_last_at > self.hover_retain {
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
                SizeConstraints {
                    min_size: ElementSize {
                        width: 0.0,
                        height: 0.0,
                    },
                    max_size: DynamicSize {
                        width: Some(constraints.scene_size.0),
                        height: Some(constraints.scene_size.1),
                    },
                    scene_size: constraints.scene_size,
                },
            );
        }

        self.element
            .layout(ctx, constraints)
            .clamp_to_constraints(constraints)
    }

    fn draw_impl(
        &mut self,
        ctx: &mut UiContext,
        origin: (f32, f32),
        size: ElementSize,
        canvas: &mut dyn Scene,
    ) {
        ctx.with_hitbox_hierarchy(
            self.id,
            canvas.current_layer_id(),
            Rect {
                position: origin.to_array(),
                size: size.to_array(),
            },
            |ctx| {
                self.element.draw(ctx, origin, size, canvas);
                if self.show_tooltip {
                    // todo: to some more intelligent positioning of the tooltip
                    let tooltip_origin = (origin.0, origin.1 - self.tool_tip_size.height - 10.0);

                    canvas.with_next_layer(&mut |canvas| {
                        self.tooltip
                            .draw(ctx, tooltip_origin, self.tool_tip_size, canvas)
                    });
                }
            },
        );
    }
}
