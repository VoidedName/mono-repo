//! # vn-ui-animation-macros
//!
//! This crate provides procedural macros for implementing animation interpolation in the VN UI framework.
//!
//! ## `Interpolatable` Derive Macro
//!
//! The `Interpolatable` derive macro automatically implements the `vn_ui_animation::Interpolatable` trait
//! for structs (both named and tuple structs). This trait enables smooth transitions between two states
//! by interpolating field values based on a time parameter `t` (ranging from 0.0 to 1.0).
//!
//! ### Supported Attributes
//!
//! #### `#[interpolate_snappy = "snap_point"]`
//! Creates a discrete transition that "snaps" at a specific point during interpolation.
//! Useful for fields that should change instantly rather than gradually.
//!
//! Available snap points:
//! - `"snap_start"` - Switches to the target value when `t > 0.0`
//! - `"snap_middle"` - Switches to the target value when `t >= 0.5`
//! - `"snap_end"` - Switches to the target value when `t >= 1.0`
//! - A float literal between `0.0` and `1.0` (e.g., `0.25`) - Switches to target value when `t >= float`
//!
//! **Example:**
//! ```rust,ignore
//! #[derive(Clone, Interpolatable)]
//! struct UIState {
//!     #[interpolate_snappy = "snap_middle"]
//!     fit_strategy: FitStrategy,
//!     #[interpolate_snappy = 0.75]
//!     is_visible: bool,
//! }
//! ```
//!
//! #### `#[interpolate_none_as_default]`
//! For `Option<T>` fields, treats `None` values as `Default::default()` during interpolation.
//! When one side is `Some(value)` and the other is `None`, it interpolates between the value
//! and the default value of type `T`.
//!
//! **Example:**
//! ```rust,ignore
//! #[derive(Clone, Interpolatable)]
//! struct Transform {
//!     #[interpolate_none_as_default]
//!     rotation: Option<f32>,
//! }
//! ```
//!
//! #### `#[interpolate_none_as_value(value)]`
//! For `Option<T>` fields, treats `None` values as a specific provided value during interpolation.
//! Similar to `interpolate_none_as_default`, but allows you to specify the fallback value.
//!
//! **Example:**
//! ```rust,ignore
//! #[derive(Clone, Interpolatable)]
//! struct Style {
//!     #[interpolate_none_as_value(0.0)]
//!     opacity: Option<f32>,
//! }
//! ```
//!
//! ### Usage
//!
//! Basic usage with automatic field interpolation:
//! ```rust,ignore
//! use vn_ui_animation_macros::Interpolatable;
//!
//! #[derive(Clone, Interpolatable)]
//! struct Position {
//!     x: f32,
//!     y: f32,
//! }
//!
//! let start = Position { x: 0.0, y: 0.0 };
//! let end = Position { x: 100.0, y: 100.0 };
//! let halfway = start.interpolate(&end, 0.5); // Position { x: 50.0, y: 50.0 }
//! ```
//!
//! Advanced usage with multiple attribute types:
//! ```rust,ignore
//! use vn_ui_animation_macros::Interpolatable;
//!
//! #[derive(Clone, Interpolatable)]
//! struct TextureParams {
//!     pub texture_id: TextureId,
//!     pub preferred_size: ElementSize,
//!     pub uv_rect: Rect,
//!     pub tint: Color,
//!     #[interpolate_snappy = "snap_middle"]
//!     pub fit_strategy: FitStrategy,
//! }
//! ```

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::spanned::Spanned;
use syn::{Attribute, DeriveInput, Expr, Field, Index, Lit, Meta, parse_macro_input};

const INTERPOLATE_SNAPPY: &str = "interpolate_snappy";
const SNAP_INTERPOLATION_AT_START: &str = "snap_start";
const SNAP_INTERPOLATION_AT_MIDDLE: &str = "snap_middle";
const SNAP_INTERPOLATION_AT_END: &str = "snap_end";
const INTERPOLATE_NONE_AS_DEFAULT: &str = "interpolate_none_as_default";
const INTERPOLATE_NONE_AS: &str = "interpolate_none_as_value";

