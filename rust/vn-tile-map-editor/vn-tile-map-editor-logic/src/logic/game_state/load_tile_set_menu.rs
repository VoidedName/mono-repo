use crate::logic::game_state::LoadTileMenuStateErrors::{
    TilesHeighIsZero, TilesHighMustDivideTexture, TilesWideIsZero, TilesWideMustDivideTexture,
    TilesetNameIsEmpty,
};
use crate::logic::game_state::editor::{Grid, GridParams};
use crate::logic::game_state::{
    ApplicationStateEx, Input, LoadedTileSet, TextFieldState, btn, empty_texture, input, label,
    labelled_input, suppress_enter_key,
};
use crate::logic::{ApplicationContext, ApplicationEvent};
use std::cell::RefCell;
use std::collections::HashSet;
use std::marker::PhantomData;
use std::rc::Rc;
use thiserror::Error;
use vn_scene::TextureId;
use vn_ui::*;

#[derive(Debug)]
pub struct LoadedTexture {
    pub suggested_name: String,
    pub id: TextureId,
    pub dimensions: (u32, u32),
}

#[derive(Debug)]
pub struct LoadTileMenuState {
    tileset_name_input_state: TextFieldState,
    loaded_texture: LoadedTexture,
    loaded_texture_scroll_x: ScrollBarParams,
    loaded_texture_scroll_y: ScrollBarParams,
    tiles_wide_input: TextFieldState,
    tiles_wide: u32,
    tiles_heigh_input: TextFieldState,
    tiles_high: u32,
    errors: HashSet<LoadTileMenuStateErrors>,
}

#[derive(Debug, Error, Hash, PartialEq, Eq)]
pub enum LoadTileMenuStateErrors {
    #[error("Tiles high must not be 0 or empty")]
    TilesHeighIsZero,
    #[error("Tiles heigh must divide textures width")]
    TilesHeighMustDivideTexture,
    #[error("Tiles wide must not be 0 or empty")]
    TilesWideIsZero,
    #[error("Tiles wide must divide textures width")]
    TilesWideMustDivideTexture,
    #[error("Tiles high must divide textures height")]
    TilesHighMustDivideTexture,
    #[error("Tileset name must not be empty")]
    TilesetNameIsEmpty,
}

#[derive(Clone, Debug)]
pub enum LoadTileSetMenuInputEvent {
    CaretMoved(usize),
    TextChanged(String),
}

#[derive(Clone, Debug)]
pub enum LoadTileSetMenuEvent {
    Save,
    Cancel,
    TileSetNameInputChanged(LoadTileSetMenuInputEvent),
    TileWideInputChanged(LoadTileSetMenuInputEvent),
    TilesWideChanged(u32),
    TileHeighInputChanged(LoadTileSetMenuInputEvent),
    TilesHighChanged(u32),
    TexturePreviewScrollX(f32),
    TexturePreviewScrollY(f32),
}

pub struct LoadTileSetMenu<ApplicationEvent> {
    #[allow(unused)]
    ctx: ApplicationContext,
    #[allow(unused)]
    ui: RefCell<Box<dyn Element<State = LoadTileMenuState, Message = LoadTileSetMenuEvent>>>,
    #[allow(unused)]
    state: LoadTileMenuState,
    #[allow(unused)]
    event_manager: Rc<RefCell<EventManager>>,
    #[allow(unused)]
    _phantom: PhantomData<ApplicationEvent>,
}

const UI_FONT: &str = "jetbrains-bold";
const UI_FONT_SIZE: f32 = 16.0;

