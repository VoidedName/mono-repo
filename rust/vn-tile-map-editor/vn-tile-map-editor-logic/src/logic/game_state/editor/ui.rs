use crate::logic::game_state::editor::grid::GridParams;
use crate::logic::game_state::editor::{Editor, EditorEvent, Grid};
use std::rc::Rc;
use vn_scene::{Color, Rect, TextureId};
use vn_ui::{Anchor, AnchorExt, AnchorLocation, AnchorParams, ButtonExt, ButtonParams, CardExt, CardParams, Element, ElementId, ElementSize, ElementWorld, EventHandler, FitStrategy, Flex, FlexChild, InteractionEventKind, InteractionState, InteractiveExt, PaddingExt, PaddingParams, ScrollAreaExt, ScrollAreaParams, Stack, StateToParamsArgs, TextField, TextFieldAction, TextFieldParams, TextMetrics, TextVisuals, Texture, TextureParams};
use winit::event::{ElementState, KeyEvent};
use winit::keyboard;
use winit::keyboard::NamedKey;
use vn_wgpu_window::resource_manager::ResourceManager;

pub const UI_FONT: &str = "jetbrains-bold";
pub const UI_FONT_SIZE: f32 = 16.0;

pub struct EditorUi {
    pub root: Box<dyn Element<State = Editor, Message = EditorEvent>>,
    pub tileset_path_input_id: ElementId,
    pub tile_width_input_id: ElementId,
    pub tile_height_input_id: ElementId,
    pub tileset_cols_input_id: ElementId,
    pub tileset_rows_input_id: ElementId,
    pub tileset_preview_scroll_area_id: ElementId,
}

pub fn build_editor_ui(
    editor: &Editor,
    world: &mut ElementWorld,
    rm: Rc<ResourceManager>,
    metrics: Rc<dyn TextMetrics>,
) -> EditorUi {
    let title = build_title(world, metrics.clone());
    let grid = build_grid(world);
    let sidebar_info = build_sidebar(editor, world, metrics.clone());
    let (preview, tileset_preview_scroll_area_id) = build_tileset_preview_panel(editor, world);
    let fps_counter = build_fps_counter(metrics.clone(), world);

    let main_layout = Flex::new_row(
        vec![
            FlexChild::new(sidebar_info.sidebar),
            FlexChild::weighted(
                Box::new(Anchor::new(
                    grid,
                    Box::new(|_| AnchorParams {
                        location: AnchorLocation::CENTER,
                    }),
                    world,
                )),
                1.0,
            ),
            FlexChild::new(preview),
        ],
        false,
        world,
    )
    .padding(Box::new(|_| PaddingParams::uniform(40.0)), world);

    let text_atlas = Texture::new(Box::new(|_| {
        TextureParams {
            texture_id: TextureId(Rc::new(1)),
            preferred_size: ElementSize {
                height: 256.0,
                width: 256.0,
            },
            uv_rect: Rect {
                position: [0.0, 0.0],
                size: [1.0, 1.0],
            },
            tint: Color::WHITE.with_alpha(0.5),
            fit_strategy: FitStrategy::PreserveAspectRatio { rotation: 0.0 },
        }
    }), world);

    let ui = Flex::new_column(
        vec![
            FlexChild::new(title),
            FlexChild::weighted(Box::new(main_layout), 1.0),
        ],
        false,
        world,
    );

    let ui = Stack::new(
        vec![
            Box::new(ui),
            Box::new(Anchor::new(
                fps_counter,
                Box::new(|_| AnchorParams {
                    location: AnchorLocation::TopRight,
                }),
                world,
            )),
            Box::new(text_atlas),
            // Box::new(text_atlas_letter),
        ],
        world,
    );

    EditorUi {
        root: Box::new(ui),
        tileset_path_input_id: sidebar_info.tileset_path_input_id,
        tile_width_input_id: sidebar_info.tile_width_input_id,
        tile_height_input_id: sidebar_info.tile_height_input_id,
        tileset_cols_input_id: sidebar_info.tileset_cols_input_id,
        tileset_rows_input_id: sidebar_info.tileset_rows_input_id,
        tileset_preview_scroll_area_id,
    }
}

