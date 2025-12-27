use std::collections::{HashMap, HashSet};
use vn_vttrpg_window::Rect;

#[derive(Clone, Copy, Eq, PartialEq, Hash, Debug)]
pub struct ElementId(pub u32);

#[derive(Debug, Clone, Copy)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}

#[derive(Debug, Clone, Copy)]
pub enum InteractionEvent {
    MouseMove { x: f32, y: f32 },
    MouseDown { button: MouseButton, x: f32, y: f32 },
    MouseUp { button: MouseButton, x: f32, y: f32 },
    MouseEnter,
    MouseLeave,
    FocusGained,
    FocusLost,
}

pub struct EventManager {
    next_id: u32,
    insertion_order: u32,
    hitboxes: HashMap<ElementId, (u32, u32, Rect)>, // id -> (layer, insertion_order, bounds)
    hovered_elements: HashSet<ElementId>,
    focused_element: Option<ElementId>,
    // We might need a parent mapping to implement bubbling correctly if we don't do it during tree traversal
    parents: HashMap<ElementId, ElementId>,
}

impl EventManager {
    pub fn new() -> Self {
        Self {
            next_id: 0,
            insertion_order: 0,
            hitboxes: HashMap::new(),
            hovered_elements: HashSet::new(),
            focused_element: None,
            parents: HashMap::new(),
        }
    }

    pub fn next_id(&mut self) -> ElementId {
        let id = ElementId(self.next_id);
        self.next_id += 1;
        id
    }

    pub fn register_hitbox(&mut self, id: ElementId, layer: u32, bounds: Rect) {
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

    pub fn handle_mouse_move(&mut self, x: f32, y: f32) -> Vec<(ElementId, InteractionEvent)> {
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

        // I need bubbling here...
        let top_hit = hits.first().map(|(id, _, _)| *id);

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
                events.push((*id, InteractionEvent::MouseLeave));
            }
        }

        // Elements that gained hover
        for id in &new_hovered {
            if !self.hovered_elements.contains(id) {
                events.push((*id, InteractionEvent::MouseEnter));
            }
        }

        // Always push MouseMove to the top hit
        if let Some(id) = top_hit {
            events.push((id, InteractionEvent::MouseMove { x, y }));
        }

        self.hovered_elements = new_hovered;

        // later going to get processed by the logic itself? or by the elements?
        // i would like my ui to only be concerned with the ui -> that means the logic
        // needs to be able to figure out something like element ids and what a click on one
        // of those would mean?
        // how do click animations then work? also via things like "is_clicked" or "is_triggered"?
        // so ui elements all just poll their state.
        // how far does this stretch? like a label / text, or arbitrary element information
        // do elements draw them from the logic itself / some other state? certainly not from here?
        // I guess i need to extend the ctx with some kind of logic state... or functions?
        // like a label would use a "get_text" callback function... how far? like same for colors?
        // Since the Ui elements actually hold state, i would not want to rebuild the tree...
        // unless i want to do something like react where i can query the ctx for local state values.
        // (either by call order or by element_id -> would require the Any trait though)
        events
    }
}

pub struct UiContext<'a> {
    pub event_manager: &'a mut EventManager,
    pub parent_id: Option<ElementId>,
}

impl UiContext<'_> {
    pub fn with_hitbox_hierarchy<F>(&mut self, id: ElementId, layer: u32, bounds: Rect, f: F)
    where
        F: FnOnce(&mut Self),
    {
        self.event_manager.register_hitbox(id, layer, bounds);
        if let Some(parent) = self.parent_id {
            self.event_manager.set_parent(id, parent);
        }

        let old_parent = self.parent_id;
        self.parent_id = Some(id);

        f(self);

        self.parent_id = old_parent;
    }
}
