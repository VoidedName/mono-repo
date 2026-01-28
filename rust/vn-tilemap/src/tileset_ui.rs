use crate::TileMapSpecification;
use std::cell::RefCell;
use std::marker::PhantomData;
use std::rc::Rc;
use vn_scene::{Color, ImagePrimitiveData, Rect, Scene, TextureId, Transform};
use vn_ui::{
    ElementId, ElementImpl, ElementSize, ElementWorld, InteractionEvent, SizeConstraints,
    StateToParams, StateToParamsArgs, UiContext, into_box_impl,
};

#[derive(Clone)]
pub struct TileMapParams {
    pub textures: Vec<TextureId>,
    pub specification: TileMapSpecification,
    pub draw_tile_size: ElementSize,
}

pub struct TileMap<State: 'static, Message> {
    id: ElementId,
    params: StateToParams<State, TileMapParams>,
    _phantom: PhantomData<Message>,
}

impl<State, Message> TileMap<State, Message> {
    pub fn new<P: Into<StateToParams<State, TileMapParams>>>(
        params: P,
        world: Rc<RefCell<ElementWorld>>,
    ) -> Self {
        Self {
            id: world.borrow_mut().next_id(),
            params: params.into(),
            _phantom: PhantomData,
        }
    }
}

impl<State, Message> ElementImpl for TileMap<State, Message> {
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
        let params = self.params.call(StateToParamsArgs {
            state,
            ctx,
            id: self.id,
        });

        let width = params.specification.map_dimensions.0 as f32 * params.draw_tile_size.width;
        let height = params.specification.map_dimensions.1 as f32 * params.draw_tile_size.height;

        ElementSize { width, height }.clamp_to_constraints(constraints)
    }

    fn draw_impl(
        &mut self,
        ctx: &mut UiContext,
        state: &Self::State,
        origin: (f32, f32),
        size: ElementSize,
        scene: &mut dyn Scene,
    ) {
        let params = self.params.call(StateToParamsArgs {
            state,
            ctx,
            id: self.id,
        });

        let specs = &params
            .specification
            .layers
            .iter()
            .zip(&params.textures)
            .collect::<Vec<_>>();

        ctx.with_clipping(
            Rect {
                position: [origin.0, origin.1],
                size: [size.width, size.height],
            },
            |ctx| {
                for x in 0..params.specification.map_dimensions.0 {
                    for y in 0..params.specification.map_dimensions.1 {
                        for (layer, texture) in specs {
                            let tile_id = layer
                                .map
                                .tiles
                                .get(y as usize)
                                .map(|row| row.get(x as usize).unwrap_or(&None))
                                .unwrap_or(&None);

                            let uv_width = 1.0 / layer.tileset_dimensions.0 as f32;
                            let uv_height = 1.0 / layer.tileset_dimensions.1 as f32;

                            if let Some(tile_id) = tile_id {
                                let uv_x = *tile_id as u32 / layer.tileset_dimensions.1;
                                let uv_y = *tile_id as u32 % layer.tileset_dimensions.0;

                                scene.add_image(ImagePrimitiveData {
                                    transform: Transform {
                                        translation: [
                                            x as f32 * params.draw_tile_size.width + origin.0,
                                            y as f32 * params.draw_tile_size.height + origin.1,
                                        ],
                                        ..Transform::DEFAULT
                                    },
                                    size: [
                                        params.draw_tile_size.width,
                                        params.draw_tile_size.height,
                                    ],
                                    tint: Color::WHITE,
                                    texture_id: (*texture).clone(),
                                    clip_rect: ctx.clip_rect,
                                    uv_rect: Rect {
                                        position: [uv_x as f32 * uv_width, uv_y as f32 * uv_height],
                                        size: [uv_width, uv_height],
                                    },
                                })
                            };
                        }
                    }
                }
            },
        );
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

into_box_impl!(TileMap);
