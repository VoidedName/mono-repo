use crate::LayoutCache;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

#[derive(Clone, Copy, Eq, PartialEq, Hash, Debug)]
pub struct ElementId(pub u32);

#[derive(Debug, Clone, Copy)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}

#[derive(Debug, Clone)]
pub struct InteractionEvent {
    pub target: Option<ElementId>,
    pub kind: InteractionEventKind,
}

#[derive(Debug, Clone)]
pub enum InteractionEventKind {
    MouseMove { x: f32, y: f32 },
    MouseDown { button: MouseButton, x: f32, y: f32 },
    MouseUp { button: MouseButton, x: f32, y: f32 },
    Click { button: MouseButton, x: f32, y: f32 },
    MouseEnter,
    MouseLeave,
    FocusGained,
    FocusLost,
    Keyboard(crate::KeyEvent),
}

pub struct EventManager {
    insertion_order: u32,
    hitboxes: HashMap<ElementId, (u32, u32, crate::Rect)>, // id -> (layer, insertion_order, bounds)
    hovered_elements: HashSet<ElementId>,
    focused_element: Option<ElementId>,
    // We might need a parent mapping to implement bubbling correctly if we don't do it during tree traversal
    parents: HashMap<ElementId, ElementId>,
    event_queue: Vec<InteractionEvent>,
}

impl EventManager {
    pub fn new() -> Self {
        Self {
            insertion_order: 0,
            hitboxes: HashMap::new(),
            hovered_elements: HashSet::new(),
            focused_element: None,
            parents: HashMap::new(),
            event_queue: Vec::new(),
        }
    }

    pub fn queue_event(&mut self, kind: InteractionEventKind) {
        self.event_queue
            .push(InteractionEvent { target: None, kind });
    }

    pub fn process_events(&mut self) -> Vec<InteractionEvent> {
        let queue = std::mem::take(&mut self.event_queue);
        let mut all_events = Vec::new();

        for event in queue {
            match event.kind {
                InteractionEventKind::MouseMove { x, y } => {
                    all_events.extend(self.handle_mouse_move(x, y));
                }
                InteractionEventKind::MouseDown { button, x, y } => {
                    all_events.extend(self.handle_mouse_down(x, y, button));
                }
                InteractionEventKind::MouseUp { button, x, y } => {
                    all_events.extend(self.handle_mouse_up(x, y, button));
                }
                InteractionEventKind::Keyboard(key_event) => {
                    all_events.extend(self.handle_key(&key_event));
                }
                _ => {}
            }
        }

        all_events
    }

    pub fn register_hitbox(&mut self, id: ElementId, layer: u32, bounds: crate::Rect) {
        self.hitboxes
            .insert(id, (layer, self.insertion_order, bounds));
        self.insertion_order += 1;
    }

    pub fn clear_hitboxes(&mut self) {
        self.hitboxes.clear();
        self.parents.clear();
        self.insertion_order = 0;
    }

    pub fn set_parent(&mut self, child: ElementId, parent: ElementId) {
        self.parents.insert(child, parent);
    }

    pub fn is_hovered(&self, id: ElementId) -> bool {
        self.hovered_elements.contains(&id)
    }

    pub fn is_focused(&self, id: ElementId) -> bool {
        self.focused_element == Some(id)
    }

    pub fn handle_mouse_move(&mut self, x: f32, y: f32) -> Vec<InteractionEvent> {
        let top_hit = self.get_top_hit(x, y);

        let mut new_hovered = HashSet::new();
        if let Some(mut current) = top_hit {
            new_hovered.insert(current);
            while let Some(parent) = self.parents.get(&current) {
                new_hovered.insert(*parent);
                current = *parent;
            }
        }

        let mut events = Vec::new();

        // Elements that lost hover
        for id in &self.hovered_elements {
            if !new_hovered.contains(id) {
                events.push(InteractionEvent {
                    target: Some(*id),
                    kind: InteractionEventKind::MouseLeave,
                });
            }
        }

        // Elements that gained hover
        for id in &new_hovered {
            if !self.hovered_elements.contains(id) {
                events.push(InteractionEvent {
                    target: Some(*id),
                    kind: InteractionEventKind::MouseEnter,
                });
            }
        }

        // Always push MouseMove to the top hit
        if let Some(id) = top_hit {
            let bounds = self.hitboxes.get(&id).unwrap().2;
            events.push(InteractionEvent {
                target: Some(id),
                kind: InteractionEventKind::MouseMove {
                    x: x - bounds.position[0],
                    y: y - bounds.position[1],
                },
            });
        }

        self.hovered_elements = new_hovered;

        events
    }

