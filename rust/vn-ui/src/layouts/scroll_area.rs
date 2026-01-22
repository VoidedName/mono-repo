use crate::{
    Element, ElementId, ElementImpl, ElementSize, ElementWorld, SizeConstraints, StateToParams,
    UiContext,
};
use std::cell::RefCell;
use std::fmt::Debug;
use std::rc::Rc;
use vn_scene::{BoxPrimitiveData, Color, Rect, Scene, Transform};

#[derive(Debug)]
pub struct ScrollAreaLayout {
    pub max_scroll_x: f32,
    pub max_scroll_y: f32,
}

pub trait ScrollAreaCallbacks: Debug {
    fn layout_changed(&mut self, layout: ScrollAreaLayout);
    fn scroll_x(&self) -> f32;
    fn scroll_y(&self) -> f32;
}

#[derive(Debug)]
pub struct SimpleScrollAreaCallbacks {
    pub layout: ScrollAreaLayout,
    pub scroll_x: f32,
    pub scroll_y: f32,
}

impl SimpleScrollAreaCallbacks {
    pub fn new() -> Self {
        Self {
            layout: ScrollAreaLayout {
                max_scroll_x: 0.0,
                max_scroll_y: 0.0,
            },
            scroll_y: 0.0,
            scroll_x: 0.0,
        }
    }
}

impl ScrollAreaCallbacks for SimpleScrollAreaCallbacks {
    fn layout_changed(&mut self, layout: ScrollAreaLayout) {
        self.layout = layout;
    }

    fn scroll_x(&self) -> f32 {
        self.scroll_x.clamp(0.0, self.layout.max_scroll_x)
    }

    fn scroll_y(&self) -> f32 {
        self.scroll_y.clamp(0.0, self.layout.max_scroll_y)
    }
}

#[derive(Clone, Debug)]
pub struct ScrollAreaParams {
    pub scroll_x: Option<f32>,
    pub scroll_y: Option<f32>,
    pub controller: Option<Rc<RefCell<dyn ScrollAreaCallbacks>>>,
    pub scrollbar_width: f32,
    pub scrollbar_margin: f32,
}

pub struct ScrollArea<State> {
    id: ElementId,
    child: Box<dyn Element<State = State>>,
    params: StateToParams<State, ScrollAreaParams>,
    child_size: ElementSize,
}

impl<State> ScrollArea<State> {
    pub fn new(
        child: Box<dyn Element<State = State>>,
        params: StateToParams<State, ScrollAreaParams>,
        world: &mut ElementWorld,
    ) -> Self {
        Self {
            id: world.next_id(),
            child,
            params,
            child_size: ElementSize::ZERO,
        }
    }
}

impl<State> ElementImpl for ScrollArea<State> {
    type State = State;

    fn id_impl(&self) -> ElementId {
        self.id
    }

    fn layout_impl(
        &mut self,
        ctx: &mut UiContext,
        state: &Self::State,
        constraints: SizeConstraints,
    ) -> ElementSize {
        let params = (self.params)(crate::StateToParamsArgs {
            state,
            id: self.id,
            ctx,
        });

        let mut child_constraints = constraints;
        let mut grow_by = ElementSize::ZERO;
        if params.scroll_x.is_some() {
            child_constraints.max_size.width = None;
            grow_by.height = params.scrollbar_width + params.scrollbar_margin;
        }
        if params.scroll_y.is_some() {
            child_constraints.max_size.height = None;
            grow_by.width = params.scrollbar_width + params.scrollbar_margin;
        }

        let child_size = self
            .child
            .layout(ctx, state, child_constraints.shrink_by(grow_by));

        self.child_size = child_size;

        child_size
            .grow_by(grow_by)
            .clamp_to_constraints(constraints)
    }

    fn draw_impl(
        &mut self,
        ctx: &mut UiContext,
        state: &Self::State,
        origin: (f32, f32),
        size: ElementSize,
        scene: &mut dyn Scene,
    ) {
        let params = (self.params)(crate::StateToParamsArgs {
            state,
            id: self.id,
            ctx,
        });

        let child_origin = (
            origin.0
                - params
                    .scroll_x
                    .unwrap_or(0.0)
                    .min((self.child_size.width - size.width).max(0.0)),
            origin.1
                - params
                    .scroll_y
                    .unwrap_or(0.0)
                    .min((self.child_size.height - size.height).max(0.0)),
        );

        let clip_rect = Rect {
            position: [origin.0, origin.1],
            size: [size.width, size.height],
        };

        if let Some(controller) = params.controller {
            controller.borrow_mut().layout_changed(ScrollAreaLayout {
                max_scroll_x: (self.child_size.width - size.width).max(0.0),
                max_scroll_y: (self.child_size.height - size.height).max(0.0),
            });
        }

        ctx.with_clipping(clip_rect, |ctx| {
            self.child
                .draw(ctx, state, child_origin, self.child_size, scene);
        });

        // Draw scroll bars
        if let Some(scroll_y) = params.scroll_y {
            if self.child_size.height > size.height {
                let scrollbar_height = (size.height / self.child_size.height) * size.height;
                let scrollbar_y = (scroll_y / self.child_size.height) * size.height;

                scene.add_box(BoxPrimitiveData {
                    transform: Transform {
                        translation: [
                            origin.0 + size.width - params.scrollbar_width,
                            origin.1 + scrollbar_y,
                        ],
                        ..Transform::DEFAULT
                    },
                    size: [params.scrollbar_width, scrollbar_height],
                    color: Color::WHITE.with_alpha(0.5),
                    border_color: Color::TRANSPARENT,
                    border_thickness: 0.0,
                    border_radius: params.scrollbar_width / 2.0,
                    clip_rect: Rect::NO_CLIP,
                });
            }
        }

        if let Some(scroll_x) = params.scroll_x {
            if self.child_size.width > size.width {
                let scrollbar_width = (size.width / self.child_size.width) * size.width;
                let scrollbar_x = (scroll_x / self.child_size.width) * size.width;

                scene.add_box(BoxPrimitiveData {
                    transform: Transform {
                        translation: [
                            origin.0 + scrollbar_x,
                            origin.1 + size.height - params.scrollbar_width,
                        ],
                        ..Transform::DEFAULT
                    },
                    size: [scrollbar_width, params.scrollbar_width],
                    color: Color::WHITE.with_alpha(0.5),
                    border_color: Color::TRANSPARENT,
                    border_thickness: 0.0,
                    border_radius: params.scrollbar_width / 2.0,
                    clip_rect: Rect::NO_CLIP,
                });
            }
        }
    }
}

pub trait ScrollAreaExt: Element {
    fn scroll_area(
        self,
        params: StateToParams<Self::State, ScrollAreaParams>,
        world: &mut ElementWorld,
    ) -> ScrollArea<Self::State>;
}

impl<E: Element + 'static> ScrollAreaExt for E {
    fn scroll_area(
        self,
        params: StateToParams<Self::State, ScrollAreaParams>,
        world: &mut ElementWorld,
    ) -> ScrollArea<Self::State> {
        ScrollArea::new(Box::new(self), params, world)
    }
}
