use vn_scene::{BoxPrimitiveData, Color, Scene, Transform};
use vn_ui::{
    ElementId, ElementImpl, ElementSize, ElementWorld, InteractionEvent, SizeConstraints,
    StateToParams, StateToParamsArgs, UiContext,
};

pub struct GridParams {
    pub rows: u32,
    pub cols: u32,
    pub grid_size: (f32, f32),
    pub grid_color: Color,
    pub grid_width: f32,
}

pub struct Grid<State, Message> {
    id: ElementId,
    params: StateToParams<State, GridParams>,
    _phantom: std::marker::PhantomData<Message>,
}

impl<State, Message> Grid<State, Message> {
    pub fn new(params: StateToParams<State, GridParams>, world: &mut ElementWorld) -> Self {
        Self {
            id: world.next_id(),
            params,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<State, Message> ElementImpl for Grid<State, Message> {
    type State = State;
    type Message = Message;

    fn id_impl(&self) -> ElementId {
        self.id
    }

    fn layout_impl(
        &mut self,
        ctx: &mut UiContext,
        state: &Self::State,
        constraints: SizeConstraints,
    ) -> ElementSize {
        let params = (self.params)(StateToParamsArgs {
            state,
            id: self.id,
            ctx,
        });

        ElementSize {
            width: params.grid_size.0 * params.cols as f32,
            height: params.grid_size.1 * params.rows as f32,
        }
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
        let params = (self.params)(StateToParamsArgs {
            state,
            id: self.id,
            ctx,
        });

        for y in 0..=params.cols {
            let px = origin.0 + y as f32 * params.grid_size.0 - params.grid_width / 2.0;
            scene.add_box(BoxPrimitiveData {
                transform: Transform::builder().translation([px, origin.1]).build(),
                size: [
                    params.grid_width,
                    size.height.min(params.grid_size.1 * params.rows as f32),
                ],
                color: params.grid_color,
                border_radius: 0.0,
                border_color: Color::TRANSPARENT,
                border_thickness: 0.0,
                clip_rect: ctx.clip_rect,
            });
        }

        for x in 0..=params.rows {
            let px = origin.1 + x as f32 * params.grid_size.1 - params.grid_width / 2.0;
            scene.add_box(BoxPrimitiveData {
                transform: Transform::builder().translation([origin.0, px]).build(),
                size: [
                    size.width.min(params.grid_size.0 * params.cols as f32),
                    params.grid_width,
                ],
                color: params.grid_color,
                border_radius: 0.0,
                border_color: Color::TRANSPARENT,
                border_thickness: 0.0,
                clip_rect: ctx.clip_rect,
            });
        }
    }

    fn handle_event_impl(
        &mut self,
        _ctx: &mut UiContext,
        _state: &Self::State,
        _event: &InteractionEvent,
    ) -> Vec<Self::Message> {
        vec![]
    }
}