impl<ApplicationEvent> LoadTileSetMenu<ApplicationEvent> {
    pub async fn new(
        ctx: ApplicationContext,
        loaded_texture: LoadedTexture,
    ) -> anyhow::Result<Self> {
        let world = &mut ElementWorld::new();
        let save = btn(
            "Save",
            UI_FONT,
            UI_FONT_SIZE,
            |state: &LoadTileMenuState| !state.errors.is_empty(),
            ctx.text_metrics.clone(),
            EventHandler::new(|_, _| vec![LoadTileSetMenuEvent::Save]),
            world,
        );
        let cancel = btn(
            "Cancel",
            UI_FONT,
            UI_FONT_SIZE,
            |_| false,
            ctx.text_metrics.clone(),
            EventHandler::new(|_, _| vec![LoadTileSetMenuEvent::Cancel]),
            world,
        );

        let actions = Flex::new_row_unweighted(
            vec![
                save,
                Box::new(
                    Empty::new(world).padding(params!(PaddingParams::horizontal(16.0)), world),
                ),
                cancel,
            ],
            true,
            world,
        )
        .anchor(
            params!(AnchorParams {
                location: AnchorLocation::RIGHT
            }),
            world,
        );

        let Input {
            id: tileset_name_input_id,
            element: tileset_name_input,
        } = input(
            |state: &LoadTileMenuState| state.tileset_name_input_state.clone(),
            Some("Tileset Name"),
            UI_FONT,
            UI_FONT_SIZE,
            ctx.text_metrics.clone(),
            EventHandler::new(|_, event| match event {
                TextFieldAction::TextChange(new_text) => {
                    vec![LoadTileSetMenuEvent::TileSetNameInputChanged(
                        LoadTileSetMenuInputEvent::TextChanged(new_text),
                    )]
                }
                TextFieldAction::CaretMove(position) => {
                    vec![LoadTileSetMenuEvent::TileSetNameInputChanged(
                        LoadTileSetMenuInputEvent::CaretMoved(position),
                    )]
                }
            })
            .with_overwrite(suppress_enter_key()),
            world,
        );

        // these could be dropboxes containing all divisors of the texture dimension instead
        let Input {
            id: tiles_wide_id,
            element: tiles_wide,
        } = labelled_input(
            |state: &LoadTileMenuState| state.tiles_wide_input.clone(),
            "Tiles Wide: ",
            UI_FONT,
            UI_FONT_SIZE,
            ctx.text_metrics.clone(),
            EventHandler::new(|_, event| match event {
                TextFieldAction::TextChange(new_text) => {
                    vec![LoadTileSetMenuEvent::TileWideInputChanged(
                        LoadTileSetMenuInputEvent::TextChanged(new_text),
                    )]
                }
                TextFieldAction::CaretMove(position) => {
                    vec![LoadTileSetMenuEvent::TileWideInputChanged(
                        LoadTileSetMenuInputEvent::CaretMoved(position),
                    )]
                }
            })
            .with_overwrite(suppress_enter_key()),
            world,
        );

        let Input {
            id: tiles_heigh_id,
            element: tiles_high,
        } = labelled_input(
            |state: &LoadTileMenuState| state.tiles_heigh_input.clone(),
            "Tiles High: ",
            UI_FONT,
            UI_FONT_SIZE,
            ctx.text_metrics.clone(),
            EventHandler::new(|_, event| match event {
                TextFieldAction::TextChange(new_text) => {
                    vec![LoadTileSetMenuEvent::TileHeighInputChanged(
                        LoadTileSetMenuInputEvent::TextChanged(new_text),
                    )]
                }
                TextFieldAction::CaretMove(position) => {
                    vec![LoadTileSetMenuEvent::TileHeighInputChanged(
                        LoadTileSetMenuInputEvent::CaretMoved(position),
                    )]
                }
            })
            .with_overwrite(suppress_enter_key()),
            world,
        );

        let error = label(
            |state: &LoadTileMenuState| {
                let mut messages: Vec<_> = state.errors.iter().map(|e| e.to_string()).collect();
                messages.sort();
                messages.join("\n")
            },
            UI_FONT,
            UI_FONT_SIZE,
            Color::RED,
            ctx.text_metrics.clone(),
            world,
        );

        let tex_description = label(
            |state: &LoadTileMenuState| {
                format!("Dimension:\n {:?}", state.loaded_texture.dimensions)
            },
            UI_FONT,
            UI_FONT_SIZE,
            Color::WHITE.with_alpha(0.5),
            ctx.text_metrics.clone(),
            world,
        );

        let grid = Grid::new(
            params!(args<LoadTileMenuState> => GridParams {
                cols: args.state.tiles_wide,
                rows: args.state.tiles_high,
                grid_size: (args.state.loaded_texture.dimensions.0 as f32 / args.state.tiles_wide as f32, args.state.loaded_texture.dimensions.1 as f32 / args.state.tiles_high as f32),
                grid_color: Color::WHITE,
                grid_width: 3.0,
            }),
            world,
        );

        // make this scrollable
        // put text with meta information below (specifically the dimensions)
        let texture = PreferSize::new(
            Box::new(ScrollArea::new(
                Box::new(Stack::new(
                    vec![
                        Box::new(Texture::new(
                            params!(args<LoadTileMenuState> =>
                                TextureParams {
                                    texture_id: args.state.loaded_texture.id.clone(),
                                    tint: Color::WHITE,
                                    fit_strategy: FitStrategy::Clip {rotation: 0.0},
                                    uv_rect: Rect {
                                        position: [0.0, 0.0],
                                        size: [1.0, 1.0],
                                    },
                                    preferred_size: ElementSize {
                                        width: args.state.loaded_texture.dimensions.0 as f32,
                                        height: args.state.loaded_texture.dimensions.1 as f32,
                                    }
                            }),
                            world,
                        )),
                        Box::new(grid),
                    ],
                    world,
                )),
                params!(args<LoadTileMenuState> =>
                    ScrollAreaParams {
                        scroll_x: args.state.loaded_texture_scroll_x.clone(),
                        scroll_y: args.state.loaded_texture_scroll_y.clone(),
                        scroll_action_handler: EventHandler::new(|_, e| {
                                match e {
                                    ScrollAreaAction::ScrollX(v) => vec![LoadTileSetMenuEvent::TexturePreviewScrollX(v)],
                                    ScrollAreaAction::ScrollY(v) => vec![LoadTileSetMenuEvent::TexturePreviewScrollY(v)],
                                }
                            }),
                    }
                ),
                world,
            )),
            params!(PreferSizeParams {
                size: ElementSize {
                    width: 256.0 + 24.0,
                    height: 256.0 + 24.0,
                }
            }),
            world,
        );

        let title = Padding::new(
            label(
                |_| "Configure Tileset".to_string(),
                UI_FONT,
                UI_FONT_SIZE,
                Color::WHITE,
                ctx.text_metrics.clone(),
                world,
            ),
            params!(PaddingParams {
                pad_bottom: 25.0,
                ..Default::default()
            }),
            world,
        )
        .anchor(
            params!(AnchorParams {
                location: AnchorLocation::TOP
            }),
            world,
        );

        let ui = PreferSize::new(
            Box::new(Flex::new_column(
                vec![
                    FlexChild::new(Box::new(title)),
                    FlexChild::weighted(
                        Box::new(Flex::new_row(
                            vec![
                                FlexChild::weighted(
                                    Box::new(Flex::new_column(
                                        vec![
                                            FlexChild::new(tileset_name_input),
                                            FlexChild::new(tiles_wide),
                                            FlexChild::new(tiles_high),
                                        ],
                                        true,
                                        world,
                                    )),
                                    1.0,
                                ),
                                FlexChild::weighted(
                                    Box::new(Flex::new_column_unweighted(
                                        vec![Box::new(texture), tex_description],
                                        true,
                                        world,
                                    )),
                                    1.0,
                                ),
                            ],
                            true,
                            world,
                        )),
                        1.0,
                    ),
                    FlexChild::weighted(Box::new(Empty::new(world)), 0.0),
                    FlexChild::new(error),
                    FlexChild::new(Box::new(actions)),
                ],
                true,
                world,
            )),
            params!(PreferSizeParams {
                size: ElementSize {
                    width: 256.0 * 2.0 + 50.0,
                    height: 256.0 * 2.0,
                }
            }),
            world,
        )
        .padding(params!(PaddingParams::uniform(25.0)), world)
        .card(
            params!(CardParams {
                background_color: Color::BLACK,
                border_size: 2.0,
                corner_radius: 5.0,
                border_color: Color::WHITE,
            }),
            world,
        )
        .anchor(
            params!(AnchorParams {
                location: AnchorLocation::CENTER
            }),
            world,
        );

        let mut errors = HashSet::new();
        if loaded_texture.suggested_name.trim().is_empty() {
            errors.insert(TilesetNameIsEmpty);
        }
        errors.insert(TilesHeighIsZero);
        errors.insert(TilesWideIsZero);

        Ok(Self {
            ctx,
            ui: RefCell::new(Box::new(ui)),
            state: LoadTileMenuState {
                tileset_name_input_state: TextFieldState {
                    id: tileset_name_input_id,
                    text: loaded_texture.suggested_name.clone(),
                    caret: None,
                },
                loaded_texture,
                tiles_high: 1,
                tiles_heigh_input: TextFieldState {
                    id: tiles_heigh_id,
                    text: "".to_string(),
                    caret: None,
                },
                tiles_wide: 1,
                tiles_wide_input: TextFieldState {
                    id: tiles_wide_id,
                    text: "".to_string(),
                    caret: None,
                },
                loaded_texture_scroll_x: ScrollBarParams {
                    position: Some(0.0),
                    width: 16.0,
                    margin: 8.0,
                    color: Color::WHITE,
                },
                loaded_texture_scroll_y: ScrollBarParams {
                    position: Some(0.0),
                    width: 16.0,
                    margin: 8.0,
                    color: Color::WHITE,
                },
                errors,
            },
            event_manager: Rc::new(RefCell::new(EventManager::new())),
            _phantom: PhantomData,
        })
    }
}

