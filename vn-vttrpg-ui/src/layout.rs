use crate::{ConcreteSize, DynamicSize, Element, ElementId, SizeConstraints, UiContext};
use vn_utils::UpdateOption;
use vn_vttrpg_window::{BoxPrimitive, Color, Rect, Scene};
use web_time::{Duration, Instant};

/// Specifies where a child element should be anchored within its container.
#[derive(Clone, Copy)]
pub enum AnchorLocation {
    TOP,
    BOTTOM,
    LEFT,
    RIGHT,

    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,

    CENTER,
}

#[derive(Clone, Copy)]
pub struct AnchorParams {
    pub location: AnchorLocation,
}

/// A container that anchors its child to a specific location.
pub struct Anchor {
    child: Box<dyn Element>,
    child_size: ConcreteSize,
    params: AnchorParams,
}

impl Anchor {
    pub fn new(child: Box<dyn Element>, params: AnchorParams) -> Self {
        Self {
            child,
            child_size: ConcreteSize::ZERO,
            params,
        }
    }
}

impl Element for Anchor {
    fn layout(&mut self, ctx: &mut UiContext, constraints: SizeConstraints) -> ConcreteSize {
        self.child_size = self.child.layout(ctx, constraints);

        // if a component has no constraints, use its child's size
        let width = match constraints.max_size.width {
            Some(w) => w,
            None => self.child_size.width,
        };

        let height = match constraints.max_size.height {
            Some(w) => w,
            None => self.child_size.height,
        };

        ConcreteSize { width, height }
    }

    fn draw_impl(
        &mut self,
        ctx: &mut UiContext,
        origin: (f32, f32),
        size: ConcreteSize,
        scene: &mut Scene,
    ) {
        match self.params.location {
            AnchorLocation::TOP => {
                self.child.draw(
                    ctx,
                    (
                        origin.0 + size.width / 2.0 - self.child_size.width / 2.0,
                        origin.1,
                    ),
                    self.child_size,
                    scene,
                );
            }
            AnchorLocation::BOTTOM => {
                self.child.draw(
                    ctx,
                    (
                        origin.0 + size.width / 2.0 - self.child_size.width / 2.0,
                        origin.1 + (size.height - self.child_size.height),
                    ),
                    self.child_size,
                    scene,
                );
            }
            AnchorLocation::LEFT => {
                self.child.draw(
                    ctx,
                    (
                        origin.0,
                        origin.1 + size.height / 2.0 - self.child_size.height / 2.0,
                    ),
                    self.child_size,
                    scene,
                );
            }
            AnchorLocation::RIGHT => {
                self.child.draw(
                    ctx,
                    (
                        origin.0 + (size.width - self.child_size.width),
                        origin.1 + size.height / 2.0 - self.child_size.height / 2.0,
                    ),
                    self.child_size,
                    scene,
                );
            }

            AnchorLocation::TopLeft => {
                self.child.draw(ctx, origin, self.child_size, scene);
            }
            AnchorLocation::TopRight => {
                self.child.draw(
                    ctx,
                    (origin.0 + size.width - self.child_size.width, origin.1),
                    self.child_size,
                    scene,
                );
            }
            AnchorLocation::BottomLeft => self.child.draw(
                ctx,
                (origin.0, origin.1 + size.height - self.child_size.height),
                self.child_size,
                scene,
            ),
            AnchorLocation::BottomRight => self.child.draw(
                ctx,
                (
                    origin.0 + size.width - self.child_size.width,
                    origin.1 + size.height - self.child_size.height,
                ),
                self.child_size,
                scene,
            ),
            AnchorLocation::CENTER => self.child.draw(
                ctx,
                (
                    origin.0 + size.width / 2.0 - self.child_size.width / 2.0,
                    origin.1 + size.height / 2.0 - self.child_size.height / 2.0,
                ),
                self.child_size,
                scene,
            ),
        }
    }
}

#[derive(Clone, Copy)]
pub struct CardParams {
    pub background_color: Color,
    pub border_size: f32,
    pub border_color: Color,
    pub corner_radius: f32,
}