    fn get_top_hit(&self, x: f32, y: f32) -> Option<ElementId> {
        let mut hits = self
            .hitboxes
            .iter()
            .filter(|(_, (_, _, rect))| rect.contains([x, y]))
            .map(|(id, (layer, order, _))| (*id, *layer, *order))
            .collect::<Vec<_>>();

        // Sort by layer (highest first, then newest)
        hits.sort_by(|(_, layer1, order1), (_, layer2, order2)| {
            let cmp = layer2.cmp(&layer1);
            if cmp == std::cmp::Ordering::Equal {
                order2.cmp(&order1)
            } else {
                cmp
            }
        });

        hits.first().map(|(id, _, _)| *id)
    }

    pub fn handle_mouse_down(
        &mut self,
        x: f32,
        y: f32,
        button: MouseButton,
    ) -> Vec<InteractionEvent> {
        let top_hit = self.get_top_hit(x, y);
        let mut events = Vec::new();

        if let Some(id) = top_hit {
            let bounds = self.hitboxes.get(&id).unwrap().2;
            events.push(InteractionEvent {
                target: Some(id),
                kind: InteractionEventKind::MouseDown {
                    button,
                    x: x - bounds.position[0],
                    y: y - bounds.position[1],
                },
            });
            if self.focused_element != Some(id) {
                if let Some(old_id) = self.focused_element {
                    events.push(InteractionEvent {
                        target: Some(old_id),
                        kind: InteractionEventKind::FocusLost,
                    });
                }
                self.focused_element = Some(id);
                events.push(InteractionEvent {
                    target: Some(id),
                    kind: InteractionEventKind::FocusGained,
                });
            }
        } else {
            if let Some(old_id) = self.focused_element {
                events.push(InteractionEvent {
                    target: Some(old_id),
                    kind: InteractionEventKind::FocusLost,
                });
            }
            self.focused_element = None;
        }

        events
    }

    pub fn handle_key(&mut self, event: &crate::KeyEvent) -> Vec<InteractionEvent> {
        let mut events = Vec::new();
        events.push(InteractionEvent {
            target: self.focused_element,
            kind: InteractionEventKind::Keyboard(event.clone()),
        });
        events
    }

    pub fn handle_mouse_up(
        &mut self,
        x: f32,
        y: f32,
        button: MouseButton,
    ) -> Vec<InteractionEvent> {
        let top_hit = self.get_top_hit(x, y);
        let mut events = Vec::new();

        if let Some(id) = top_hit {
            let bounds = self.hitboxes.get(&id).unwrap().2;
            events.push(InteractionEvent {
                target: Some(id),
                kind: InteractionEventKind::MouseUp {
                    button,
                    x: x - bounds.position[0],
                    y: y - bounds.position[1],
                },
            });

            if self.focused_element == Some(id) {
                events.push(InteractionEvent {
                    target: Some(id),
                    kind: InteractionEventKind::Click {
                        button,
                        x: x - bounds.position[0],
                        y: y - bounds.position[1],
                    },
                });
            }
        }

        events
    }
}

pub struct UiContext {
    pub event_manager: Rc<RefCell<EventManager>>,
    pub parent_id: Option<ElementId>,
    /// Since the layout cache is used to determine if one should reflow an element but is not
    /// sensitive to parameter changes, we MUST supply a fresh cache for each render cycle
    pub layout_cache: Box<dyn LayoutCache>,
    pub interactive: bool,
    /// Now should never change within a render cycle (i.e. between layout and render calls)
    pub now: web_time::Instant,
}

impl UiContext {
    pub fn new(
        event_manager: Rc<RefCell<EventManager>>,
        layout_cache: Box<dyn LayoutCache>,
        now: web_time::Instant,
    ) -> Self {
        Self {
            event_manager,
            parent_id: None,
            layout_cache,
            interactive: true,
            now,
        }
    }

    pub fn with_hitbox_hierarchy<F>(&mut self, id: ElementId, layer: u32, bounds: crate::Rect, f: F)
    where
        F: FnOnce(&mut Self),
    {
        if self.interactive {
            self.event_manager
                .borrow_mut()
                .register_hitbox(id, layer, bounds);
            if let Some(parent) = self.parent_id {
                self.event_manager.borrow_mut().set_parent(id, parent);
            }
        }

        let old_parent = self.parent_id;
        self.parent_id = Some(id);

        f(self);

        self.parent_id = old_parent;
    }

    pub fn with_interactivity<F>(&mut self, interactive: bool, f: F)
    where
        F: FnOnce(&mut Self),
    {
        let old_interactive = self.interactive;
        self.interactive = interactive;
        f(self);
        self.interactive = old_interactive;
    }
}