impl ApplicationStateEx for LoadTileSetMenu<ApplicationEvent> {
    type StateEvent = LoadTileSetMenuEvent;
    type State = LoadTileMenuState;
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
        log::info!("handling event: {:?}", event);
        match event {
            LoadTileSetMenuEvent::TileSetNameInputChanged(event) => match event {
                LoadTileSetMenuInputEvent::CaretMoved(mut position) => {
                    if self.state.tileset_name_input_state.text.is_empty() {
                        position = 0
                    }
                    self.state.tileset_name_input_state.caret = Some(position);
                }
                LoadTileSetMenuInputEvent::TextChanged(new_text) => {
                    if new_text.trim().is_empty() {
                        self.state.errors.insert(TilesetNameIsEmpty);
                    } else {
                        self.state.errors.remove(&TilesetNameIsEmpty);
                    }
                    self.state.tileset_name_input_state.text = new_text;
                }
            },
            LoadTileSetMenuEvent::TileWideInputChanged(event) => match event {
                LoadTileSetMenuInputEvent::CaretMoved(mut position) => {
                    if self.state.tiles_wide_input.text.is_empty() {
                        position = 0
                    }
                    self.state.tiles_wide_input.caret =
                        Some(position.min(self.state.tiles_wide_input.text.chars().count()));
                }
                LoadTileSetMenuInputEvent::TextChanged(new_text) => {
                    let new_text = new_text.trim().to_string();
                    if new_text.is_empty() {
                        self.state.tiles_wide = 0;
                        self.state.tiles_wide_input.text = new_text;
                        self.handle_event(LoadTileSetMenuEvent::TilesWideChanged(0));
                    } else {
                        let wide = new_text.parse::<u32>();
                        match wide {
                            Ok(wide) => {
                                self.state.tiles_wide = wide;
                                self.state.tiles_wide_input.text = new_text;
                                self.handle_event(LoadTileSetMenuEvent::TilesWideChanged(wide));
                            }
                            Err(_) => {}
                        }
                    }
                }
            },
            LoadTileSetMenuEvent::TileHeighInputChanged(event) => match event {
                LoadTileSetMenuInputEvent::CaretMoved(mut position) => {
                    if self.state.tiles_heigh_input.text.is_empty() {
                        position = 0
                    }
                    self.state.tiles_heigh_input.caret =
                        Some(position.min(self.state.tiles_heigh_input.text.chars().count()));
                }
                LoadTileSetMenuInputEvent::TextChanged(new_text) => {
                    let new_text = new_text.trim().to_string();
                    if new_text.is_empty() {
                        self.state.tiles_high = 0;
                        self.state.tiles_heigh_input.text = new_text;
                        self.handle_event(LoadTileSetMenuEvent::TilesHighChanged(0));
                    } else {
                        let heigh = new_text.parse::<u32>();
                        match heigh {
                            Ok(heigh) => {
                                self.state.tiles_high = heigh;
                                self.state.tiles_heigh_input.text = new_text;
                                self.handle_event(LoadTileSetMenuEvent::TilesHighChanged(heigh));
                            }
                            Err(_) => {}
                        }
                    }
                }
            },
            LoadTileSetMenuEvent::TilesWideChanged(wide) => {
                if wide == 0 {
                    self.state.errors.insert(TilesWideIsZero);
                } else {
                    self.state.errors.remove(&TilesWideIsZero);
                    if self.state.loaded_texture.dimensions.0.is_multiple_of(wide) {
                        self.state.errors.remove(&TilesWideMustDivideTexture);
                    } else {
                        self.state.errors.insert(TilesWideMustDivideTexture);
                    }
                }
            }
            LoadTileSetMenuEvent::TilesHighChanged(high) => {
                if high == 0 {
                    self.state.errors.insert(TilesHeighIsZero);
                } else {
                    self.state.errors.remove(&TilesHeighIsZero);
                    if self.state.loaded_texture.dimensions.1.is_multiple_of(high) {
                        self.state.errors.remove(&TilesHighMustDivideTexture);
                    } else {
                        self.state.errors.insert(TilesHighMustDivideTexture);
                    }
                }
            }
            LoadTileSetMenuEvent::TexturePreviewScrollX(v) => {
                self.state.loaded_texture_scroll_x.position = Some(v);
            }
            LoadTileSetMenuEvent::TexturePreviewScrollY(v) => {
                self.state.loaded_texture_scroll_y.position = Some(v);
            }

            LoadTileSetMenuEvent::Save => {
                return Some(ApplicationEvent::TileSetLoaded(LoadedTileSet {
                    texture_id: self.state.loaded_texture.id.clone(),
                    name: self.state.tileset_name_input_state.text.clone(),
                    texture_dimensions: self.state.loaded_texture.dimensions,
                    tile_dimensions: (
                        self.state.loaded_texture.dimensions.0 / self.state.tiles_wide,
                        self.state.loaded_texture.dimensions.1 / self.state.tiles_high,
                    ),
                }));
            }
            LoadTileSetMenuEvent::Cancel => return Some(ApplicationEvent::TileSetLoadCanceled),
        }

        None
    }
}
