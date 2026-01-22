#[derive(Clone, Debug)]
pub enum EditorEvent {
    AddLayer,
    RemoveLayer(usize),
    SelectLayer(usize),
    SaveMap,
    LoadMap,
    OpenSettings,
    ChangeMapDimensions(u32, u32),
    ChangeTileDimensions(u32, u32),
    ChangeTileSetDimensions(u32, u32),
    SelectTileset(String),
    LoadTilesetFromInput,
    ScrollTileset(f32),
}
