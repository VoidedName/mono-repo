use crate::{ElementImpl, ElementSize, SizeConstraints, UiContext};
use std::cell::RefCell;
use std::rc::Rc;
use vn_scene::Scene;
use vn_ui_definitions::{Component, ui_component};

ui_component!(Empty);

impl<State, Message: Clone> Component for Empty<State, Message> {
    type State = State;
    type Message = Message;
    type Params = ();

    fn layout(
        &mut self,
        _ctx: &mut UiContext,
        _state: &Self::State,
        _params: &Self::Params,
        constraints: SizeConstraints,
    ) -> ElementSize {
        constraints.min_size
    }

    fn draw(
        &mut self,
        _ctx: &mut UiContext,
        _state: &Self::State,
        _params: &Self::Params,
        _origin: (f32, f32),
        _size: ElementSize,
        _scene: &mut dyn Scene,
    ) {
    }

    fn handle_event(
        &mut self,
        _ctx: &mut UiContext,
        _state: &Self::State,
        _params: &Self::Params,
        _event: &crate::InteractionEvent,
    ) -> Vec<Self::Message> {
        vec![]
    }
}
