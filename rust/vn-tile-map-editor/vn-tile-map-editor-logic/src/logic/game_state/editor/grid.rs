use vn_scene::{BoxPrimitiveData, Color, Rect, Scene, Transform};
use vn_ui::{Element, ElementId, ElementImpl, ElementSize, ElementWorld, ScrollAreaParams, SizeConstraints, StateToParams, StateToParamsArgs, UiContext};
use crate::logic::game_state::editor::Editor;

pub struct GridParams {
    pub rows: u32,
    pub cols: u32,
    pub grid_size: (f32, f32),
}

pub struct Grid<State> {
    id: ElementId,
    params: StateToParams<State, GridParams>
}

impl<State> Grid<State> {

    pub fn new(
        params: StateToParams<State, GridParams>,
        world: &mut ElementWorld,
    ) -> Self {
        Self {
            id: world.next_id(),
            params
        }
    }
}

impl<State> ElementImpl for Grid<State> {
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

        ElementSize {
            width: params.grid_size.0 * params.cols as f32,
            height: params.grid_size.1 * params.rows as f32,
        }.clamp_to_constraints(constraints)
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
            let px = origin.0 + y as f32 * params.grid_size.0;
            scene.add_box(BoxPrimitiveData {
                transform: Transform::builder().translation([px, origin.1]).build(),
                size: [1.0, size.height.min(params.grid_size.1 * params.rows as f32)],
                color: Color::WHITE.with_alpha(0.2),
                border_radius: 0.0,
                border_color: Color::TRANSPARENT,
                border_thickness: 0.0,
                clip_rect: Rect::NO_CLIP,
            });
        }

        for x in 0..=params.rows {
            let px = origin.1 + x as f32 * params.grid_size.1;
            scene.add_box(BoxPrimitiveData {
                transform: Transform::builder().translation([origin.0, px]).build(),
                size: [size.width.min(params.grid_size.0 * params.cols as f32), 1.0],
                color: Color::WHITE.with_alpha(0.2),
                border_radius: 0.0,
                border_color: Color::TRANSPARENT,
                border_thickness: 0.0,
                clip_rect: Rect::NO_CLIP,
            });
        }
    }
}

pub struct TilesetGrid {
    id: ElementId,
}

impl TilesetGrid {
    pub fn new(world: &mut vn_ui::ElementWorld) -> Self {
        Self {
            id: world.next_id(),
        }
    }
}

impl ElementImpl for TilesetGrid {
    type State = Editor;

    fn id_impl(&self) -> ElementId {
        self.id
    }

    fn layout_impl(
        &mut self,
        _ctx: &mut UiContext,
        _state: &Self::State,
        constraints: SizeConstraints,
    ) -> ElementSize {
        // TilesetGrid should probably match the size of its container (the Stack it's in)
        // Or we can try to derive it from the selected layer's tileset dimensions.
        // For simplicity in Stack, we can return a zero size or the max available.
        ElementSize {
            width: constraints.max_size.width.unwrap_or(0.0),
            height: constraints.max_size.height.unwrap_or(0.0),
        }
    }

    fn draw_impl(
        &mut self,
        ctx: &mut UiContext,
        state: &Self::State,
        origin: (f32, f32),
        size: ElementSize,
        scene: &mut dyn Scene,
    ) {
        if let Some(layer) = state.map_spec.layers.get(state.selected_layer_index) {
            let (tile_w, tile_h) = (layer.tile_dimensions.0 as f32, layer.tile_dimensions.1 as f32);
            let (ts_w_tiles, ts_h_tiles) = layer.tile_set_dimensions;

            if tile_w <= 0.0 || tile_h <= 0.0 || ts_w_tiles == 0 || ts_h_tiles == 0 {
                return;
            }

            // Calculate scale if the rendered size doesn't match the pixel size
            let actual_w = ts_w_tiles as f32 * tile_w;
            let actual_h = ts_h_tiles as f32 * tile_h;

            if actual_w <= 0.0 || actual_h <= 0.0 {
                return;
            }

            let scale_x = size.width / actual_w;
            let scale_y = size.height / actual_h;

            let scaled_tile_w = tile_w * scale_x;
            let scaled_tile_h = tile_h * scale_y;

            let clip_rect = ctx.clip_rect;

            // Draw vertical lines
            for x in 0..=ts_w_tiles {
                let px = origin.0 + x as f32 * scaled_tile_w;
                if px > origin.0 + size.width + 0.1 {
                    break;
                }
                scene.add_box(BoxPrimitiveData {
                    transform: Transform::builder().translation([px, origin.1]).build(),
                    size: [1.0, size.height],
                    color: Color::WHITE.with_alpha(0.3),
                    border_radius: 0.0,
                    border_color: Color::TRANSPARENT,
                    border_thickness: 0.0,
                    clip_rect,
                });
            }

            // Draw horizontal lines
            for y in 0..=ts_h_tiles {
                let py = origin.1 + y as f32 * scaled_tile_h;
                if py > origin.1 + size.height + 0.1 {
                    break;
                }
                scene.add_box(BoxPrimitiveData {
                    transform: Transform::builder().translation([origin.0, py]).build(),
                    size: [size.width, 1.0],
                    color: Color::WHITE.with_alpha(0.3),
                    border_radius: 0.0,
                    border_color: Color::TRANSPARENT,
                    border_thickness: 0.0,
                    clip_rect,
                });
            }
        }
    }
}