fn build_title(
    world: &mut ElementWorld,
    metrics: Rc<dyn TextMetrics>,
) -> Box<dyn Element<State = Editor, Message = EditorEvent>> {
    Box::new(
        TextField::new(
            Box::new({
                let metrics = metrics.clone();
                move |_: StateToParamsArgs<'_, Editor>| TextFieldParams {
                    visuals: TextVisuals {
                        text: "Tile Map Editor".to_string(),
                        caret_position: None,
                        font: UI_FONT.to_string(),
                        font_size: UI_FONT_SIZE,
                        color: Color::WHITE,
                        caret_width: None,
                        caret_blink_duration: None,
                    },
                    metrics: metrics.clone(),
                    interaction: Default::default(),
                    text_field_action_handler: EventHandler::none(),
                }
            }),
            world,
        )
        .padding(Box::new(|_| PaddingParams::uniform(10.0)), world)
        .anchor(
            Box::new(|_| AnchorParams {
                location: AnchorLocation::TOP,
            }),
            world,
        ),
    )
}

fn build_grid(world: &mut ElementWorld) -> Box<dyn Element<State = Editor, Message = EditorEvent>> {
    Box::new(
        Grid::new(
            Box::new(|_| GridParams {
                grid_size: (32.0, 32.0),
                cols: 10,
                rows: 10,
                grid_width: 3.0,
                grid_color: Color::WHITE.with_alpha(0.5),
            }),
            world,
        )
        .padding(Box::new(|_| PaddingParams::uniform(10.0)), world)
        .card(
            Box::new(|_| CardParams {
                background_color: Color::BLACK.with_alpha(0.3),
                border_size: 2.0,
                border_color: Color::WHITE.with_alpha(0.5),
                corner_radius: 5.0,
            }),
            world,
        ),
    )
}

pub struct SidebarInfo {
    pub sidebar: Box<dyn Element<State = Editor, Message = EditorEvent>>,
    pub tileset_path_input_id: ElementId,
    pub tile_width_input_id: ElementId,
    pub tile_height_input_id: ElementId,
    pub tileset_cols_input_id: ElementId,
    pub tileset_rows_input_id: ElementId,
}

pub struct TilesetViewInfo {
    pub element: Box<dyn Element<State = Editor, Message = EditorEvent>>,
    pub tileset_path_input_id: ElementId,
    pub tile_width_input_id: ElementId,
    pub tile_height_input_id: ElementId,
    pub tileset_cols_input_id: ElementId,
    pub tileset_rows_input_id: ElementId,
}

fn build_sidebar(
    editor: &Editor,
    world: &mut ElementWorld,
    metrics: Rc<dyn TextMetrics>,
) -> SidebarInfo {
    let sidebar_title = Box::new(TextField::new(
        Box::new({
            let metrics = metrics.clone();
            move |_: StateToParamsArgs<'_, Editor>| TextFieldParams {
                visuals: TextVisuals {
                    text: "Layers".to_string(),
                    caret_position: None,
                    font: UI_FONT.to_string(),
                    font_size: UI_FONT_SIZE,
                    color: Color::WHITE,
                    caret_width: None,
                    caret_blink_duration: None,
                },
                metrics: metrics.clone(),
                interaction: Default::default(),
                text_field_action_handler: EventHandler::none(),
            }
        }),
        world,
    ));

    let layer_list = build_layer_list(editor, world, metrics.clone());
    let add_layer_button = build_add_layer_button(editor, world, metrics.clone());
    let tileset_title = build_tileset_title(world, metrics.clone());
    let tileset_view_info = build_tileset_view(editor, world, metrics.clone());
    let selection_info = build_selection_info(editor, world, metrics.clone());
    let footer = build_footer(editor, world, metrics.clone());

    let sidebar = Box::new(
        Flex::new_column_unweighted(
            vec![
                sidebar_title,
                layer_list,
                add_layer_button,
                tileset_title,
                tileset_view_info.element,
                selection_info,
                footer,
            ],
            true,
            world,
        )
        .padding(Box::new(|_| PaddingParams::uniform(10.0)), world)
        .card(
            Box::new(|_| CardParams {
                background_color: Color::BLACK.with_alpha(0.5),
                border_size: 2.0,
                border_color: Color::WHITE.with_alpha(0.5),
                corner_radius: 5.0,
            }),
            world,
        ),
    );

    SidebarInfo {
        sidebar,
        tileset_path_input_id: tileset_view_info.tileset_path_input_id,
        tile_width_input_id: tileset_view_info.tile_width_input_id,
        tile_height_input_id: tileset_view_info.tile_height_input_id,
        tileset_cols_input_id: tileset_view_info.tileset_cols_input_id,
        tileset_rows_input_id: tileset_view_info.tileset_rows_input_id,
    }
}

