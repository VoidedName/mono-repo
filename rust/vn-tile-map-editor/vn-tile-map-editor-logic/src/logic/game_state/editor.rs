use crate::logic::game_state::editor_ui::{editor, layers, tileset};
use crate::logic::game_state::{ApplicationStateEx, TryLoadTileSetResult, label};
use crate::logic::{ApplicationContext, ApplicationEvent, EditorCallback};
use crate::{UI_FONT, UI_FONT_SIZE};
use std::cell::RefCell;
use std::collections::{HashMap};
use std::rc::Rc;
use vn_scene::{Color, TextureId};
use vn_tilemap::{TileMapLayerMapSpecification, TileMapLayerSpecification, TileMapSpecification};
use vn_ui::{
    AnchorExt, AnchorLocation, AnchorParams, Element, ElementWorld, EventManager, Flex, FlexChild,
    FlexDirection, FlexParams, PaddingExt, PaddingParams, ScrollBarParams, params,
};

pub mod editor_ui;

#[derive(Debug)]
pub struct EditorState {
    loaded_tilesets: HashMap<String, TextureId>,
    current_layer: Option<usize>,
    tile_map: TileMapSpecification,
    tileset_view_scroll_x: ScrollBarParams,
    tileset_view_scroll_y: ScrollBarParams,
    layer_list_scroll_x: ScrollBarParams,
    layer_list_scroll_y: ScrollBarParams,
    loaded_tileset_list_view_scroll_x: ScrollBarParams,
    loaded_tileset_list_view_scroll_y: ScrollBarParams,
}

#[derive(Debug, Clone)]
pub enum EditorEvent {
    TilesetViewScrollX(f32),
    TilesetViewScrollY(f32),
    LayerListScrollX(f32),
    LayerListScrollY(f32),
    LoadedTilesetListScrollX(f32),
    LoadedTilesetListScrollY(f32),
    TryAddingLayer,
    LoadSpec,
    SaveSpec,
    AddLayer(TryLoadTileSetResult),
}

pub struct Editor {
    #[allow(unused)]
    ctx: ApplicationContext,
    ui: RefCell<Box<dyn Element<State = EditorState, Message = EditorEvent>>>,
    state: EditorState,
    event_manager: Rc<RefCell<EventManager>>,
}

impl Editor {
    pub async fn new(ctx: ApplicationContext) -> anyhow::Result<Self> {
        let world = Rc::new(RefCell::new(ElementWorld::new()));

        let title = label(
            |_| "Tile Map Editor".to_string(),
            UI_FONT,
            UI_FONT_SIZE,
            Color::WHITE,
            ctx.text_metrics.clone(),
            world.clone(),
        )
        .padding(params!(PaddingParams::vertical(25.0)), world.clone())
        .anchor(
            params!(AnchorParams {
                location: AnchorLocation::Top
            }),
            world.clone(),
        );

        let layers = layers(&ctx, world.clone());
        let editor = editor(&ctx, world.clone());
        let tileset = tileset(&ctx, world.clone());

        let ui = Flex::new(
            {
                let children = vec![
                    FlexChild::new(title).into_rc_refcell(),
                    FlexChild::weighted(
                        Flex::new(
                            {
                                let children = vec![
                                    FlexChild::new(layers).into_rc_refcell(),
                                    FlexChild::weighted(editor, 1.0).into_rc_refcell(),
                                    FlexChild::new(tileset).into_rc_refcell(),
                                ];
                                params!(FlexParams {
                                    direction: FlexDirection::Row,
                                    force_orthogonal_same_size: true,
                                    children: children.clone(),
                                })
                            },
                            world.clone(),
                        ),
                        1.0,
                    )
                    .into_rc_refcell(),
                ];
                params!(FlexParams {
                    direction: FlexDirection::Column,
                    children: children.clone(),
                    force_orthogonal_same_size: true,
                })
            },
            world.clone(),
        );

        let scroll_bar = ScrollBarParams {
            width: 16.0,
            color: Color::WHITE,
            position: Some(0.0),
            margin: 8.0,
        };

        Ok(Self {
            ctx,
            ui: RefCell::new(Box::new(ui)),
            state: EditorState {
                current_layer: None,
                loaded_tilesets: HashMap::new(),
                tile_map: TileMapSpecification {
                    layers: vec![],
                    map_dimensions: (10, 5),
                },
                tileset_view_scroll_x: scroll_bar,
                tileset_view_scroll_y: scroll_bar,
                layer_list_scroll_x: scroll_bar,
                layer_list_scroll_y: scroll_bar,
                loaded_tileset_list_view_scroll_x: scroll_bar,
                loaded_tileset_list_view_scroll_y: scroll_bar,
            },
            event_manager: Rc::new(RefCell::new(EventManager::new())),
        })
    }
}