pub struct Card {
    child: Box<dyn Element>,
    child_size: ConcreteSize,
    params: CardParams,
}

impl Card {
    pub fn new(child: Box<dyn Element>, params: CardParams) -> Self {
        Self {
            child,
            child_size: ConcreteSize::ZERO,
            params,
        }
    }
}

impl Element for Card {
    fn layout(&mut self, ctx: &mut UiContext, constraints: SizeConstraints) -> ConcreteSize {
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

        ConcreteSize {
            width: self.child_size.width + margin,
            height: self.child_size.height + margin,
        }
        .clamp_to_constraints(constraints)
    }

    fn draw_impl(
        &mut self,
        ctx: &mut UiContext,
        origin: (f32, f32),
        size: ConcreteSize,
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
            ConcreteSize {
                width: size.width.max(margin) - margin,
                height: size.height.max(margin) - margin,
            },
            scene,
        );
    }
}

#[derive(Clone, Copy)]
pub enum FlexDirection {
    Row,
    Column,
}

#[derive(Clone, Copy)]
pub struct FlexParams {
    pub direction: FlexDirection,
}

pub struct Flex {
    children: Vec<Box<dyn Element>>,
    layout: Vec<ConcreteSize>,
    params: FlexParams,
}

impl Flex {
    pub fn new(children: Vec<Box<dyn Element>>, params: FlexParams) -> Self {
        Self {
            layout: std::iter::repeat(ConcreteSize::ZERO)
                .take(children.len())
                .collect(),
            children,
            params,
        }
    }

    pub fn new_row(children: Vec<Box<dyn Element>>) -> Self {
        Self::new(
            children,
            FlexParams {
                direction: FlexDirection::Row,
            },
        )
    }

    pub fn new_column(children: Vec<Box<dyn Element>>) -> Self {
        Self::new(
            children,
            FlexParams {
                direction: FlexDirection::Column,
            },
        )
    }
}

// todo: allow for weight / spacing between children?
impl Element for Flex {
    fn layout(&mut self, ctx: &mut UiContext, constraints: SizeConstraints) -> ConcreteSize {
        // what do we do with containers that grow? like anchor?
        // do we extend constraints to denote that they should not grow along some axis?

        let mut total_in_direction: f32 = 0.0;
        let mut max_orthogonal: f32 = 0.0;

        let child_constraints = match self.params.direction {
            FlexDirection::Row => SizeConstraints {
                min_size: ConcreteSize {
                    width: 0.0,
                    height: constraints.min_size.height,
                },
                max_size: DynamicSize {
                    width: None,
                    height: constraints.max_size.height,
                },
                scene_size: constraints.scene_size,
            },
            FlexDirection::Column => SizeConstraints {
                min_size: ConcreteSize {
                    width: constraints.min_size.width,
                    height: 0.0,
                },
                max_size: DynamicSize {
                    width: constraints.max_size.width,
                    height: None,
                },
                scene_size: constraints.scene_size,
            },
        };

        for (idx, child) in self.children.iter_mut().enumerate() {
            let child_size = child.layout(ctx, child_constraints);

            match self.params.direction {
                FlexDirection::Row => {
                    total_in_direction += child_size.width;
                    max_orthogonal = max_orthogonal.max(child_size.height);
                }
                FlexDirection::Column => {
                    total_in_direction += child_size.height;
                    max_orthogonal = max_orthogonal.max(child_size.width);
                }
            }

            self.layout[idx] = child_size;
        }

        match self.params.direction {
            FlexDirection::Row => ConcreteSize {
                width: total_in_direction,
                height: max_orthogonal,
            },
            FlexDirection::Column => ConcreteSize {
                width: max_orthogonal,
                height: total_in_direction,
            },
        }
        .clamp_to_constraints(constraints)
    }