fn build_layer_list(
    editor: &Editor,
    world: &mut ElementWorld,
    metrics: Rc<dyn TextMetrics>,
) -> Box<dyn Element<State = Editor, Message = EditorEvent>> {
    let mut layer_elements: Vec<Box<dyn Element<State = Editor, Message = EditorEvent>>> =
        Vec::new();
    for (i, _layer) in editor.map_spec.layers.iter().enumerate() {
        let is_selected = i == editor.selected_layer_index;

        let layer_label = TextField::new(
            Box::new({
                let metrics = metrics.clone();
                move |_: StateToParamsArgs<'_, Editor>| TextFieldParams {
                    visuals: TextVisuals {
                        text: format!("Layer {}", i),
                        caret_position: None,
                        font: UI_FONT.to_string(),
                        font_size: UI_FONT_SIZE,
                        color: if is_selected {
                            Color::RED
                        } else {
                            Color::WHITE
                        },
                        caret_width: None,
                        caret_blink_duration: None,
                    },
                    metrics: metrics.clone(),
                    interaction: Default::default(),
                    text_field_action_handler: EventHandler::none(),
                }
            }),
            world,
        )
        .interactive_set(false, world);

        let remove_button = TextField::new(
            Box::new({
                let metrics = metrics.clone();
                move |_: StateToParamsArgs<'_, Editor>| TextFieldParams {
                    visuals: TextVisuals {
                        text: "X".to_string(),
                        caret_position: None,
                        font: UI_FONT.to_string(),
                        font_size: UI_FONT_SIZE,
                        color: Color::RED,
                        caret_width: None,
                        caret_blink_duration: None,
                    },
                    metrics: metrics.clone(),
                    interaction: Default::default(),
                    text_field_action_handler: EventHandler::none(),
                }
            }),
            world,
        )
        .interactive_set(false, world)
        .padding(Box::new(|_| PaddingParams::uniform(2.0)), world)
        .button(
            Box::new({
                let i = i;
                move |args| ButtonParams {
                    background: Color::BLACK.with_alpha(0.3),
                    border_color: if args.ctx.event_manager.borrow().is_hovered(args.id) {
                        Color::RED
                    } else {
                        Color::TRANSPARENT
                    },
                    border_width: 2.0,
                    corner_radius: 2.0,
                    interaction: vn_ui::InteractionState {
                        is_hovered: args.ctx.event_manager.borrow().is_hovered(args.id),
                        is_focused: false,
                    },
                    on_click: Some(EditorEvent::RemoveLayer(i)),
                }
            }),
            world,
        );

        let layer_row = Flex::new_row_unweighted(
            vec![Box::new(layer_label), Box::new(remove_button)],
            false,
            world,
        )
        .padding(Box::new(|_| PaddingParams::uniform(5.0)), world)
        .button(
            Box::new({
                let i = i;
                move |args| ButtonParams {
                    background: if is_selected {
                        Color::WHITE.with_alpha(0.2)
                    } else {
                        Color::WHITE.with_alpha(0.1)
                    },
                    border_color: if args.ctx.event_manager.borrow().is_hovered(args.id) {
                        Color::WHITE
                    } else {
                        Color::WHITE.with_alpha(0.3)
                    },
                    border_width: 2.0,
                    corner_radius: 3.0,
                    interaction: vn_ui::InteractionState {
                        is_hovered: args.ctx.event_manager.borrow().is_hovered(args.id),
                        is_focused: false,
                    },
                    on_click: Some(EditorEvent::SelectLayer(i)),
                }
            }),
            world,
        );

        layer_elements.push(Box::new(layer_row));
    }

    Box::new(Flex::new_column_unweighted(layer_elements, false, world))
}

