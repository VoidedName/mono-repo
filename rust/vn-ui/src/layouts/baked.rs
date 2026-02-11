// use crate::{
//     Element, ElementId, ElementImpl, ElementSize, ElementWorld, InteractionEvent, SizeConstraints,
//     UiContext, into_box_impl,
// };
// use std::cell::RefCell;
// use std::rc::Rc;
// use vn_scene::{
//     Color, ConstructableScene, GenericScene, ImagePrimitiveData, Rect, Scene, TextureId, Transform,
// };
//
// pub struct Baked<State: 'static, Message: 'static> {
//     id: ElementId,
//     child: Box<dyn Element<State = State, Message = Message>>,
//     cached_texture: Option<(TextureId, ElementSize)>,
//     invalidated: bool,
// }
//
// impl<State, Message> Baked<State, Message> {
//     pub fn new(
//         child: Box<dyn Element<State = State, Message = Message>>,
//         world: Rc<RefCell<ElementWorld>>,
//     ) -> Self {
//         Self {
//             id: world.borrow_mut().next_id(),
//             child,
//             cached_texture: None,
//             invalidated: true,
//         }
//     }
// }
//
// impl<State, Message> ElementImpl for Baked<State, Message> {
//     type State = State;
//     type Message = Message;
//
//     fn id_impl(&self) -> ElementId {
//         self.id
//     }
//
//     fn layout_impl(
//         &mut self,
//         ctx: &mut UiContext,
//         state: &Self::State,
//         constraints: SizeConstraints,
//     ) -> ElementSize {
//         let size = self.child.layout(ctx, state, constraints);
//         if let Some((_, cached_size)) = self.cached_texture {
//             if cached_size != size {
//                 self.invalidated = true;
//             }
//         }
//         size
//     }
//
//     fn draw_impl(
//         &mut self,
//         ctx: &mut UiContext,
//         state: &Self::State,
//         origin: (f32, f32),
//         size: ElementSize,
//         scene: &mut dyn Scene,
//     ) {
//         if self.invalidated || self.child.invalidated(ctx, state) || self.cached_texture.is_none() {
//             if let Some(hook) = ctx.scene_renderer.clone() {
//                 let mut sub_scene = GenericScene::new((size.width, size.height));
//                 self.child.draw(
//                     ctx,
//                     state,
//                     (0.0, 0.0), // Draw at origin in sub-scene
//                     size,
//                     &mut sub_scene,
//                 );
//
//                 let texture_id = hook.borrow().render_to_texture(
//                     &sub_scene,
//                     (size.width, size.height),
//                     self.cached_texture.take().map(|t| t.0),
//                 );
//                 self.cached_texture = Some((texture_id, size));
//                 self.invalidated = false;
//             }
//         }
//
//         if let Some((texture_id, _)) = &self.cached_texture {
//             scene.add_image(ImagePrimitiveData {
//                 transform: Transform {
//                     translation: [origin.0, origin.1],
//                     ..Transform::DEFAULT
//                 },
//                 size: [size.width, size.height],
//                 tint: Color::WHITE,
//                 texture_id: texture_id.clone(),
//                 clip_rect: ctx.clip_rect,
//                 uv_rect: Rect::UNIT,
//             });
//         } else {
//             // Fallback if baking failed or no hook available
//             self.child.draw(ctx, state, origin, size, scene);
//         }
//     }
//
//     fn handle_event_impl(
//         &mut self,
//         ctx: &mut UiContext,
//         state: &Self::State,
//         event: &InteractionEvent,
//     ) -> Vec<Self::Message> {
//         self.child.handle_event(ctx, state, event)
//     }
//
//     fn invalidated_impl(&self, ctx: &UiContext, state: &Self::State) -> bool {
//         self.invalidated || self.child.invalidated(ctx, state)
//     }
// }
//
// into_box_impl!(Baked);
