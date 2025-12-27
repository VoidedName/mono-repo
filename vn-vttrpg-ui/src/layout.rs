use crate::{
    ConcreteSize, DynamicSize, Element, EventHandler, SizeConstraints, UiEvents, UiMouseEvent,
};
use std::cell::RefCell;
use std::sync::Arc;
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
    fn layout(&mut self, constraints: SizeConstraints) -> ConcreteSize {
        self.child_size = self.child.layout(constraints);

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

    fn draw_impl(&mut self, origin: (f32, f32), size: ConcreteSize, scene: &mut Scene) {
        match self.params.location {
            AnchorLocation::TOP => {
                self.child.draw(
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
                    (
                        origin.0 + (size.width - self.child_size.width),
                        origin.1 + size.height / 2.0 - self.child_size.height / 2.0,
                    ),
                    self.child_size,
                    scene,
                );
            }

            AnchorLocation::TopLeft => {
                self.child.draw(origin, self.child_size, scene);
            }
            AnchorLocation::TopRight => {
                self.child.draw(
                    (origin.0 + size.width - self.child_size.width, origin.1),
                    self.child_size,
                    scene,
                );
            }
            AnchorLocation::BottomLeft => self.child.draw(
                (origin.0, origin.1 + size.height - self.child_size.height),
                self.child_size,
                scene,
            ),
            AnchorLocation::BottomRight => self.child.draw(
                (
                    origin.0 + size.width - self.child_size.width,
                    origin.1 + size.height - self.child_size.height,
                ),
                self.child_size,
                scene,
            ),

            AnchorLocation::CENTER => self.child.draw(
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
    fn layout(&mut self, constraints: SizeConstraints) -> ConcreteSize {
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

        self.child_size = self.child.layout(child_constraints);
        ConcreteSize {
            width: self.child_size.width + margin,
            height: self.child_size.height + margin,
        }
        .clamp_to_constraints(constraints)
    }

    fn draw_impl(&mut self, origin: (f32, f32), size: ConcreteSize, scene: &mut Scene) {
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
    fn layout(&mut self, constraints: SizeConstraints) -> ConcreteSize {
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
            let child_size = child.layout(child_constraints);

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

    fn draw_impl(&mut self, origin: (f32, f32), size: ConcreteSize, scene: &mut Scene) {
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

                    child.draw((offset, origin.1), child_size, scene);
                    offset += self.layout[idx].width;
                }
                FlexDirection::Column => {
                    // making sure we are not drawing out of bounds for some reason
                    child_size.width = child_size.width.min(size.width);
                    child_size.height = child_size.height.min(size.height - (offset - origin.1));

                    child.draw((origin.0, offset), child_size, scene);
                    offset += self.layout[idx].height;
                }
            }
        }
    }
}

// extended hitbox works for non interactive tooltips, but a tooltip that has a tooltip
// will not work this way, as we are unaware of the sub-tooltips extended hitbox.
// should we instead finally start constructing elements with ids and trigger some
// "has mouse focus" event? What would this event look like?
//      MouseFocus { target_id: ElementId } // if the parent knows the id, they can now
//                                              also become focused, but what about their parent?
//                                              and should it be "focused" anyway?
pub struct ToolTip {
    ui_id: u32,
    ui_events: Arc<RefCell<UiEvents>>,
    extended_hitbox: ExtendedHitbox,
    element: Box<dyn Element>,
    tooltip: Box<dyn Element>,
    show_tooltip: bool,
    tool_tip_size: ConcreteSize,
    mouse_over_time: Arc<RefCell<Instant>>,
}

impl ToolTip {
    pub fn new(
        element: Box<dyn Element>,
        tooltip: Box<dyn Element>,
        ui_events: Arc<RefCell<UiEvents>>,
    ) -> Self {
        let ui_events = ui_events.clone();
        let mouse_over_time = Arc::new(RefCell::new(Instant::now() - Duration::new(1, 0)));
        let mouse_was_in_hitbox_last_at = mouse_over_time.clone();

        let handler = Arc::new(RefCell::new(move |event: UiMouseEvent| {
            if let UiMouseEvent::Moved { .. } = event {
                mouse_was_in_hitbox_last_at.replace(Instant::now());
            }
        }));

        let ui_id = ui_events.borrow_mut().register_hitbox(
            0,
            Rect {
                position: [-1.0, -1.0],
                size: [0.0, 0.0],
            },
            handler.clone(),
        );

        let extended_hitbox = ExtendedHitbox::new(ConcreteSize::ZERO, ui_events.clone(), handler);

        Self {
            ui_id,
            ui_events,
            extended_hitbox,
            element,
            tooltip,
            show_tooltip: false,
            mouse_over_time,
            tool_tip_size: ConcreteSize::ZERO,
        }
    }
}

impl Drop for ToolTip {
    fn drop(&mut self) {
        self.ui_events.borrow_mut().deregister_hitbox(self.ui_id);
    }
}

impl Element for ToolTip {
    fn layout(&mut self, constraints: SizeConstraints) -> ConcreteSize {
        // serious question, should layout precompute things like positions and sizes or just
        // give an "estimate"?
        // does the next draw call HAVE to honour the size returned by layout?
        // or do we return a min / max?
        // cause if we do not store stuff like that, then we don't need to layout the tooltip itself
        self.show_tooltip = self.mouse_over_time.borrow().elapsed().as_secs_f32() < 0.1;

        if self.show_tooltip {
            self.tool_tip_size = self.tooltip.layout(SizeConstraints {
                min_size: ConcreteSize {
                    width: 0.0,
                    height: 0.0,
                },
                max_size: DynamicSize {
                    width: Some(constraints.scene_size.0),
                    height: Some(constraints.scene_size.1),
                },
                scene_size: constraints.scene_size,
            });
            self.extended_hitbox.bbox = self.tool_tip_size;
        } else {
            self.extended_hitbox.bbox = ConcreteSize::ZERO;
        }

        self.element.layout(constraints)
    }

    fn draw_impl(&mut self, origin: (f32, f32), size: ConcreteSize, scene: &mut Scene) {
        self.ui_events.borrow_mut().update_hitbox(
            self.ui_id,
            Rect {
                position: [origin.0, origin.1],
                size: [size.width, size.height],
            },
        );

        self.element.draw(origin, size, scene);
        if self.show_tooltip {
            // todo: to some more intelligent positioning of the tooltip
            scene.with_next_layer(|scene| {
                self.tooltip.draw(
                    (origin.0, origin.1 - self.tool_tip_size.height),
                    self.tool_tip_size,
                    scene,
                )
            });

            self.extended_hitbox.draw_impl(
                (origin.0, origin.1 - self.tool_tip_size.height),
                self.tool_tip_size,
                scene,
            );
        } else {
            self.extended_hitbox
                .draw_impl(origin, ConcreteSize::ZERO, scene);
        }
    }
}

/// You can use this to compose a hitbox from multiple rectangles and potentially draw them outside bounds.
/// As this box is only logical, it is fine to "draw" it where you want.
pub struct ExtendedHitbox {
    ui_id: u32,
    ui_events: Arc<RefCell<UiEvents>>,
    bbox: ConcreteSize,
}

impl ExtendedHitbox {
    pub fn new(
        size: ConcreteSize,
        ui_events: Arc<RefCell<UiEvents>>,
        callback: EventHandler,
    ) -> Self {
        let id = ui_events.borrow_mut().register_hitbox(
            0,
            Rect {
                position: [-1.0, -1.0],
                size: [0.0, 0.0],
            },
            callback,
        );

        Self {
            ui_id: id,
            ui_events,
            bbox: size,
        }
    }
}

impl Drop for ExtendedHitbox {
    fn drop(&mut self) {
        self.ui_events.borrow_mut().deregister_hitbox(self.ui_id);
    }
}

impl Element for ExtendedHitbox {
    fn layout(&mut self, constraints: SizeConstraints) -> ConcreteSize {
        self.bbox.clamp_to_constraints(constraints)
    }

    fn draw_impl(&mut self, origin: (f32, f32), size: ConcreteSize, _scene: &mut Scene) {
        self.ui_events.borrow_mut().update_hitbox(
            self.ui_id,
            Rect {
                position: [origin.0, origin.1],
                size: [size.width, size.height],
            },
        );
    }
}
