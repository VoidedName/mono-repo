use crate::context::UiContext;
use crate::event::{ElementId, InteractionEvent};
use crate::geometry::ElementSize;
use crate::layout::SizeConstraints;
use std::cell::{Ref, RefCell, RefMut};
use std::rc::Rc;
use vn_scene::Scene;

/// Represents a UI component that can be laid out and drawn.
/// Elements implementing this trait can use the `UiElement` derive macro to automatically implement `ElementImpl`.
pub trait Component {
    type State: 'static;
    type Message: Clone + 'static;
    type Params: 'static;

    fn layout(
        &mut self,
        ctx: &mut UiContext,
        state: &Self::State,
        params: &Self::Params,
        constraints: SizeConstraints,
    ) -> ElementSize;

    fn draw(
        &mut self,
        ctx: &mut UiContext,
        state: &Self::State,
        params: &Self::Params,
        origin: (f32, f32),
        size: ElementSize,
        scene: &mut dyn Scene,
    );

    fn handle_event(
        &mut self,
        ctx: &mut UiContext,
        state: &Self::State,
        params: &Self::Params,
        event: &InteractionEvent,
    ) -> Vec<Self::Message>;

    fn invalidated(&self, _ctx: &UiContext, _state: &Self::State, _params: &Self::Params) -> bool {
        true
    }
}

pub struct ChildElement<State, Message> {
    child: Rc<RefCell<dyn Element<State = State, Message = Message>>>,
}

impl<State, Message> Clone for ChildElement<State, Message> {
    fn clone(&self) -> Self {
        Self {
            child: self.child.clone(),
        }
    }
}

impl<State, Message> ChildElement<State, Message> {
    pub fn new(
        child: impl Into<Rc<RefCell<dyn Element<State = State, Message = Message>>>>,
    ) -> Self {
        Self {
            child: child.into(),
        }
    }

    pub fn borrow<'a>(&'a self) -> Ref<'a, dyn Element<State = State, Message = Message>> {
        self.child.borrow()
    }

    pub fn borrow_mut<'a>(&'a self) -> RefMut<'a, dyn Element<State = State, Message = Message>> {
        self.child.borrow_mut()
    }
}

impl<I: Into<Rc<RefCell<dyn Element<State = State, Message = Message>>>>, State, Message> From<I>
    for ChildElement<State, Message>
{
    fn from(value: I) -> Self {
        Self::new(value)
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
        ctx: &mut UiContext,
        state: &Self::State,
        event: &InteractionEvent,
    ) -> Vec<Self::Message>;

    /// Returns true if the element needs to be recalculated (layout or draw).
    /// This is used to invalidate caches.
    fn invalidated_impl(&self, _ctx: &UiContext, _state: &Self::State) -> bool {
        true
    }
}

/// Represents a UI element that can be laid out and drawn.
/// This trait is automatically implemented for all types that implement [ElementImpl].
pub trait Element: ElementImpl {
    /// Returns the unique ID of this element.
    fn id(&self) -> ElementId {
        self.id_impl()
    }

    /// Returns true if the element needs to be recalculated.
    fn invalidated(&self, ctx: &UiContext, state: &Self::State) -> bool {
        self.invalidated_impl(ctx, state)
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
        if !self.invalidated(ctx, state) {
            if let Some(cached_size) = ctx.layout_cache.lookup(self.id(), constraints) {
                return cached_size;
            }
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

impl<State: 'static, Message: 'static, T: ElementImpl<State = State, Message = Message>> Element
    for T
{
}

pub struct StateToParamsArgs<'a, State: 'static> {
    pub state: &'a State,
    pub id: ElementId,
    pub ctx: &'a UiContext,
}

pub struct StateToParams<State: 'static, Params: 'static>(
    pub Box<dyn Fn(StateToParamsArgs<State>) -> Params>,
);

impl<State: 'static, Params: 'static> StateToParams<State, Params> {
    pub fn new<F: Fn(StateToParamsArgs<State>) -> Params + 'static>(f: F) -> Self {
        Self(Box::new(f))
    }

    pub fn call(&self, args: StateToParamsArgs<State>) -> Params {
        self.0(args)
    }
}

impl<State: 'static, Params: 'static, F> From<F> for StateToParams<State, Params>
where
    F: Fn(StateToParamsArgs<State>) -> Params + 'static,
{
    fn from(f: F) -> Self {
        Self(Box::new(f))
    }
}
