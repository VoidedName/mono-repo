use crate::event::ElementId;
use crate::geometry::{DynamicSize, ElementSize, SceneSize};
use std::collections::HashMap;

pub trait LayoutCache {
    fn lookup(&self, element_id: ElementId, constraints: SizeConstraints) -> Option<ElementSize>;
    fn cache(&mut self, element_id: ElementId, constraints: SizeConstraints, size: ElementSize);
}

pub struct SimpleLayoutCache {
    pub cache: HashMap<ElementId, (SizeConstraints, ElementSize)>,
}

impl SimpleLayoutCache {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
        }
    }
}

impl LayoutCache for SimpleLayoutCache {
    fn lookup(&self, element_id: ElementId, constraints: SizeConstraints) -> Option<ElementSize> {
        self.cache
            .get(&element_id)
            .and_then(|(cached_constraints, s)| {
                if constraints == *cached_constraints {
                    Some(*s)
                } else {
                    None
                }
            })
    }

    fn cache(&mut self, element_id: ElementId, constraints: SizeConstraints, size: ElementSize) {
        self.cache.insert(element_id, (constraints, size));
    }
}

/// Defines the minimum and maximum size constraints for layout.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SizeConstraints {
    pub min_size: ElementSize,
    pub max_size: DynamicSize,
    pub scene_size: SceneSize,
}

impl SizeConstraints {
    pub fn shrink_by(&self, size: ElementSize) -> Self {
        Self {
            min_size: self.min_size.shrink_by(size),
            max_size: self.max_size.shrink_by(size),
            scene_size: self.scene_size,
        }
    }

    pub fn grow_by(&self, size: ElementSize) -> Self {
        Self {
            min_size: self.min_size.grow_by(size),
            max_size: self.max_size.grow_by(size),
            scene_size: self.scene_size,
        }
    }
}
