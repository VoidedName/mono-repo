// consider if elements should be stored as box<dyn> internally, or maybe arc/rc, or maybe just
// as some "id" which gets looked up in an element storage
// those would allow me to access the elements without traversing the tree, like updating an
// element directly (let's say the fps stat)
// or should the elements go and lookup data instead when being rendered? <-- probably this, avoids
// stale references and my state is free from pollution.
// example could be some thing DynamicText::new( Arc<Logic>, | Arc<Logic> | -> String )
//
// this does not solve ui restructuring, i.e. changing the tree (example, switching menus)
// can also be solved with callbacks from the ui elements (assuming they receive events)
// events don't have to be keyboard and click events, we could just feed any arbitrary event?
// in this case we could not pass the arc logic but rather just some event handler / listener?
// then a component can simply listen to it
//
// just receiving events, like mouse position, is not quite sufficient though, since with stacked
// elements, i would not know which one is the click target. This means at least a click / mouse event
// or mouse focus event would need to propagate through the tree to find the first valid target.
//
// what states do i need? should they be managed within the ui tree? externally? via callbacks?
//
// if allow absolute positioning, i.e. an element is placed independently of the constraints, then
// finding a mouse target is unreasonable. would i register their locations in a spacial index?

mod element;
mod layout;
mod sizes;
mod event_manager;

pub use element::*;
pub use layout::*;
pub use sizes::*;
pub use event_manager::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;
use vn_vttrpg_window::{Color, Rect, Scene, TextPrimitive};

pub type ElementId = u32;

pub struct MouseFocusEvent {
    pub target_id: ElementId,
    pub child: Option<Box<MouseFocusEvent>>,
}

pub struct MouseMoveEvent {
    pub x: f32,
    pub y: f32,
}

pub enum UiMouseEvent {
    /// Mouse moved to this position. We do not track the path it took, although we could extend it
    /// in the future to do so.
    Moved(MouseMoveEvent),
}

// can I sub to the events of a specific component?
// wrapping elements would need to re-emit events with their id?
// every element needs access to an event manager and an id?
// we could then also wrap a child event manager to intercept all events emitted by it.
// though we would need to redispatch global events to it?

// this will need mutliple iterations i think...

pub type EventHandler = Arc<RefCell<dyn FnMut(UiMouseEvent) + 'static>>;

#[derive(Clone, Copy)]
struct HitBoxEntry {
    id: u32,
    layer: u32,
    bounds: Rect,
}

pub struct UiEvents {
    next_id: u32,
    hit_box_index: HashMap<u32, HitBoxEntry>,
    event_handlers: HashMap<u32, EventHandler>,
}

impl UiEvents {
    pub fn new() -> Self {
        Self {
            next_id: 0,
            hit_box_index: HashMap::new(),
            event_handlers: HashMap::new(),
        }
    }

    pub fn register_hitbox(&mut self, layer: u32, bounds: Rect, handler: EventHandler) -> u32 {
        self.hit_box_index.insert(
            self.next_id,
            HitBoxEntry {
                id: self.next_id,
                bounds,
                layer,
            },
        );
        self.event_handlers.insert(self.next_id, handler);
        self.next_id += 1;
        self.next_id - 1
    }

    pub fn update_hitbox(&mut self, id: u32, bounds: Rect) {
        self.hit_box_index.get_mut(&id).unwrap().bounds = bounds;
    }

    pub fn deregister_hitbox(&mut self, id: u32) {
        self.hit_box_index.remove(&id);
        self.event_handlers.remove(&id);
    }

    pub fn handle_mouse_position(&self, x: f32, y: f32) {
        // sort hits first by layer, then by id... i should do a better hit detect than just the box
        // since elements might have a non rectangular interactive hitbox (like circles)
        let mut hits = self
            .hit_box_index
            .iter()
            .filter(|(_, entry)| entry.bounds.contains([x, y]))
            .collect::<Vec<_>>();
        hits.sort_by(|(_, a), (_, b)| {
            let cmp = a.layer.cmp(&b.layer).reverse();
            if cmp == std::cmp::Ordering::Equal {
                a.id.cmp(&b.id).reverse()
            } else {
                cmp
            }
        });

        for (_, entry) in hits {
            let mut handler = self.event_handlers.get(&entry.id).unwrap().borrow_mut();
            handler(UiMouseEvent::Moved( MouseMoveEvent { x, y })); // does handler return "bubbling" and then stop the loop?
        }
    }
}

/// This keeps the UI agnostic to any specific graphics and resource management
pub trait TextMetrics {
    fn size_of_text(&self, text: &str, font: &str, font_size: f32) -> (f32, f32);
}

#[derive(Clone)]
pub struct LabelParams {
    pub text: String,
    pub font: String,
    pub font_size: f32,
    pub color: Color,
}

/// A UI element that renders a string of text.
pub struct Label {
    params: LabelParams,
    size: ConcreteSize,
}

impl Label {
    pub fn new<T: TextMetrics>(params: LabelParams, text_metrics: &T) -> Self {
        let size = text_metrics.size_of_text(&params.text, &params.font, params.font_size);

        Self {
            params,
            size: ConcreteSize {
                width: size.0,
                height: size.1,
            },
        }
    }
}

impl Element for Label {
    fn layout(&mut self, constraints: SizeConstraints) -> ConcreteSize {
        self.size.clamp_to_constraints(constraints)
    }

    fn draw_impl(&mut self, origin: (f32, f32), size: ConcreteSize, scene: &mut Scene) {
        scene.add_text(
            TextPrimitive::builder(self.params.text.clone(), self.params.font.clone())
                .transform(|t| t.translation([origin.0, origin.1]))
                // dunno if i should be squishing / stretching or clipping here...
                .size([self.size.width, self.size.height])
                .clip_area(|c| c.size([size.width, size.height]))
                .font_size(self.params.font_size)
                .tint(self.params.color)
                .build(),
        )
    }
}

pub struct ButtonParams {
    pub background: Color,
    pub border_color: Color,
    pub border_width: f32,
    pub corner_radius: f32,
}

// A button is basically just a card... but it should support some animations
// (maybe cards should also support them? is that in general? how do i animate things?
// primitive properties would be simple enough, but the parent has no idea about any other props
// should i restructure things to take in parameter objects and then in the constructor take an
// animator that returns those properties? then we could just tick the entire ui and it would grab
// it from those animators... those animators could also listen to events? they'd be specific to a
// component)
pub struct Button {
    child: Box<dyn Element>,
}

impl Button {
    pub fn new(child: Box<dyn Element>, params: ButtonParams) -> Self {
        Self {
            child: Box::new(Card::new(
                child,
                CardParams {
                    background_color: params.background,
                    border_color: params.border_color,
                    border_size: params.border_width,
                    corner_radius: params.corner_radius,
                },
            )),
        }
    }
}

impl Element for Button {
    fn layout(&mut self, constraints: SizeConstraints) -> ConcreteSize {
        self.child.layout(constraints)
    }

    fn draw_impl(&mut self, origin: (f32, f32), size: ConcreteSize, scene: &mut Scene) {
        self.child.draw(origin, size, scene);
    }
}