impl ApplicationStateEx for Editor {
    type StateEvent = EditorEvent;
    type State = EditorState;
    type ApplicationEvent = ApplicationEvent;

    fn ui(&self) -> &RefCell<Box<dyn Element<State = Self::State, Message = Self::StateEvent>>> {
        &self.ui
    }

    fn state(&self) -> &Self::State {
        &self.state
    }

    fn event_manager(&self) -> Rc<RefCell<EventManager>> {
        self.event_manager.clone()
    }

    fn handle_event(&mut self, event: Self::StateEvent) -> Option<Self::ApplicationEvent> {
        log::info!("handling state event: {:?}", event);

        match event {
            EditorEvent::TilesetViewScrollX(v) => {
                self.state.tileset_view_scroll_x.position = Some(v)
            }
            EditorEvent::TilesetViewScrollY(v) => {
                self.state.tileset_view_scroll_y.position = Some(v)
            }
            EditorEvent::LayerListScrollX(v) => self.state.layer_list_scroll_x.position = Some(v),
            EditorEvent::LayerListScrollY(v) => self.state.layer_list_scroll_y.position = Some(v),
            EditorEvent::LoadedTilesetListScrollX(v) => {
                self.state.loaded_tileset_list_view_scroll_x.position = Some(v)
            }
            EditorEvent::LoadedTilesetListScrollY(v) => {
                self.state.loaded_tileset_list_view_scroll_y.position = Some(v)
            }
            EditorEvent::TryAddingLayer => {
                return Some(ApplicationEvent::NewLayer(
                    self.state.loaded_tilesets.keys().cloned().collect(),
                    EditorCallback {
                        call: Box::new(|editor, tiles| match tiles {
                            Some(tiles) => {
                                editor.handle_event(EditorEvent::AddLayer(tiles));
                            }
                            None => {}
                        }),
                    },
                ));
            }
            EditorEvent::AddLayer(tileset) => {
                match tileset {
                    TryLoadTileSetResult::Loaded(tileset) => {
                        let cols = tileset.texture_dimensions.0 / tileset.tile_dimensions.0;
                        let rows = tileset.texture_dimensions.1 / tileset.tile_dimensions.1;

                        self.state.tile_map.layers.push(TileMapLayerSpecification {
                            name: "Unnamed Layer".to_string(),
                            tileset: tileset.name.clone(),
                            tile_dimensions: (tileset.tile_dimensions.0, tileset.tile_dimensions.1),
                            map: TileMapLayerMapSpecification { tiles: vec![] },
                            tileset_dimensions: (cols, rows),
                        });

                        self.state
                            .loaded_tilesets
                            .insert(tileset.name, tileset.texture_id.clone());
                    }
                    TryLoadTileSetResult::Reuse(tileset) => {
                        let settings = self
                            .state
                            .tile_map
                            .layers
                            .iter()
                            .find(|t| t.tileset == tileset)
                            .unwrap();

                        self.state.tile_map.layers.push(TileMapLayerSpecification {
                            name: "Unnamed Layer".to_string(),
                            tileset: settings.tileset.clone(),
                            tile_dimensions: settings.tile_dimensions.clone(),
                            map: TileMapLayerMapSpecification { tiles: vec![] },
                            tileset_dimensions: settings.tileset_dimensions.clone(),
                        });
                    }
                }
                self.state.current_layer = Some(self.state.tile_map.layers.len() - 1);
            }
            EditorEvent::LoadSpec => {}
            EditorEvent::SaveSpec => {}
        }

        None
    }
}
