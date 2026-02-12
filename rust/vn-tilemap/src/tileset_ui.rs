use crate::TileMapSpecification;
use std::cell::RefCell;
use std::marker::PhantomData;
use std::rc::Rc;
use vn_scene::{
    Color, ConstructableScene, GenericScene, ImagePrimitiveData, Rect, Scene, TextureId, Transform,
};
use vn_ui_definitions::{
    ElementId, ElementImpl, ElementSize, ElementWorld, InteractionEvent, SizeConstraints,
    StateToParams, StateToParamsArgs, UiContext, into_box_impl,
};

#[derive(Clone)]
pub struct TileMapParams {
    pub textures: Vec<TextureId>,
    pub specification: TileMapSpecification,
    pub draw_tile_size: ElementSize,
    pub chunk_size: (u32, u32),
}

struct Chunk {
    texture: Option<TextureId>,
    dirty: bool,
    last_state: Vec<Vec<Option<usize>>>,
}

impl Chunk {
    fn new() -> Self {
        Self {
            texture: None,
            dirty: true,
            last_state: Vec::new(),
        }
    }
}

// pre-bake tilemaps -> use chunks when exceeding 4096x4096 pixels
pub struct TileMap<State: 'static, Message> {
    id: ElementId,
    params: StateToParams<State, TileMapParams>,
    chunks: Vec<Vec<Vec<Chunk>>>, // [layer][chunk_y][chunk_x]
    last_specification: Option<TileMapSpecification>,
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
            chunks: Vec::new(),
            last_specification: None,
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

        // performance opt, only do this for chunks on screen and store partial specs with the chunks
        if let Some(last_spec) = &self.last_specification {
            if last_spec.map_dimensions != params.specification.map_dimensions
                || last_spec.layers.len() != params.specification.layers.len()
            {
                self.chunks.clear();
                self.last_specification = Some(params.specification.clone());
            } else {
                // Check if any layer other than the tile data changed (e.g. tileset path, tile dimensions)
                // If so, we still need to mark all chunks as dirty for that layer.
                for (layer_idx, (old_layer, new_layer)) in last_spec
                    .layers
                    .iter()
                    .zip(&params.specification.layers)
                    .enumerate()
                {
                    if old_layer.tileset != new_layer.tileset
                        || old_layer.tileset_dimensions != new_layer.tileset_dimensions
                        || old_layer.tile_dimensions != new_layer.tile_dimensions
                    {
                        if let Some(layer_chunks) = self.chunks.get_mut(layer_idx) {
                            for row in layer_chunks {
                                for chunk in row {
                                    chunk.dirty = true;
                                }
                            }
                        }
                    }
                }
                self.last_specification = Some(params.specification.clone());
            }
        } else {
            self.last_specification = Some(params.specification.clone());
            self.chunks.clear();
        }

        let map_w = params.specification.map_dimensions.0;
        let map_h = params.specification.map_dimensions.1;
        let chunk_w = params.chunk_size.0;
        let chunk_h = params.chunk_size.1;

        let num_chunks_x = (map_w + chunk_w - 1) / chunk_w;
        let num_chunks_y = (map_h + chunk_h - 1) / chunk_h;

        if self.chunks.is_empty() {
            self.chunks = (0..params.specification.layers.len())
                .map(|_| {
                    (0..num_chunks_y)
                        .map(|_| (0..num_chunks_x).map(|_| Chunk::new()).collect())
                        .collect()
                })
                .collect();
        }

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
                let start_chunk_x = ((ctx.clip_rect.position[0] - origin.0)
                    / (params.draw_tile_size.width * chunk_w as f32))
                    .floor()
                    .max(0.0) as u32;
                let start_chunk_y = ((ctx.clip_rect.position[1] - origin.1)
                    / (params.draw_tile_size.height * chunk_h as f32))
                    .floor()
                    .max(0.0) as u32;

                let end_chunk_x = ((ctx.clip_rect.position[0] + ctx.clip_rect.size[0] - origin.0)
                    / (params.draw_tile_size.width * chunk_w as f32))
                    .floor()
                    .max(start_chunk_x as f32)
                    .min(num_chunks_x as f32 - 1.0) as u32;
                let end_chunk_y = ((ctx.clip_rect.position[1] + ctx.clip_rect.size[1] - origin.1)
                    / (params.draw_tile_size.height * chunk_h as f32))
                    .floor()
                    .max(start_chunk_y as f32)
                    .min(num_chunks_y as f32 - 1.0) as u32;

                for (layer_idx, (layer, texture)) in specs.iter().enumerate() {
                    for cy in start_chunk_y..=end_chunk_y {
                        for cx in start_chunk_x..=end_chunk_x {
                            let chunk = &mut self.chunks[layer_idx][cy as usize][cx as usize];

                            let c_start_x = cx * chunk_w;
                            let c_start_y = cy * chunk_h;
                            let c_end_x = ((cx + 1) * chunk_w).min(map_w);
                            let c_end_y = ((cy + 1) * chunk_h).min(map_h);

                            let mut current_state =
                                Vec::with_capacity((c_end_y - c_start_y) as usize);
                            for y in c_start_y..c_end_y {
                                let mut row = Vec::with_capacity((c_end_x - c_start_x) as usize);
                                for x in c_start_x..c_end_x {
                                    row.push(
                                        layer
                                            .map
                                            .tiles
                                            .get(y as usize)
                                            .and_then(|row| row.get(x as usize).copied().flatten()),
                                    );
                                }
                                current_state.push(row);
                            }

                            if chunk.dirty
                                || chunk.texture.is_none()
                                || chunk.last_state != current_state
                            {
                                let mut sub_scene = GenericScene::new((
                                    (c_end_x - c_start_x) as f32 * params.draw_tile_size.width,
                                    (c_end_y - c_start_y) as f32 * params.draw_tile_size.height,
                                ));

                                for (y_offset, row) in current_state.iter().enumerate() {
                                    for (x_offset, tile_id) in row.iter().enumerate() {
                                        if let Some(tile_id) = tile_id {
                                            let uv_width = 1.0 / layer.tileset_dimensions.0 as f32;
                                            let uv_height = 1.0 / layer.tileset_dimensions.1 as f32;

                                            let uv_x = *tile_id as u32 % layer.tileset_dimensions.0;
                                            let uv_y = *tile_id as u32 / layer.tileset_dimensions.0;

                                            sub_scene.add_image(ImagePrimitiveData {
                                                transform: Transform {
                                                    translation: [
                                                        x_offset as f32
                                                            * params.draw_tile_size.width,
                                                        y_offset as f32
                                                            * params.draw_tile_size.height,
                                                    ],
                                                    ..Transform::DEFAULT
                                                },
                                                size: [
                                                    params.draw_tile_size.width,
                                                    params.draw_tile_size.height,
                                                ],
                                                tint: Color::WHITE,
                                                texture_id: (*texture).clone(),
                                                clip_rect: Rect::NO_CLIP,
                                                uv_rect: Rect {
                                                    position: [
                                                        uv_x as f32 * uv_width,
                                                        uv_y as f32 * uv_height,
                                                    ],
                                                    size: [uv_width, uv_height],
                                                },
                                            });
                                        }
                                    }
                                }

                                chunk.texture =
                                    Some(ctx.scene_renderer.borrow().render_to_texture(
                                        &sub_scene,
                                        sub_scene.scene_size(),
                                        chunk.texture.take(),
                                    ));
                                chunk.dirty = false;
                                chunk.last_state = current_state;
                            }

                            if let Some(texture_id) = &chunk.texture {
                                let c_start_x = cx * chunk_w;
                                let c_start_y = cy * chunk_h;
                                let c_end_x = ((cx + 1) * chunk_w).min(map_w);
                                let c_end_y = ((cy + 1) * chunk_h).min(map_h);

                                scene.add_image(ImagePrimitiveData {
                                    transform: Transform {
                                        translation: [
                                            c_start_x as f32 * params.draw_tile_size.width
                                                + origin.0,
                                            c_start_y as f32 * params.draw_tile_size.height
                                                + origin.1,
                                        ],
                                        ..Transform::DEFAULT
                                    },
                                    size: [
                                        (c_end_x - c_start_x) as f32 * params.draw_tile_size.width,
                                        (c_end_y - c_start_y) as f32 * params.draw_tile_size.height,
                                    ],
                                    tint: Color::WHITE,
                                    texture_id: texture_id.clone(),
                                    clip_rect: ctx.clip_rect,
                                    uv_rect: Rect::UNIT,
                                });
                            }
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

    fn invalidated_impl(&self, ctx: &UiContext, state: &Self::State) -> bool {
        let params = self.params.call(StateToParamsArgs {
            state,
            ctx,
            id: self.id,
        });

        if let Some(last_spec) = &self.last_specification {
            if last_spec.map_dimensions != params.specification.map_dimensions
                || last_spec.layers.len() != params.specification.layers.len()
            {
                return true;
            }

            for (old_layer, new_layer) in last_spec.layers.iter().zip(&params.specification.layers)
            {
                if old_layer.name != new_layer.name
                    || old_layer.tileset != new_layer.tileset
                    || old_layer.tileset_dimensions != new_layer.tileset_dimensions
                    || old_layer.tile_dimensions != new_layer.tile_dimensions
                    || old_layer.map.tiles != new_layer.map.tiles
                {
                    return true;
                }
            }
        } else {
            return true;
        }

        false
    }
}

into_box_impl!(TileMap);
