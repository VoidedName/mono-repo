use crate::logic::game_state::{EditorEvent, EditorState, btn, empty_texture, label};
use crate::logic::{ApplicationContext, Grid, GridParams};
use crate::{UI_FONT, UI_FONT_SIZE};
use std::cell::RefCell;
use std::rc::Rc;
use vn_scene::{Color, Rect};
use vn_tilemap::{TileMap, TileMapParams};
use vn_ui::*;
use vn_wgpu_window::resource_manager::Sampling;

pub fn layers(
    ctx: &ApplicationContext,
    world: Rc<RefCell<ElementWorld>>,
) -> Box<dyn Element<State = EditorState, Message = EditorEvent>> {
    let title = label(
        |_| "Layer Settings".to_string(),
        UI_FONT,
        UI_FONT_SIZE,
        Color::WHITE,
        ctx.text_metrics.clone(),
        world.clone(),
    )
    .padding(
        params!(PaddingParams {
            pad_bottom: 25.0,
            ..Default::default()
        }),
        world.clone(),
    )
    .anchor(center!(), world.clone());

    let new_layer = btn(
        |_| "Add Layer".to_string(),
        UI_FONT,
        UI_FONT_SIZE,
        |_| false,
        |_| Color::WHITE,
        |_| Color::WHITE,
        |_| Color::WHITE,
        ctx.text_metrics.clone(),
        EventHandler::new(|_, e| match e {
            ButtonAction::Clicked => {
                vec![EditorEvent::TryAddingLayer]
            }
        }),
        world.clone(),
    );

    let save = btn(
        |_| "Save".to_string(),
        UI_FONT,
        UI_FONT_SIZE,
        |_| false,
        |_| Color::WHITE,
        |_| Color::WHITE,
        |_| Color::WHITE,
        ctx.text_metrics.clone(),
        EventHandler::new(|_, e| match e {
            ButtonAction::Clicked => {
                vec![EditorEvent::SaveSpec]
            }
        }),
        world.clone(),
    );

    let load = btn(
        |_| "Load".to_string(),
        UI_FONT,
        UI_FONT_SIZE,
        |_| false,
        |_| Color::WHITE,
        |_| Color::WHITE,
        |_| Color::WHITE,
        ctx.text_metrics.clone(),
        EventHandler::new(|_, e| match e {
            ButtonAction::Clicked => {
                vec![EditorEvent::LoadSpec]
            }
        }),
        world.clone(),
    );

    let save_load = Flex::new(
        {
            let c = vec![
                FlexChild::new(save).into_rc_refcell(),
                FlexChild::new(
                    Empty::new(world.clone())
                        .padding(params!(PaddingParams::horizontal(25.0)), world.clone()),
                )
                .into_rc_refcell(),
                FlexChild::new(load).into_rc_refcell(),
            ];
            params!(FlexParams {
                direction: FlexDirection::Row,
                force_orthogonal_same_size: false,
                children: c.clone()
            })
        },
        world.clone(),
    )
    .anchor(bottom_right!(), world.clone());

    let layer_flex = Flex::new(
        {
            let cache: Rc<RefCell<Vec<Rc<RefCell<FlexChild<EditorState, EditorEvent>>>>>> =
                Rc::new(RefCell::new(vec![]));
            let world = world.clone();
            let metrics = ctx.text_metrics.clone();

            params!(args<EditorState> => {
                let cache_len = { cache.borrow().len() };

                for idx in cache_len..args.state.tile_map.layers.len() {
                     let layer = btn(
                        move |state: &EditorState| state.tile_map.layers[idx].name.clone(),
                        UI_FONT,
                        UI_FONT_SIZE,
                        |_| false,
                        move |state: &EditorState| if state.current_layer.map(|l| l == idx).unwrap_or(false) {
                            Color::GREEN
                        } else {
                            Color::WHITE.with_alpha(0.8)
                        },
                        |_| Color::TRANSPARENT,
                        |_| Color::WHITE,
                        metrics.clone(),
                        EventHandler::new(move |_, e| match e {
                            ButtonAction::Clicked => vec![EditorEvent::SwitchToLayer(idx)],
                        }),
                        world.clone(),
                    );

                    cache.borrow_mut().push(FlexChild::new(
                        layer
                    ).into_rc_refcell())
                }

                FlexParams {
                    direction: FlexDirection::Column,
                    children: cache.borrow()[0..args.state.tile_map.layers.len()].iter().cloned().collect(),
                    force_orthogonal_same_size: true
            }})
        },
        world.clone(),
    );

    let layer_list = layer_flex
        .padding(params!(PaddingParams::uniform(5.0)), world.clone())
        .card(
            params!(CardParams {
                border_color: Color::WHITE,
                corner_radius: 5.0,
                border_size: 2.0,
                background_color: Color::BLACK,
            }),
            world.clone(),
        );

    Flex::new(
        {
            let c = vec![
                FlexChild::new(Flex::new(
                    {
                        let c = vec![
                            FlexChild::weighted(Empty::new(world.clone()), 1.0).into_rc_refcell(),
                        ];
                        params!(FlexParams {
                            force_orthogonal_same_size: true,
                            direction: FlexDirection::Row,
                            children: c.clone()
                        })
                    },
                    world.clone(),
                ))
                .into_rc_refcell(),
                FlexChild::new(title).into_rc_refcell(),
                FlexChild::new(
                    layer_list.padding(params!(PaddingParams::bottom(25.0)), world.clone()),
                )
                .into_rc_refcell(),
                FlexChild::new(new_layer).into_rc_refcell(),
                FlexChild::weighted(Empty::new(world.clone()), 1.0).into_rc_refcell(),
                FlexChild::new(save_load).into_rc_refcell(),
            ];
            params!(FlexParams {
                direction: FlexDirection::Column,
                children: c.clone(),
                force_orthogonal_same_size: true,
            })
        },
        world.clone(),
    )
    .padding(params!(PaddingParams::uniform(25.0)), world.clone())
    .anchor(top!(), world.clone())
    .fill(world.clone())
    .card(
        params!(CardParams {
            border_color: Color::WHITE,
            border_size: 2.0,
            background_color: Color::BLACK,
            corner_radius: 5.0,
        }),
        world.clone(),
    )
    .prefer_size(
        params!(PreferSizeParams {
            width: Some(400.0),
            height: None,
        }),
        world.clone(),
    )
    .into()
}

