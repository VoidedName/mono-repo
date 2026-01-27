use crate::logic::game_state::editor_ui::{editor, layers, tileset};
use crate::logic::game_state::{ApplicationStateEx, label};
use crate::logic::{ApplicationContext, ApplicationEvent};
use crate::{UI_FONT, UI_FONT_SIZE};
use std::cell::RefCell;
use std::rc::Rc;
use vn_scene::Color;
use vn_tilemap::TileMapSpecification;
use vn_ui::{
    AnchorExt, AnchorLocation, AnchorParams, Element, ElementWorld, EventManager, Flex, FlexChild,
    FlexDirection, FlexParams, params,
};

pub mod editor_ui;

#[derive(Debug)]
pub struct EditorState {
    tile_map: TileMapSpecification,
}

#[derive(Debug, Clone)]
pub enum EditorEvent {}

pub struct Editor {
    #[allow(unused)]
    ctx: ApplicationContext,
    ui: RefCell<Box<dyn Element<State = EditorState, Message = EditorEvent>>>,
    state: EditorState,
    event_manager: Rc<RefCell<EventManager>>,
}

impl Editor {
    pub async fn new(ctx: ApplicationContext) -> anyhow::Result<Self> {
        let world = &mut ElementWorld::new();

        let title = label(
            |_| "Tile Map Editor".to_string(),
            UI_FONT,
            UI_FONT_SIZE,
            Color::WHITE,
            ctx.text_metrics.clone(),
            world,
        )
        .anchor(
            params!(AnchorParams {
                location: AnchorLocation::Top
            }),
            world,
        );

        let layers = layers(&ctx, world);
        let editor = editor(&ctx, world);
        let tileset = tileset(&ctx, world);

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
                            world,
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
            world,
        );

        Ok(Self {
            ctx,
            ui: RefCell::new(Box::new(ui)),
            state: EditorState {
                tile_map: TileMapSpecification {
                    layers: vec![],
                    map_dimensions: (10, 5),
                },
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

        None
    }
}