fn build_add_layer_button(
    editor: &Editor,
    world: &mut ElementWorld,
    metrics: Rc<dyn TextMetrics>,
) -> Box<dyn Element<State = Editor, Message = EditorEvent>> {
    let button = TextField::new(
        Box::new({
            let metrics = metrics.clone();
            move |_: StateToParamsArgs<'_, Editor>| TextFieldParams {
                visuals: TextVisuals {
                    text: "Add Layer".to_string(),
                    caret_position: None,
                    font: UI_FONT.to_string(),
                    font_size: UI_FONT_SIZE,
                    color: Color::WHITE,
                    caret_width: None,
                    caret_blink_duration: None,
                },
                metrics: metrics.clone(),
                interaction: Default::default(),
                text_field_action_handler: EventHandler::none(),
            }
        }),
        world,
    )
    .interactive_set(false, world)
    .padding(Box::new(|_| PaddingParams::uniform(5.0)), world)
    .button(
        Box::new(|args| ButtonParams {
            background: Color::WHITE.with_alpha(0.1),
            border_color: if args.ctx.event_manager.borrow().is_hovered(args.id) {
                Color::WHITE
            } else {
                Color::WHITE.with_alpha(0.3)
            },
            border_width: 2.0,
            corner_radius: 3.0,
            interaction: InteractionState {
                is_hovered: args.ctx.event_manager.borrow().is_hovered(args.id),
                is_focused: false,
            },
            on_click: Some(EditorEvent::AddLayer),
        }),
        world,
    );

    Box::new(button)
}

fn build_tileset_title(
    world: &mut ElementWorld,
    metrics: Rc<dyn TextMetrics>,
) -> Box<dyn Element<State = Editor, Message = EditorEvent>> {
    Box::new(TextField::new(
        Box::new({
            let metrics = metrics.clone();
            move |_: StateToParamsArgs<'_, Editor>| TextFieldParams {
                visuals: TextVisuals {
                    text: "Tileset".to_string(),
                    caret_position: None,
                    font: UI_FONT.to_string(),
                    font_size: UI_FONT_SIZE,
                    color: Color::WHITE,
                    caret_width: None,
                    caret_blink_duration: None,
                },
                metrics: metrics.clone(),
                interaction: Default::default(),
                text_field_action_handler: EventHandler::none(),
            }
        }),
        world,
    ))
}

fn build_dimension_input(
    world: &mut ElementWorld,
    metrics: Rc<dyn TextMetrics>,
    label: String,
    text: fn(&Editor) -> String,
    caret: fn(&Editor) -> usize,
    on_action: Option<fn(ElementId, TextFieldAction) -> EditorEvent>,
) -> (
    Box<dyn Element<State = Editor, Message = EditorEvent>>,
    ElementId,
) {
    let label_el = TextField::new(
        Box::new({
            let metrics = metrics.clone();
            move |_: StateToParamsArgs<'_, Editor>| TextFieldParams {
                visuals: TextVisuals {
                    text: label.clone(),
                    caret_position: None,
                    font: UI_FONT.to_string(),
                    font_size: UI_FONT_SIZE,
                    color: Color::WHITE.with_alpha(0.7),
                    caret_width: None,
                    caret_blink_duration: None,
                },
                metrics: metrics.clone(),
                interaction: Default::default(),
                text_field_action_handler: EventHandler::none(),
            }
        }),
        world,
    )
    .padding(Box::new(|_| PaddingParams::uniform(5.0)), world);

    let input = TextField::new(
        Box::new({
            let metrics = metrics.clone();
            let text = text.clone();
            move |args: StateToParamsArgs<'_, Editor>| {
                let is_focused = args.ctx.event_manager.borrow().is_focused(args.id);
                TextFieldParams {
                    visuals: TextVisuals {
                        text: text(args.state),
                        caret_position: if is_focused {
                            Some(caret(args.state))
                        } else {
                            None
                        },
                        font: UI_FONT.to_string(),
                        font_size: UI_FONT_SIZE,
                        color: Color::WHITE,
                        caret_width: Some(2.0),
                        caret_blink_duration: Some(0.5),
                    },
                    metrics: metrics.clone(),
                    interaction: InteractionState {
                        is_hovered: args.ctx.event_manager.borrow().is_hovered(args.id),
                        is_focused,
                    },
                    text_field_action_handler: on_action.map_or(EventHandler::none(), |f| {
                        EventHandler::new(f).with_overwrite(|a, b| match b.kind {
                            InteractionEventKind::Keyboard(KeyEvent {
                                state: ElementState::Pressed,
                                logical_key: keyboard::Key::Named(NamedKey::Enter),
                                ..
                            }) => {
                                return (vec![], false);
                            }
                            _ => (vec![], true),
                        })
                    }),
                }
            }
        }),
        world,
    );

    let input_id = input.id();

    let input = input
        .padding(Box::new(|_| PaddingParams::uniform(5.0)), world)
        .interactive_set(true, world)
        .card(
            Box::new(|_| CardParams {
                background_color: Color::BLACK.with_alpha(0.5),
                border_size: 2.0,
                border_color: Color::WHITE.with_alpha(0.3),
                corner_radius: 3.0,
            }),
            world,
        );

    (
        Box::new(Flex::new_row_unweighted(
            vec![Box::new(label_el), Box::new(input)],
            false,
            world,
        )),
        input_id,
    )
}

