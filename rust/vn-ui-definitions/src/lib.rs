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

#[macro_export]
macro_rules! params {
    {$args:ident<$ty:ty> => $($expr:tt)*} => (move |$args: $crate::StateToParamsArgs<$ty>| { $($expr)* });
    {$args:ident => $($expr:tt)*} => (move |$args: $crate::StateToParamsArgs<_>| { $($expr)* });
    {$($expr:tt)*} => (move |args: $crate::StateToParamsArgs<_>| $($expr)*);
}

#[macro_export]
macro_rules! into_box_impl {
    ($ident:ident) => {
        impl<S: 'static, M: Clone + 'static>
            Into<Box<dyn $crate::Element<State = S, Message = M>>> for $ident<S, M>
        {
            fn into(self) -> Box<dyn $crate::Element<State = S, Message = M>> {
                Box::new(self)
            }
        }

        impl<S: 'static, M: Clone + 'static>
            Into<Box<dyn $crate::Element<State = S, Message = M>>>
            for Box<$ident<S, M>>
        {
            fn into(self) -> Box<dyn $crate::Element<State = S, Message = M>> {
                self
            }
        }
    };
}
