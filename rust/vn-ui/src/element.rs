use crate::{ElementId, ElementSize, InteractionEvent, SizeConstraints, UiContext};
use std::collections::HashMap;
use vn_scene::Scene;

pub struct SimpleLayoutCache {
    cache: HashMap<ElementId, (SizeConstraints, ElementSize)>,
}

impl SimpleLayoutCache {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
        }
    }
}

pub trait LayoutCache {
    fn lookup(&self, element_id: ElementId, constraints: SizeConstraints) -> Option<ElementSize>;
    fn cache(&mut self, element_id: ElementId, constraints: SizeConstraints, size: ElementSize);
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

/// Concrete implementation of an element. Implementing this automatically also implements [Element].
/// Use the [Element] trait to interact with elements and do not call anything in here manually.
pub trait ElementImpl {
    type State;
    type Message;

    /// This ID is used in both the layout cache and for identifying elements in the UI and **MUST**
    /// be unique for each element.
    fn id_impl(&self) -> ElementId;

    /// Implement the layouting work. It will be called before drawing the element.
    /// And you can assume that the size you return here is the size the element will be drawn with.
    ///
    /// !!! DO NOT MANUALLY CALL THIS, CALL [layout](Self::layout) INSTEAD !!!
    fn layout_impl(
        &mut self,
        ctx: &mut UiContext,
        state: &Self::State,
        constraints: SizeConstraints,
    ) -> ElementSize;

    /// Draws the element at the specified origin with the given size into the scene.
    ///
    /// !!! DO NOT MANUALLY CALL THIS, CALL [draw](Self::draw) INSTEAD !!!
    fn draw_impl(
        &mut self,
        ctx: &mut UiContext,
        state: &Self::State,
        origin: (f32, f32),
        size: ElementSize,
        scene: &mut dyn Scene,
    );

    /// Handles an interaction event.
    fn handle_event_impl(
        &mut self,
        _ctx: &mut UiContext,
        _state: &Self::State,
        _event: &InteractionEvent,
    ) -> Vec<Self::Message>;
}

/// Represents a UI element that can be laid out and drawn.
/// This trait is automatically implemented for all types that implement [ElementImpl].
pub trait Element: ElementImpl {
    /// Returns the unique ID of this element.
    fn id(&self) -> ElementId {
        self.id_impl()
    }

    /// Call this method to perform the layouting work. It must be called before drawing the element.
    /// And elements assume that the size they get drawn with is the size they report here.
    ///
    /// !!! IF YOU OVERWRITE THIS METHOD, YOU MUST IMPLEMENT LAYOUT CACHING YOURSELF !!!
    fn layout(
        &mut self,
        ctx: &mut UiContext,
        state: &Self::State,
        constraints: SizeConstraints,
    ) -> ElementSize {
        if let Some(cached_size) = ctx.layout_cache.lookup(self.id(), constraints) {
            return cached_size;
        }

        let size = self.layout_impl(ctx, state, constraints);

        ctx.layout_cache.cache(self.id(), constraints, size);

        size
    }

    /// Call this method to draw the element at the specified origin with the given size into the scene.
    ///
    /// !!! IF YOU OVERWRITE THIS METHOD, DEBUG FEATURES WILL NOT WORK !!!
    fn draw(
        &mut self,
        ctx: &mut UiContext,
        state: &Self::State,
        origin: (f32, f32),
        size: ElementSize,
        scene: &mut dyn Scene,
    ) {
        self.draw_impl(ctx, state, origin, size, scene);
        #[cfg(feature = "debug_outlines")]
        {
            use rand::rngs::SmallRng;
            use rand::{Rng, SeedableRng};
            use vn_scene::{BoxPrimitiveData, Color, Rect};
            let mut rng = SmallRng::seed_from_u64(self.id().0 as u64);
            use vn_scene::Transform;

            let color = Color {
                r: 1.0 - (rng.random_range(0.0..=1.0) as f32).powi(3),
                g: 1.0 - (rng.random_range(0.0..=1.0) as f32).powi(3),
                b: 1.0 - (rng.random_range(0.0..=1.0) as f32).powi(3),
                a: 1.0,
            };

            const DEBUG_THICKNESS: f32 = 4.0;

            scene.with_next_layer(&mut |scene| {
                scene.add_box(BoxPrimitiveData {
                    transform: Transform {
                        translation: [
                            origin.0 - DEBUG_THICKNESS / 2.0,
                            origin.1 - DEBUG_THICKNESS / 2.0,
                        ],
                        ..Transform::DEFAULT
                    },
                    size: [size.width + DEBUG_THICKNESS, size.height + DEBUG_THICKNESS],
                    color: Color::TRANSPARENT,
                    border_color: color.with_alpha(0.5),
                    border_thickness: DEBUG_THICKNESS,
                    border_radius: 0.0,
                    clip_rect: Rect::NO_CLIP,
                })
            });
        }
    }

    /// Handles an interaction event.
    fn handle_event(
        &mut self,
        ctx: &mut UiContext,
        state: &Self::State,
        event: &InteractionEvent,
    ) -> Vec<Self::Message> {
        log::trace!(
            "Start handling event {:?} for element {:?}",
            event,
            self.id()
        );
        let messages = self.handle_event_impl(ctx, state, event);
        log::trace!(
            "Finished handling event {:?} for element {:?}, sending {} messages",
            event,
            self.id(),
            messages.len()
        );
        messages
    }
}

impl<State, Message, T: ElementImpl<State = State, Message = Message>> Element for T {}