    fn draw_impl(
        &mut self,
        ctx: &mut UiContext,
        origin: (f32, f32),
        size: ConcreteSize,
        scene: &mut Scene,
    ) {
        let mut offset = match self.params.direction {
            FlexDirection::Row => origin.0,
            FlexDirection::Column => origin.1,
        };
        for (idx, child) in self.children.iter_mut().enumerate() {
            let mut child_size = self.layout[idx];

            match self.params.direction {
                FlexDirection::Row => {
                    // making sure we are not drawing out of bounds for some reason
                    child_size.width = child_size.width.min(size.width - (offset - origin.0));
                    child_size.height = child_size.height.min(size.height);

                    child.draw(ctx, (offset, origin.1), child_size, scene);
                    offset += self.layout[idx].width;
                }
                FlexDirection::Column => {
                    // making sure we are not drawing out of bounds for some reason
                    child_size.width = child_size.width.min(size.width);
                    child_size.height = child_size.height.min(size.height - (offset - origin.1));

                    child.draw(ctx, (origin.0, offset), child_size, scene);
                    offset += self.layout[idx].height;
                }
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct TooltipParams {
    pub hover_delay: Option<Duration>,
    pub hover_retain: Option<Duration>,
}

pub struct ToolTip {
    ui_id: ElementId,
    element: Box<dyn Element>,
    tooltip: Box<dyn Element>,
    show_tooltip: bool,
    tool_tip_size: ConcreteSize,
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
        let ui_id = ctx.event_manager.next_id();

        let hover_delay = params.hover_delay.unwrap_or(Duration::from_secs_f32(0.1));
        let hover_retain = params.hover_retain.unwrap_or(Duration::from_secs_f32(0.1));

        Self {
            ui_id,
            element,
            tooltip: Box::new(ExtendedHitbox::new(tooltip, ctx)),
            show_tooltip: false,
            tool_tip_size: ConcreteSize::ZERO,
            hovered_last_at: Instant::now() - hover_retain - hover_retain,
            hovered_start_at: None,
            hover_delay,
            hover_retain,
        }
    }
}

impl Element for ToolTip {
    fn layout(&mut self, ctx: &mut UiContext, constraints: SizeConstraints) -> ConcreteSize {
        let is_hovered = ctx.event_manager.is_hovered(self.ui_id);

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
                    min_size: ConcreteSize {
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

        self.element.layout(ctx, constraints)
    }

    fn draw_impl(
        &mut self,
        ctx: &mut UiContext,
        origin: (f32, f32),
        size: ConcreteSize,
        scene: &mut Scene,
    ) {
        ctx.with_hitbox_hierarchy(
            self.ui_id,
            scene.current_layer_id(),
            Rect {
                position: [origin.0, origin.1],
                size: [size.width, size.height],
            },
            |ctx| {
                self.element.draw(ctx, origin, size, scene);
                if self.show_tooltip {
                    let tooltip_origin = (origin.0, origin.1 - self.tool_tip_size.height - 10.0);

                    // todo: to some more intelligent positioning of the tooltip
                    scene.with_next_layer(|scene| {
                        self.tooltip
                            .draw(ctx, tooltip_origin, self.tool_tip_size, scene)
                    });
                }
            },
        );
    }
}

pub struct ExtendedHitbox {
    ui_id: ElementId,
    element: Box<dyn Element>,
}

impl ExtendedHitbox {
    pub fn new(element: Box<dyn Element>, ctx: &mut UiContext) -> Self {
        let ui_id = ctx.event_manager.next_id();
        Self { ui_id, element }
    }
}

impl Element for ExtendedHitbox {
    fn layout(&mut self, ctx: &mut UiContext, constraints: SizeConstraints) -> ConcreteSize {
        self.element.layout(ctx, constraints)
    }

    fn draw_impl(
        &mut self,
        ctx: &mut UiContext,
        origin: (f32, f32),
        size: ConcreteSize,
        scene: &mut Scene,
    ) {
        ctx.with_hitbox_hierarchy(
            self.ui_id,
            scene.current_layer_id(),
            Rect {
                position: [origin.0, origin.1],
                size: [size.width, size.height],
            },
            |ctx| {
                self.element.draw(ctx, origin, size, scene);
            },
        );
    }
}
