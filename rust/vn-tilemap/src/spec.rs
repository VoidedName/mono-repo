use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TileMapSpecification {
    /// Dimensions of the map in tiles
    pub map_dimensions: (u32, u32),
    pub layers: Vec<TileMapLayerSpecification>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TileMapLayerMapSpecification {
    /// index is tile id, i.e. tiles[3] = tile for id 3
    ///
    /// this must conform to map_dimensions (or be at least as large)
    pub tiles: Vec<Vec<Option<usize>>>,
}

/// This assumes that the tileset image is not padded in any way, i.e. the pixel dimensions are a multiple
/// of the tile dimensions
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TileMapLayerSpecification {
    pub name: String,
    /// Path to the tileset image
    pub tileset: String,
    /// Dimensions of the tileset image in tiles
    pub tileset_dimensions: (u32, u32),
    /// Dimensions of a single tile inside the texture in pixels
    pub tile_dimensions: (u32, u32),
    /// The actual map
    pub map: TileMapLayerMapSpecification,
}