/// Derives the `Interpolatable` trait for a struct.
///
/// This macro generates an implementation of `vn_ui_animation::Interpolatable` that enables
/// smooth interpolation between two instances of the struct. Each field is interpolated
/// individually based on its type and any applied attributes.
///
/// # Supported Types
///
/// - Named structs: `struct Point { x: f32, y: f32 }`
/// - Tuple structs: `struct Point(f32, f32)`
///
/// # Attributes
///
/// See the module-level documentation for detailed information about supported attributes:
/// - `#[interpolate_snappy]`
/// - `#[interpolate_none_as_default]`
/// - `#[interpolate_none_as_value]`
///
/// # Examples
///
/// See the module-level documentation for usage examples.
#[proc_macro_derive(
    Interpolatable,
    attributes(
        interpolate_snappy,
        interpolate_none_as_value,
        interpolate_none_as_default
    )
)]
pub fn interpolate(item: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(item as DeriveInput);
    let name = &ast.ident;

    let fields = match &ast.data {
        syn::Data::Struct(data) => &data.fields,
        _ => {
            return syn::Error::new(
                name.span(),
                "'Interpolatable' can only be derived for Named or Tuple Structs",
            )
            .to_compile_error()
            .into();
        }
    };

    let interpolation = match fields {
        syn::Fields::Named(fields) => fields
            .named
            .iter()
            .map(|field| {
                let ident = field.ident.as_ref().unwrap();
                process_field(field, quote!(#ident))
            })
            .collect::<Vec<_>>(),
        syn::Fields::Unnamed(fields) => fields
            .unnamed
            .iter()
            .enumerate()
            .map(|(i, field)| {
                let index = Index::from(i);
                process_field(field, quote!(#index))
            })
            .collect::<Vec<_>>(),
        syn::Fields::Unit => {
            return syn::Error::new(
                name.span(),
                "'Interpolatable' can only be derived for Named or Tuple Structs",
            )
            .to_compile_error()
            .into();
        }
    };

    let (impl_generics, type_generics, where_clause) = ast.generics.split_for_impl();

    let output = quote! {
        impl #impl_generics ::vn_ui_animation::Interpolatable for #name #type_generics #where_clause {
            fn interpolate(&self, other: &Self, t: f32) -> Self {
                let mut result = self.clone();
                #(#interpolation)*
                result
            }
        }
    };

    output.into()
}

enum InterpolationMode<'a> {
    Snapping(&'a Attribute),
    NoneUseDefault,
    NoneUseValue(&'a Attribute),
    Normal,
}

fn interpolation_mode(field: &'_ Field) -> InterpolationMode<'_> {
    if let Some(attr) = helper_attr(field, INTERPOLATE_SNAPPY) {
        InterpolationMode::Snapping(attr)
    } else if let Some(_) = helper_attr(field, INTERPOLATE_NONE_AS_DEFAULT) {
        InterpolationMode::NoneUseDefault
    } else if let Some(attr) = helper_attr(field, INTERPOLATE_NONE_AS) {
        InterpolationMode::NoneUseValue(attr)
    } else {
        InterpolationMode::Normal
    }
}

fn process_field(field: &Field, access: TokenStream2) -> TokenStream2 {
    let mode = interpolation_mode(field);

    match mode {
        InterpolationMode::Snapping(attr) => interpolate_snappy(&access, &attr),
        InterpolationMode::NoneUseDefault => interpolate_none_as_default(&access),
        InterpolationMode::NoneUseValue(attr) => interpolate_none_as_value(&access, &attr),
        InterpolationMode::Normal => quote! {
            result.#access = self.#access.interpolate(&other.#access, t);
        },
    }
}

fn interpolate_none_as_value(access: &TokenStream2, attr: &Attribute) -> TokenStream2 {
    match &attr.meta {
        Meta::List(l) => {
            let value = &l.tokens;
            quote! {
                result.#access = match (&self.#access, &other.#access) {
                    (Some(a), Some(b)) => Some(a.interpolate(b, t)),
                    (Some(a), None) => Some(a.interpolate(&#value, t)),
                    (None, Some(b)) => Some(b.interpolate(&#value, 1.0 - t)),
                    (None, None) => Some(#value.clone()),
                };
            }
        }
        _ => syn::Error::new(
            attr.path().span(),
            "'interpolate_none_as_value' must have a value like `interpolate_none_as_value(42u32)`",
        )
        .to_compile_error(),
    }
}

fn interpolate_none_as_default(access: &TokenStream2) -> TokenStream2 {
    quote! {
        result.#access = match (&self.#access, &other.#access) {
            (Some(a), Some(b)) => Some(a.interpolate(b, t)),
            (Some(a), None) => Some(a.interpolate(&Default::default(), t)),
            (None, Some(b)) => Some(b.interpolate(&Default::default(), 1.0 - t)),
            (None, None) => None,
        };
    }
}

fn interpolate_snappy(access: &TokenStream2, attr: &Attribute) -> TokenStream2 {
    let snap_point = match &attr.meta {
        Meta::NameValue(nv) => {
            if let Expr::Lit(expr_lit) = &nv.value {
                match &expr_lit.lit {
                    Lit::Str(lit_str) => match lit_str.value().as_str() {
                        SNAP_INTERPOLATION_AT_START => Some(f32::MIN_POSITIVE),
                        SNAP_INTERPOLATION_AT_MIDDLE => Some(0.5),
                        SNAP_INTERPOLATION_AT_END => Some(1.0),
                        _ => None,
                    },
                    Lit::Float(lit_float) => {
                        if let Ok(val) = lit_float.base10_parse::<f32>() {
                            if (0.0..=1.0).contains(&val) {
                                Some(val)
                            } else {
                                return syn::Error::new(
                                    lit_float.span(),
                                    "Snap point must be between 0.0 and 1.0",
                                )
                                .to_compile_error();
                            }
                        } else {
                            None
                        }
                    }
                    _ => None,
                }
            } else {
                None
            }
        }
        _ => None,
    };

    if let Some(snap_point) = snap_point {
        return quote! {
            result.#access = if t >= #snap_point {
                other.#access.clone()
            } else {
                self.#access.clone()
            };
        };
    }

    syn::Error::new(
        attr.path().span(),
        format!("'interpolate_snappy' must have a value like `interpolate_snappy = \"{}\" | \"{}\" | \"{}\"` or a float between 0.0 and 1.0",
                SNAP_INTERPOLATION_AT_START, SNAP_INTERPOLATION_AT_MIDDLE, SNAP_INTERPOLATION_AT_END),
    ).to_compile_error()
}

fn helper_attr<'a>(field: &'a Field, attr: &str) -> Option<&'a Attribute> {
    field
        .attrs
        .iter()
        .find(|a| a.meta.path().segments[0].ident.to_string() == attr)
}
