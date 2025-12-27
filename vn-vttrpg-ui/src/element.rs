use crate::{ConcreteSize, ElementId, SizeConstraints, UiContext};
use std::collections::HashMap;
use vn_vttrpg_window::Scene;

pub struct SimpleLayoutCache {
    cache: HashMap<ElementId, (SizeConstraints, ConcreteSize)>,
}

impl SimpleLayoutCache {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
        }
    }
}

pub trait LayoutCache {
    fn lookup(&self, element_id: ElementId, constraints: SizeConstraints) -> Option<ConcreteSize>;
    fn cache(&mut self, element_id: ElementId, constraints: SizeConstraints, size: ConcreteSize);
}

impl LayoutCache for SimpleLayoutCache {
    fn lookup(&self, element_id: ElementId, constraints: SizeConstraints) -> Option<ConcreteSize> {
        self.cache
            .get(&element_id)
            .map(|(cached_constraints, s)| {
                if constraints == *cached_constraints {
                    Some(*s)
                } else {
                    None
                }
            })
            .flatten()
    }

    fn cache(&mut self, element_id: ElementId, constraints: SizeConstraints, size: ConcreteSize) {
        self.cache.insert(element_id, (constraints, size));
    }
}

// implement layout caching here?

/// Represents a UI element that can be laid out and drawn.
pub trait Element {
    fn id(&self) -> ElementId;

    fn layout(&mut self, ctx: &mut UiContext, constraints: SizeConstraints) -> ConcreteSize {
        if let Some(cached_size) = ctx.layout_cache.lookup(self.id(), constraints) {
            return cached_size;
        }

        let size = self.layout_impl(ctx, constraints);

        ctx.layout_cache.cache(self.id(), constraints, size);

        size
    }

    /// Determines the size of the element given the layout constraints.
    fn layout_impl(&mut self, ctx: &mut UiContext, constraints: SizeConstraints) -> ConcreteSize;

    /// Call this method to draw the element at the specified origin with the given size into the scene.
    ///
    /// !!! IF YOU OVERWRITE THIS METHOD, DEBUG FEATURES WILL NOT WORK !!!
    fn draw(
        &mut self,
        ctx: &mut UiContext,
        origin: (f32, f32),
        size: ConcreteSize,
        scene: &mut Scene,
    ) {
        self.draw_impl(ctx, origin, size, scene);
        #[cfg(feature = "debug_outlines")]
        {
            use vn_vttrpg_window::BoxPrimitive;

            scene.with_next_layer(|scene| {
                scene.add_box(
                    BoxPrimitive::builder()
                        .transform(|t| t.translation([origin.0, origin.1]))
                        .size([size.width, size.height])
                        .color(vn_vttrpg_window::Color::GREEN.with_alpha(0.05))
                        .border_color(vn_vttrpg_window::Color::RED.with_alpha(0.25))
                        .border_thickness(5.0)
                        .build(),
                )
            });
        }
    }

    /// Draws the element at the specified origin with the given size into the scene.
    ///
    /// !!! DO NOT MANUALLY CALL THIS, CALL [draw](Self::draw) INSTEAD !!!
    fn draw_impl(
        &mut self,
        ctx: &mut UiContext,
        origin: (f32, f32),
        size: ConcreteSize,
        scene: &mut Scene,
    );
}