fn build_tileset_view(
    editor: &Editor,
    world: &mut ElementWorld,
    metrics: Rc<dyn TextMetrics>,
) -> TilesetViewInfo {
    let mut tileset_elements: Vec<Box<dyn Element<State = Editor, Message = EditorEvent>>> =
        Vec::new();
    let current_tileset = editor
        .map_spec
        .layers
        .get(editor.selected_layer_index)
        .map(|l| l.tile_set.clone())
        .unwrap_or_else(|| "none".to_string());

    let current_ts_label = TextField::new(
        Box::new({
            let metrics = metrics.clone();
            let current_tileset = current_tileset.clone();
            move |_: StateToParamsArgs<'_, Editor>| TextFieldParams {
                visuals: TextVisuals {
                    text: format!("Current: {}", current_tileset),
                    caret_position: None,
                    font: UI_FONT.to_string(),
                    font_size: UI_FONT_SIZE,
                    color: Color::WHITE.with_alpha(0.7),
                    caret_width: None,
                    caret_blink_duration: None,
                },
                metrics: metrics.clone(),
                interaction: Default::default(),
                text_field_action_handler: EventHandler::none(),
            }
        }),
        world,
    )
    .padding(Box::new(|_| PaddingParams::uniform(5.0)), world);
    tileset_elements.push(Box::new(current_ts_label));

    let tileset_input = TextField::new(
        Box::new({
            let metrics = metrics.clone();
            move |args: StateToParamsArgs<'_, Editor>| {
                let is_focused = args.ctx.event_manager.borrow().is_focused(args.id);
                TextFieldParams {
                    visuals: TextVisuals {
                        text: args.state.tileset_path.clone(),
                        caret_position: if is_focused {
                            Some(args.state.tileset_path_caret)
                        } else {
                            None
                        },
                        font: UI_FONT.to_string(),
                        font_size: UI_FONT_SIZE,
                        color: Color::WHITE,
                        caret_width: Some(2.0),
                        caret_blink_duration: Some(0.5),
                    },
                    metrics: metrics.clone(),
                    interaction: InteractionState {
                        is_hovered: args.ctx.event_manager.borrow().is_hovered(args.id),
                        is_focused,
                    },
                    text_field_action_handler: EventHandler::new(|id, action| {
                        EditorEvent::TextFieldAction { id, action }
                    }),
                }
            }
        }),
        world,
    );

    let path_input_id = tileset_input.id();

    let tileset_input = tileset_input
        .padding(Box::new(|_| PaddingParams::uniform(5.0)), world)
        .interactive_set(true, world)
        .card(
            Box::new(|_| CardParams {
                background_color: Color::BLACK.with_alpha(0.5),
                border_size: 2.0,
                border_color: Color::WHITE.with_alpha(0.3),
                corner_radius: 3.0,
            }),
            world,
        );
    tileset_elements.push(Box::new(tileset_input));

    let load_button = TextField::new(
        Box::new({
            let metrics = metrics.clone();
            move |_: StateToParamsArgs<'_, Editor>| TextFieldParams {
                visuals: TextVisuals {
                    text: "Load Tileset".to_string(),
                    caret_position: None,
                    font: UI_FONT.to_string(),
                    font_size: UI_FONT_SIZE,
                    color: Color::WHITE,
                    caret_width: None,
                    caret_blink_duration: None,
                },
                metrics: metrics.clone(),
                interaction: Default::default(),
                text_field_action_handler: EventHandler::none(),
            }
        }),
        world,
    )
    .interactive_set(false, world)
    .padding(Box::new(|_| PaddingParams::uniform(5.0)), world)
    .button(
        Box::new(|args| ButtonParams {
            background: Color::WHITE.with_alpha(0.1),
            border_color: if args.ctx.event_manager.borrow().is_hovered(args.id) {
                Color::WHITE
            } else {
                Color::WHITE.with_alpha(0.3)
            },
            border_width: 2.0,
            corner_radius: 3.0,
            interaction: vn_ui::InteractionState {
                is_hovered: args.ctx.event_manager.borrow().is_hovered(args.id),
                is_focused: false,
            },
            on_click: Some(EditorEvent::LoadTilesetFromInput),
        }),
        world,
    );

    tileset_elements.push(Box::new(load_button));

    let (tw_input, tw_id) = build_dimension_input(
        world,
        metrics.clone(),
        "Tile W:".to_string(),
        |editor| editor.tile_width_text.clone(),
        |editor| editor.tile_width_caret,
        Some(|id, action| EditorEvent::TextFieldAction { id, action }),
    );
    let (th_input, th_id) = build_dimension_input(
        world,
        metrics.clone(),
        "Tile H:".to_string(),
        |editor| editor.tile_height_text.clone(),
        |editor| editor.tile_height_caret,
        Some(|id, action| EditorEvent::TextFieldAction { id, action }),
    );
    tileset_elements.push(Box::new(Flex::new_row_unweighted(
        vec![tw_input, th_input],
        false,
        world,
    )));

    // Tileset Dimensions (in tiles)
    let (tsw_input, tsw_id) = build_dimension_input(
        world,
        metrics.clone(),
        "Set Cols:".to_string(),
        |editor| editor.tileset_cols_text.clone(),
        |editor| editor.tileset_cols_caret,
        Some(|id, action| EditorEvent::TextFieldAction { id, action }),
    );
    let (tsh_input, tsh_id) = build_dimension_input(
        world,
        metrics.clone(),
        "Set Rows:".to_string(),
        |editor| editor.tileset_rows_text.clone(),
        |editor| editor.tileset_rows_caret,
        Some(|id, action| EditorEvent::TextFieldAction { id, action }),
    );
    tileset_elements.push(Box::new(Flex::new_row_unweighted(
        vec![tsw_input, tsh_input],
        false,
        world,
    )));

    let mut recently_loaded_elements: Vec<Box<dyn Element<State = Editor, Message = EditorEvent>>> =
        Vec::new();
    for (path, _) in &editor.loaded_tilesets {
        let path = path.clone();
        let is_selected = path == current_tileset;
        let ts_button = TextField::new(
            Box::new({
                let metrics = metrics.clone();
                let path = path.clone();
                move |_: StateToParamsArgs<'_, Editor>| TextFieldParams {
                    visuals: TextVisuals {
                        text: path.clone(),
                        caret_position: None,
                        font: UI_FONT.to_string(),
                        font_size: UI_FONT_SIZE,
                        color: if is_selected {
                            Color::RED
                        } else {
                            Color::WHITE
                        },
                        caret_width: None,
                        caret_blink_duration: None,
                    },
                    metrics: metrics.clone(),
                    interaction: Default::default(),
                    text_field_action_handler: EventHandler::none(),
                }
            }),
            world,
        )
        .interactive_set(false, world)
        .padding(Box::new(|_| PaddingParams::uniform(3.0)), world)
        .button(
            Box::new({
                let path = path.clone();
                move |args| ButtonParams {
                    background: if is_selected {
                        Color::WHITE.with_alpha(0.2)
                    } else {
                        Color::WHITE.with_alpha(0.1)
                    },
                    border_color: if args.ctx.event_manager.borrow().is_hovered(args.id) {
                        Color::WHITE
                    } else {
                        Color::WHITE.with_alpha(0.3)
                    },
                    border_width: 2.0,
                    corner_radius: 3.0,
                    interaction: vn_ui::InteractionState {
                        is_hovered: args.ctx.event_manager.borrow().is_hovered(args.id),
                        is_focused: false,
                    },
                    on_click: Some(EditorEvent::SelectTileset(path.clone())),
                }
            }),
            world,
        );

        recently_loaded_elements.push(Box::new(ts_button));
    }

    if !recently_loaded_elements.is_empty() {
        tileset_elements.push(Box::new(
            TextField::new(
                Box::new({
                    let metrics = metrics.clone();
                    move |_: StateToParamsArgs<'_, Editor>| TextFieldParams {
                        visuals: TextVisuals {
                            text: "Recently Loaded:".to_string(),
                            caret_position: None,
                            font: UI_FONT.to_string(),
                            font_size: UI_FONT_SIZE,
                            color: Color::WHITE.with_alpha(0.7),
                            caret_width: None,
                            caret_blink_duration: None,
                        },
                        metrics: metrics.clone(),
                        interaction: Default::default(),
                        text_field_action_handler: EventHandler::none(),
                    }
                }),
                world,
            )
            .padding(Box::new(|_| PaddingParams::uniform(5.0)), world),
        ));
        tileset_elements.push(Box::new(Flex::new_column_unweighted(
            recently_loaded_elements,
            false,
            world,
        )));
    }

    let element = Box::new(Flex::new_column_unweighted(tileset_elements, false, world));

    TilesetViewInfo {
        element,
        tileset_path_input_id: path_input_id,
        tile_width_input_id: tw_id,
        tile_height_input_id: th_id,
        tileset_cols_input_id: tsw_id,
        tileset_rows_input_id: tsh_id,
    }
}

