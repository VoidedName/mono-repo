use std::cell::RefCell;
use std::rc::Rc;
use vn_scene::Rect;
use crate::event::{ElementId, EventManager};
use crate::layout::LayoutCache;

pub struct UiContext {
    pub event_manager: Rc<RefCell<EventManager>>,
    pub parent_id: Option<ElementId>,
    /// Since the layout cache is used to determine if one should reflow an element but is not
    /// sensitive to parameter changes, we MUST supply a fresh cache for each render cycle
    pub layout_cache: Box<dyn LayoutCache>,
    pub interactive: bool,
    pub clip_rect: Rect,
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
            clip_rect: Rect::NO_CLIP,
            now,
        }
    }

    pub fn with_hitbox_hierarchy<F>(&mut self, id: ElementId, layer: u32, bounds: Rect, f: F)
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

    pub fn with_clipping<F>(&mut self, clip_rect: Rect, f: F)
    where
        F: FnOnce(&mut Self),
    {
        let old_clip = self.clip_rect;
        let x1 = self.clip_rect.position[0].max(clip_rect.position[0]);
        let y1 = self.clip_rect.position[1].max(clip_rect.position[1]);
        let x2 = (self.clip_rect.position[0] + self.clip_rect.size[0])
            .min(clip_rect.position[0] + clip_rect.size[0]);
        let y2 = (self.clip_rect.position[1] + self.clip_rect.size[1])
            .min(clip_rect.position[1] + clip_rect.size[1]);

        let width = (x2 - x1).max(0.0);
        let height = (y2 - y1).max(0.0);

        self.clip_rect = Rect {
            position: [x1, y1],
            size: [width, height],
        };
        f(self);
        self.clip_rect = old_clip;
    }
}

pub struct ElementWorld {
    next_id: u32,
}

impl ElementWorld {
    pub fn new() -> Self {
        Self { next_id: 0 }
    }

    pub fn next_id(&mut self) -> ElementId {
        let id = ElementId(self.next_id);
        self.next_id += 1;
        id
    }
}