pub fn editor(
    ctx: &ApplicationContext,
    world: Rc<RefCell<ElementWorld>>,
) -> Box<dyn Element<State = EditorState, Message = EditorEvent>> {
    let title = label(
        |_| "Map".to_string(),
        UI_FONT,
        UI_FONT_SIZE,
        Color::WHITE,
        ctx.text_metrics.clone(),
        world.clone(),
    )
    .padding(
        params!(PaddingParams {
            pad_bottom: 25.0,
            ..Default::default()
        }),
        world.clone(),
    )
    .anchor(center!(), world.clone());

    let grid = Grid::new(
        params!(args<EditorState> =>
        GridParams {
            cols: args.state.tile_map.map_dimensions.0,
            rows: args.state.tile_map.map_dimensions.1,
            grid_color: Color::WHITE.with_alpha(0.5),
            grid_width: 3.0,
            grid_size: (32.0, 32.0),
        }),
        world.clone(),
    );

    let map = TileMap::new(
        params!(args<EditorState> => TileMapParams {
            draw_tile_size: ElementSize {
                width: 32.0,
                height: 32.0,
            },
            textures: args.state.tile_map.layers.iter()
                .map(|l| args.state.loaded_tilesets.get(&l.tileset).unwrap().clone())
                .collect(),
            specification: args.state.tile_map.clone(),
        }),
        world.clone(),
    );

    let map = Stack::new(vec![map.into(), grid.into()], world.clone())
        .anchor(center!(), world.clone())
        .scroll_area(
            params!(args<EditorState> => ScrollAreaParams {
                scroll_x: args.state.tilemap_view_scroll_x,
                scroll_y: args.state.tilemap_view_scroll_y,
                scroll_action_handler: EventHandler::new(|_, e| match e {
                    ScrollAreaAction::ScrollX(v) => vec![EditorEvent::TilemapViewScrollX(v)],
                    ScrollAreaAction::ScrollY(v) => vec![EditorEvent::TilemapViewScrollY(v)],
                })
            }),
            world.clone(),
        )
        .fill(world.clone());

    Box::new(Flex::new(
        {
            let c = vec![
                FlexChild::new(Flex::new(
                    {
                        let c =
                            FlexChild::weighted(Empty::new(world.clone()), 1.0).into_rc_refcell();
                        params!(FlexParams {
                            direction: FlexDirection::Row,
                            children: vec![c.clone()],
                            force_orthogonal_same_size: true,
                        })
                    },
                    world.clone(),
                ))
                .into_rc_refcell(),
                FlexChild::new(title).into_rc_refcell(),
                FlexChild::weighted(map, 1.0).into_rc_refcell(),
            ];
            params!(FlexParams {
                children: c.clone(),
                direction: FlexDirection::Column,
                force_orthogonal_same_size: true,
            })
        },
        world.clone(),
    ))
}