fn build_selection_info(
    editor: &Editor,
    world: &mut ElementWorld,
    metrics: Rc<dyn TextMetrics>,
) -> Box<dyn Element<State = Editor, Message = EditorEvent>> {
    Box::new(TextField::new(
        Box::new({
            let metrics = metrics.clone();
            let layer_count = editor.map_spec.layers.len();
            let selected = editor.selected_layer_index;
            move |_: StateToParamsArgs<'_, Editor>| TextFieldParams {
                visuals: TextVisuals {
                    text: format!(
                        "Selected Layer: {} / {}",
                        (selected + 1).min(layer_count),
                        layer_count
                    ),
                    caret_position: None,
                    font: UI_FONT.to_string(),
                    font_size: UI_FONT_SIZE,
                    color: Color::WHITE.with_alpha(0.7),
                    caret_width: None,
                    caret_blink_duration: None,
                },
                metrics: metrics.clone(),
                interaction: Default::default(),
                text_field_action_handler: EventHandler::none(),
            }
        }),
        world,
    ))
}

fn build_footer(
    editor: &Editor,
    world: &mut ElementWorld,
    metrics: Rc<dyn TextMetrics>,
) -> Box<dyn Element<State = Editor, Message = EditorEvent>> {
    let mut footer_elements: Vec<Box<dyn Element<State = Editor, Message = EditorEvent>>> =
        Vec::new();
    for btn_text in ["Save", "Load", "Settings"] {
        let event = match btn_text {
            "Save" => EditorEvent::SaveMap,
            "Load" => EditorEvent::LoadMap,
            "Settings" => EditorEvent::OpenSettings,
            _ => unreachable!(),
        };
        let button = TextField::new(
            Box::new({
                let metrics = metrics.clone();
                let btn_text = btn_text.to_string();
                move |_: StateToParamsArgs<'_, Editor>| TextFieldParams {
                    visuals: TextVisuals {
                        text: btn_text.clone(),
                        caret_position: None,
                        font: UI_FONT.to_string(),
                        font_size: UI_FONT_SIZE,
                        color: Color::WHITE,
                        caret_width: None,
                        caret_blink_duration: None,
                    },
                    metrics: metrics.clone(),
                    interaction: Default::default(),
                    text_field_action_handler: EventHandler::none(),
                }
            }),
            world,
        )
        .padding(Box::new(|_| PaddingParams::uniform(5.0)), world)
        .button(
            Box::new({
                let event = event.clone();
                move |args| ButtonParams {
                    background: Color::WHITE.with_alpha(0.1),
                    border_color: if args.ctx.event_manager.borrow().is_hovered(args.id) {
                        Color::WHITE
                    } else {
                        Color::WHITE.with_alpha(0.3)
                    },
                    border_width: 2.0,
                    corner_radius: 3.0,
                    interaction: vn_ui::InteractionState {
                        is_hovered: args.ctx.event_manager.borrow().is_hovered(args.id),
                        is_focused: false,
                    },
                    on_click: Some(event.clone()),
                }
            }),
            world,
        );
        footer_elements.push(Box::new(button));
    }
    Box::new(Flex::new_row_unweighted(footer_elements, false, world))
}

