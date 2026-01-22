use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TileMapSpecification {
    /// Drawing grid size in pixels.
    pub grid_dimensions: (f32, f32),
    /// Dimensions of the map in tiles
    pub map_dimensions: (u32, u32),
    pub layers: Vec<TileMapLayerSpecification>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum TilePositioning {
    Center,
    TopLeft,
}

/// Different layers may not have the same tile dimensions.
/// This means that we need to decide how to fit the tile into the grid cell.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum TileFitStrategy {
    Stretch,
    /// Layers do not have to have the same aspect ratio as the global grid.
    PreserveAspect(TilePositioning),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TileMapLayerMapSpecification {
    /// index is tile id, i.e. tiles[3] = tile for id 3
    ///
    /// this must conform to map_dimensions (or be at least as large)
    pub tiles: Vec<Vec<Option<usize>>>,
}

/// This assumes that the tile_set image is not padded in any way, i.e. the pixel dimensions are a multiple
/// of the tile dimensions
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TileMapLayerSpecification {
    /// Path to the tile_set image?
    pub tile_set: String,
    /// Dimensions of the tile set image in tiles
    pub tile_set_dimensions: (u32, u32),
    /// Dimensions of a single tile inside the texture in pixels
    pub tile_dimensions: (u32, u32),
    /// How to fit the tile into the grid cell
    pub fit_strategy: TileFitStrategy,
    /// The actual map
    pub map: TileMapLayerMapSpecification,
}
