use crate::TileMapSpecification;
use std::marker::PhantomData;
use serde::Deserialize;
use vn_scene::{Color, ImagePrimitiveData, Rect, Scene, TextureId, Transform};
use vn_ui::{
    ElementId, ElementImpl, ElementSize, ElementWorld, InteractionEvent, SizeConstraints,
    StateToParams, StateToParamsArgs, UiContext,
};

#[derive(Clone)]
pub struct TileMapParams {
    pub texture: Vec<TextureId>,
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
        world: &mut ElementWorld,
    ) -> Self {
        Self {
            id: world.next_id(),
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

        for (layer, tex) in params.specification.layers.iter().zip(&params.texture) {
            let tile_width_in_tex =
                layer.tile_dimensions.0 as f32 / layer.tile_set_dimensions.0 as f32;
            let tile_height_in_tex =
                layer.tile_dimensions.1 as f32 / layer.tile_set_dimensions.1 as f32;

            for (y, tiles) in layer.map.tiles.iter().enumerate() {
                for (x, tile) in tiles.iter().enumerate() {
                    if let Some(tile) = tile {
                        let x_in_tex = *tile as u32 % layer.tile_set_dimensions.1;
                        let y_in_tex = *tile as u32 / layer.tile_set_dimensions.0;

                        ctx.with_clipping(
                            Rect {
                                position: [origin.0, origin.1],
                                size: [size.width, size.height],
                            },
                            |ctx| {
                                scene.add_image(ImagePrimitiveData {
                                    transform: Transform {
                                        translation: [
                                            x as f32 * params.draw_tile_size.width,
                                            y as f32 * params.draw_tile_size.height,
                                        ],
                                        ..Transform::DEFAULT
                                    },
                                    size: [
                                        params.draw_tile_size.width,
                                        params.draw_tile_size.height,
                                    ],
                                    tint: Color::WHITE,
                                    texture_id: tex.clone(),
                                    clip_rect: ctx.clip_rect,
                                    uv_rect: Rect {
                                        position: [
                                            x_in_tex as f32 * tile_width_in_tex,
                                            y_in_tex as f32 * tile_height_in_tex,
                                        ],
                                        size: [tile_width_in_tex, tile_height_in_tex],
                                    },
                                })
                            },
                        )
                    }
                }
            }
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