pub fn tileset(
    ctx: &ApplicationContext,
    world: Rc<RefCell<ElementWorld>>,
) -> Box<dyn Element<State = EditorState, Message = EditorEvent>> {
    let title = label(
        |_| "Tileset".to_string(),
        UI_FONT,
        UI_FONT_SIZE,
        Color::WHITE,
        ctx.text_metrics.clone(),
        world.clone(),
    )
    .padding(
        params!(PaddingParams {
            pad_bottom: 25.0,
            ..Default::default()
        }),
        world.clone(),
    )
    .anchor(center!(), world.clone());

    let empty_text = ctx
        .rm
        .load_texture_from_bytes(empty_texture(), Sampling::Nearest)
        .expect("empty texture");

    let tileset_tex = Texture::new(
        params!(args<EditorState> =>
            let id = args.state.current_layer.map(|layer |
                    args.state.loaded_tilesets
                        .get(&args.state.tile_map.layers[layer].tileset)
                        .unwrap().clone())
                    .unwrap_or(empty_text.id.clone());
            let size = args.state.current_layer.map(|layer | {
                (
                    args.state.tile_map.layers[layer].tileset_dimensions.0 * args.state.tile_map.layers[layer].tile_dimensions.0,
                    args.state.tile_map.layers[layer].tileset_dimensions.1 * args.state.tile_map.layers[layer].tile_dimensions.1,
                )
            }).unwrap_or((0, 0));

            let size = ElementSize {
                width: size.0 as f32,
                height: size.1 as f32,
            };

            TextureParams {
                texture_id: id,
                preferred_size: size,
                uv_rect: Rect::UNIT,
                tint: Color::WHITE,
                fit_strategy: FitStrategy::Clip { rotation: 0.0 },
            }
        ),
        world.clone(),
    );

    let grid = Grid::new(
        params!(args<EditorState> =>
        let (cols, rows, grid_w, grid_h) = args.state.current_layer.map(|layer | {
            let tileset_dim = args.state.tile_map.layers[layer].tileset_dimensions;
            let tile_dim = args.state.tile_map.layers[layer].tile_dimensions;

            (tileset_dim.0, tileset_dim.1, tile_dim.0, tile_dim.1)
        }).unwrap_or((0, 0, 0, 0));

        GridParams {
            cols,
            rows,
            grid_color: Color::WHITE.with_alpha(0.25),
            grid_width: 3.0,
            grid_size: (grid_w as f32, grid_h as f32),
        }),
        world.clone(),
    );

    // todo: add click behaviour for brushes

    let layout = Stack::new(vec![tileset_tex.into(), grid.into()], world.clone());

    let tileset = ScrollArea::new(
        layout,
        params!(args<EditorState> => ScrollAreaParams {
            scroll_x: args.state.tileset_view_scroll_x.clone(),
            scroll_y: args.state.tileset_view_scroll_y.clone(),
            scroll_action_handler: EventHandler::new(|_, e| {
                match e {
                    ScrollAreaAction::ScrollX(v) => vec![EditorEvent::TilesetViewScrollX(v)],
                    ScrollAreaAction::ScrollY(v) => vec![EditorEvent::TilesetViewScrollY(v)],
                }
            })
        }),
        world.clone(),
    );

    Box::new(
        Flex::new(
            {
                let c = vec![
                    FlexChild::new(title).into_rc_refcell(),
                    FlexChild::weighted(tileset, 1.0).into_rc_refcell(),
                ];
                params!(FlexParams {
                    force_orthogonal_same_size: true,
                    direction: FlexDirection::Column,
                    children: c.clone(),
                })
            },
            world.clone(),
        )
        .padding(params!(PaddingParams::uniform(25.0)), world.clone())
        .anchor(top!(), world.clone())
        .fill(world.clone())
        .card(
            params!(CardParams {
                border_color: Color::WHITE,
                corner_radius: 5.0,
                border_size: 2.0,
                background_color: Color::BLACK,
            }),
            world.clone(),
        )
        .prefer_size(
            params!(PreferSizeParams {
                width: Some(400.0),
                height: None,
            }),
            world.clone(),
        ),
    )
}
