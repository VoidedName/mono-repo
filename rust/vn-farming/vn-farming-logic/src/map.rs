use serde::{Deserialize, Serialize};
use vn_scene::{Color, ImagePrimitiveData, Rect, Scene, TextureId, Transform};
use vn_ui::{ElementId, ElementImpl, ElementSize, ElementWorld, SizeConstraints, StateToParams, StateToParamsArgs, UiContext};

pub struct TileMap {
    pub texture_id: TextureId,
    /// index is tile id, i.e. tile_locations\[3] = tile for id 3
    pub tile_locations: Vec<Rect>,
}

// think about how to place the camera (and zoom?), "center on" or "rectangle" or "top left" etc?
pub struct MapParams {
    pub tile_map: TileMap,
    pub tile_size: f32,
    pub map: Vec<Vec<usize>>,
}

pub struct Map<State: 'static> {
    id: ElementId,
    params: StateToParams<State, MapParams>,
}

impl<State> Map<State> {
    pub fn new(params: StateToParams<State, MapParams>, world: Rc<RefCell<ElementWorld>>) -> Self {
        Self {
            id: world.borrow_mut().next_id(),
            params,
        }
    }
}

impl<State> ElementImpl for Map<State> {
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
        let params = (self.params)(StateToParamsArgs {
            state,
            id: self.id,
            ctx,
        });

        match params.map.first() {
            Some(row) => ElementSize {
                width: row.len() as f32 * params.tile_size,
                height: params.map.len() as f32 * params.tile_size,
            },
            None => ElementSize::ZERO,
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

        params.map.iter().enumerate().for_each(|(y, row)| {
            row.iter().enumerate().for_each(|(x, tile)| {
                let tile_origin = [
                    origin.0 + x as f32 * params.tile_size,
                    origin.1 + y as f32 * params.tile_size,
                ];

                scene.add_image(ImagePrimitiveData {
                    transform: Transform::builder().translation(tile_origin).build(),
                    size: [params.tile_size, params.tile_size],
                    tint: Color::WHITE,
                    texture_id: params.tile_map.texture_id.clone(),
                    clip_rect: Rect {
                        position: [origin.0, origin.1],
                        size: [size.width, size.height],
                    },
                    uv_rect: params.tile_map.tile_locations[*tile],
                })
            })
        })
    }
}
