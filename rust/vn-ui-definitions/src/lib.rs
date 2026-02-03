pub mod component;
pub mod context;
pub mod event;
pub mod geometry;
pub mod layout;

pub use component::*;
pub use context::*;
pub use event::*;
pub use geometry::*;
pub use layout::*;

/// Creates a UI component struct with automatic implementation of the Element trait.
///
/// This macro generates a component struct that integrates with the UI system by:
/// - Creating a struct with generic `State` and `Msg` type parameters
/// - Including an `ElementId` for tracking the component in the UI tree
/// - Supporting parameterized component behavior through `StateToParams`
/// - Deriving the `UiElement` trait implementation via `vn_ui_macros::UiElement`
///
/// # Syntax
///
/// ```ignore
/// ui_component!(ComponentName<ParamsType>);
/// ```
///
/// # Generated Structure
///
/// The macro generates:
/// - A public struct with `State` and `Msg` generic parameters
/// - An `id` field marked with `#[id]` attribute for element identification
/// - A `params` field marked with `#[params]` attribute for state-to-params conversion
/// - A `PhantomData` field to maintain the `Msg` type parameter
/// - A `new` constructor that accepts params and an `ElementWorld`
///
/// # Type Parameters
///
/// - `State: 'static` - The application state type this component can access
/// - `Msg: Clone + 'static` - The message type this component can emit
///
/// # Examples
///
/// - Simple parameters
/// ```ignore
/// use vn_ui_definitions::ui_component;
///
/// pub struct MyLabelParams {
///     pub label: String,
/// }
///
/// ui_component!(MyLabel<MyLabelParams>);
/// ```
///
/// - Parameterized with message type
/// ```ignore
/// use vn_ui_definitions::ui_component;
///
/// pub struct MyButtonParams<Message> {
///     pub label: String,
///     pub on_click: Message,
/// }
///
/// ui_component!(MyButton<MyButtonParams<Msg>>);
/// ```
///
/// # Notes
///
/// Components created with this macro must also implement the `Component` trait
/// to define their layout, drawing, and event handling behavior.
#[macro_export]
macro_rules! ui_component {
    ($name:ident<$params:ty>) => {
        #[derive(::vn_ui_macros::UiElement)]
        pub struct $name<State: 'static, Msg: Clone + 'static> {
            #[id]
            id: ::vn_ui_definitions::ElementId,
            #[params]
            params: ::vn_ui_definitions::StateToParams<State, $params>,
            _phantom: std::marker::PhantomData<Msg>,
        }

        impl<State: 'static, Msg: Clone + 'static> $name<State, Msg> {
            pub fn new(
                params: impl Into<::vn_ui_definitions::StateToParams<State, $params>>,
                world: Rc<RefCell<::vn_ui_definitions::ElementWorld>>,
            ) -> Self {
                Self {
                    id: world.borrow_mut().next_id(),
                    params: params.into(),
                    _phantom: std::marker::PhantomData,
                }
            }
        }
    };

    ($name:ident) => {
        $crate::ui_component!($name<()>);
    };
}

#[macro_export]
macro_rules! params {
    {$args:ident<$ty:ty>, $expr:expr} => (move |$args: $crate::StateToParamsArgs<$ty>| $expr);
    {$args:ident, $expr:expr} => (move |$args: $crate::StateToParamsArgs<_>| $expr);
    {$expr:expr} => (move |args: $crate::StateToParamsArgs<_>| $expr);
    {} => {$crate::params!(())};
}

#[macro_export]
macro_rules! into_box_impl {
    ($ident:ident) => {
        impl<S: 'static, M: Clone + 'static> Into<Box<dyn $crate::Element<State = S, Message = M>>>
            for $ident<S, M>
        {
            fn into(self) -> Box<dyn $crate::Element<State = S, Message = M>> {
                Box::new(self)
            }
        }

        impl<S: 'static, M: Clone + 'static> Into<Box<dyn $crate::Element<State = S, Message = M>>>
            for Box<$ident<S, M>>
        {
            fn into(self) -> Box<dyn $crate::Element<State = S, Message = M>> {
                self
            }
        }

        impl<S: 'static, M: Clone + 'static>
            Into<Rc<RefCell<dyn $crate::Element<State = S, Message = M>>>> for $ident<S, M>
        {
            fn into(self) -> Rc<RefCell<dyn $crate::Element<State = S, Message = M>>> {
                Rc::new(RefCell::new(self))
            }
        }

        impl<S: 'static, M: Clone + 'static>
            Into<Rc<RefCell<dyn $crate::Element<State = S, Message = M>>>> for Box<$ident<S, M>>
        {
            fn into(self) -> Rc<RefCell<dyn $crate::Element<State = S, Message = M>>> {
                Rc::new(RefCell::new(*self))
            }
        }
    };
}
