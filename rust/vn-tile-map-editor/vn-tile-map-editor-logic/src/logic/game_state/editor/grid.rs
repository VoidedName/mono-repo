use vn_scene::{BoxPrimitiveData, Color, Rect, Scene, Transform};
use vn_ui::{ElementId, ElementImpl, ElementSize, SizeConstraints, UiContext};
use crate::logic::game_state::editor::Editor;

pub struct Grid {
    id: ElementId,
}

impl Grid {
    pub fn new(world: &mut vn_ui::ElementWorld) -> Self {
        Self {
            id: world.next_id(),
        }
    }
}

impl ElementImpl for Grid {
    type State = Editor;

    fn id_impl(&self) -> ElementId {
        self.id
    }

    fn layout_impl(
        &mut self,
        _ctx: &mut UiContext,
        state: &Self::State,
        constraints: SizeConstraints,
    ) -> ElementSize {
        ElementSize {
            width: state.map_spec.map_dimensions.0 as f32 * state.map_spec.grid_dimensions.0 + 1.0,
            height: state.map_spec.map_dimensions.1 as f32 * state.map_spec.grid_dimensions.1 + 1.0,
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
        ctx.with_hitbox_hierarchy(
            self.id,
            scene.current_layer_id(),
            Rect {
                position: [origin.0, origin.1],
                size: [size.width, size.height],
            },
            |ctx| {
                let clip_rect = ctx.clip_rect;
                let (grid_w, grid_h) = state.map_spec.grid_dimensions;
                let (map_w, map_h) = state.map_spec.map_dimensions;

                // Draw tiles
                for (_layer_index, layer) in state.map_spec.layers.iter().enumerate() {
                    // Only draw up to selected layer or all? Usually all.
                    for (_y, row) in layer.map.tiles.iter().enumerate() {
                        for (_x, tile_id) in row.iter().enumerate() {
                            if let Some(_id) = tile_id {
                                // TODO: Render actual tile image when textures are loaded
                            }
                        }
                    }
                }

                // Draw grid lines
                for x in 0..=map_w {
                    let px = origin.0 + x as f32 * grid_w;
                    if px > origin.0 + size.width {
                        break;
                    }
                    scene.add_box(BoxPrimitiveData {
                        transform: Transform::builder().translation([px, origin.1]).build(),
                        size: [1.0, size.height],
                        color: Color::WHITE.with_alpha(0.2),
                        border_radius: 0.0,
                        border_color: Color::TRANSPARENT,
                        border_thickness: 0.0,
                        clip_rect,
                    });
                }

                for y in 0..=map_h {
                    let py = origin.1 + y as f32 * grid_h;
                    if py > origin.1 + size.height {
                        break;
                    }
                    scene.add_box(BoxPrimitiveData {
                        transform: Transform::builder().translation([origin.0, py]).build(),
                        size: [size.width, 1.0],
                        color: Color::WHITE.with_alpha(0.2),
                        border_radius: 0.0,
                        border_color: Color::TRANSPARENT,
                        border_thickness: 0.0,
                        clip_rect,
                    });
                }
            },
        );
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