fn build_tileset_preview_panel(
    editor: &Editor,
    world: &mut ElementWorld,
) -> (
    Box<dyn Element<State = Editor, Message = EditorEvent>>,
    ElementId,
) {
    let mut tileset_preview_elements: Vec<Box<dyn Element<State = Editor, Message = EditorEvent>>> =
        Vec::new();
    let mut scroll_area_id = world.next_id(); // Placeholder if no texture

    if let Some(layer) = editor.map_spec.layers.get(editor.selected_layer_index) {
        if let Some(texture_id) = editor.loaded_tilesets.get(&layer.tile_set) {
            let texture_id = texture_id.clone();
            let texture_preview = Texture::new(
                Box::new(move |_: StateToParamsArgs<'_, Editor>| TextureParams {
                    texture_id: texture_id.clone(),
                    preferred_size: vn_ui::ElementSize {
                        width: 256.0,
                        height: 4256.0,
                    },
                    uv_rect: Rect {
                        position: [0.0, 0.0],
                        size: [1.0, 1.0],
                    },
                    tint: Color::WHITE,
                    fit_strategy: FitStrategy::PreserveAspectRatio { rotation: 0.0 },
                }),
                world,
            );

            let grid_overlay = Grid::new(
                Box::new(move |_: StateToParamsArgs<'_, Editor>| GridParams {
                    rows: 133,
                    cols: 8,
                    grid_size: (32.0, 32.0),
                    grid_width: 3.0,
                    grid_color: Color::WHITE.with_alpha(0.5),
                }),
                world,
            );

            let scroll_area = Stack::new(
                vec![Box::new(texture_preview), Box::new(grid_overlay)],
                world,
            )
            .scroll_area(
                Box::new(
                    move |args: StateToParamsArgs<'_, Editor>| ScrollAreaParams {
                        scroll_x: Some(args.state.tileset_scroll_x),
                        scroll_y: Some(args.state.tileset_scroll_y),
                        scroll_action_handler: EventHandler::new(|id, action| {
                            EditorEvent::ScrollAction { id, action }
                        }),
                        scrollbar_width: 16.0,
                        scrollbar_margin: 8.0,
                    },
                ),
                world,
            );

            scroll_area_id = scroll_area.id();

            let preview_card = Box::new(scroll_area)
                .padding(Box::new(|_| PaddingParams::uniform(10.0)), world)
                .card(
                    Box::new(|_| CardParams {
                        background_color: Color::BLACK.with_alpha(0.3),
                        border_size: 2.0,
                        border_color: Color::WHITE.with_alpha(0.5),
                        corner_radius: 5.0,
                    }),
                    world,
                );
            tileset_preview_elements.push(Box::new(preview_card));
        }
    }
    (
        Box::new(
            Flex::new_column_unweighted(tileset_preview_elements, false, world)
                .padding(Box::new(|_| PaddingParams::uniform(10.0)), world)
                .card(
                    Box::new(|_| CardParams {
                        background_color: Color::BLACK.with_alpha(0.3),
                        border_size: 2.0,
                        border_color: Color::WHITE.with_alpha(0.5),
                        corner_radius: 5.0,
                    }),
                    world,
                ),
        ),
        scroll_area_id,
    )
}

pub fn build_fps_counter(
    metrics: Rc<dyn TextMetrics>,
    world: &mut ElementWorld,
) -> Box<dyn Element<State = Editor, Message = EditorEvent>> {
    let counter_text = TextField::new(
        Box::new(move |args: StateToParamsArgs<Editor>| TextFieldParams {
            visuals: TextVisuals {
                text: format!(
                    "FPS: {:7>.2}",
                    args.state
                        .fps
                        .borrow()
                        .current_fps
                        .borrow()
                        .as_ref()
                        .unwrap_or(&0.0)
                ),
                caret_position: None,
                font: UI_FONT.to_string(),
                font_size: UI_FONT_SIZE,
                color: Color::WHITE.with_alpha(0.3),
                caret_width: None,
                caret_blink_duration: None,
            },
            metrics: metrics.clone(),
            interaction: Default::default(),
            text_field_action_handler: EventHandler::none(),
        }),
        world,
    )
    .card(
        Box::new(|_| CardParams {
            background_color: Color::BLACK.with_alpha(0.3),
            border_size: 2.0,
            border_color: Color::WHITE.with_alpha(0.5),
            corner_radius: 5.0,
        }),
        world,
    );
    Box::new(counter_text)
}
