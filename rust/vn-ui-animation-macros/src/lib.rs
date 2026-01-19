use proc_macro::TokenStream;
use proc_macro2::{Span};
use quote::quote;
use syn::spanned::Spanned;
use syn::{Attribute, DeriveInput, Expr, Field, Lit, Meta, parse_macro_input, Index};

const IGNORE_INTERPOLATION: &str = "no_interpolation";
const IGNORE_INTERPOLATION_AT_START: &str = "flip_start";
const IGNORE_INTERPOLATION_AT_MIDDLE: &str = "flip_middle";
const IGNORE_INTERPOLATION_AT_END: &str = "flip_at_end";
const INTERPOLATE_NONE_AS_DEFAULT: &str = "interpolate_none_as_default";
const INTERPOLATE_NONE_AS: &str = "interpolate_none_as_value";

// todo deal with duplicate code

#[proc_macro_derive(
    Interpolatable,
    attributes(
        no_interpolation,
        interpolate_none_as_value,
        interpolate_none_as_default
    )
)]
pub fn interpolate(item: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(item as DeriveInput);
    let name = &ast.ident;

    let inner = if let syn::Data::Struct(data) = &ast.data {
        match data.fields {
            syn::Fields::Named(ref fields) => {
                let interpolation = fields.named.iter().map(|field| {
                    let ignore = helper_attr(field, IGNORE_INTERPOLATION);
                    let none_use_default = helper_attr(field, INTERPOLATE_NONE_AS_DEFAULT).is_some();
                    let none_use = helper_attr(field, INTERPOLATE_NONE_AS);

                    let field = &field.ident;
                    if let Some(attr) = ignore {
                        match &attr.meta {
                            Meta::NameValue(nv) => {
                                if let Expr::Lit(value) = &nv.value {
                                    if let Lit::Str(value) = &value.lit {
                                        match value.value().as_str() {
                                            IGNORE_INTERPOLATION_AT_START => {
                                                quote! {
                                                    result.#field = if t > 0.0 {
                                                        other.#field.clone()
                                                    } else {
                                                        self.#field.clone()
                                                    };
                                                }
                                            }
                                            IGNORE_INTERPOLATION_AT_MIDDLE => {
                                                quote! {
                                                    result.#field = if t >= 0.5 {
                                                        other.#field.clone()
                                                    } else {
                                                        self.#field.clone()
                                                    };
                                                }
                                            }
                                            IGNORE_INTERPOLATION_AT_END => {
                                                quote! {
                                                    result.#field = if t >= 1.0 {
                                                        other.#field.clone()
                                                    } else {
                                                        self.#field.clone()
                                                    };
                                                }
                                            }
                                            _ => {
                                                return TokenStream::from(
                                                    syn::Error::new(
                                                        attr.path().span(),
                                                        "'no_interpolation' must have a value like `no_interpolation = flip_start | flip_middle | flip_end`",
                                                    ).to_compile_error(),
                                                ).into();
                                            },
                                        }
                                    }else {
                                        return TokenStream::from(
                                            syn::Error::new(
                                                attr.path().span(),
                                                "'no_interpolation' must have a value like `no_interpolation = flip_start | flip_middle | flip_end`",
                                            ).to_compile_error(),
                                        ).into();
                                    }
                                } else {
                                    return TokenStream::from(
                                        syn::Error::new(
                                            attr.path().span(),
                                            "'no_interpolation' must have a value like `no_interpolation = flip_start | flip_middle | flip_end`",
                                        ).to_compile_error(),
                                    ).into();
                                }
                            }
                            _ => {
                                return TokenStream::from(
                                    syn::Error::new(
                                        attr.path().span(),
                                        "'no_interpolation' must have a value like `no_interpolation = flip_immediately | flip_middle | flip_end`",
                                    ).to_compile_error(),
                                ).into();
                            }
                        }
                    }
                    else {
                        if none_use_default {
                            quote! {
                                result.#field = match (self.#field, other.#field) {
                                    (Some(a), Some(b)) => Some(a.interpolate(&b, t)),
                                    (Some(a), None) => Some(a.interpolate(&Default::default(), t)),
                                    (None, Some(b)) => Some(b.interpolate(&Default::default(), 1.0 - t)),
                                    (None, None) => None,
                                };
                            }
                        } else if let Some(attr) = none_use {
                            match &attr.meta {
                                Meta::List(l) => {
                                    // consider improving errors, since it only accepts a list with a single value
                                    let value = &l.tokens;

                                    quote! {
                                        result.#field = match (self.#field, other.#field) {
                                            (Some(a), Some(b)) => Some(a.interpolate(&b, t)),
                                            (Some(a), None) => Some(a.interpolate(&#value, t)),
                                            (None, Some(b)) => Some(b.interpolate(&#value, 1.0 - t)),
                                            (None, None) => Some(#value),
                                        };
                                    }
                                }
                                _ => {
                                    return TokenStream::from(
                                        syn::Error::new(
                                            attr.path().span(),
                                            "'interpolate_none_as_value' must have a value like `interpolate_none_as_value(42u32)`",
                                        ).to_compile_error(),
                                    ).into();
                                }
                            }
                        }
                        else {
                            quote! {
                                result.#field = self.#field.interpolate(&other.#field, t);
                            }
                        }
                    }
                });

                quote! {
                    let mut result = self.clone();
                    #(#interpolation)*
                    result
                }
            }
            syn::Fields::Unnamed(ref fields) => {
                let interpolation = fields.unnamed.iter().enumerate().map(|(i, field)| {
                    let ignore = helper_attr(field, IGNORE_INTERPOLATION);
                    let none_use_default = helper_attr(field, INTERPOLATE_NONE_AS_DEFAULT).is_some();
                    let none_use = helper_attr(field, INTERPOLATE_NONE_AS);

                    let i = Index {
                        index: i as u32,
                        span: Span::call_site(),
                    };

                    if let Some(attr) = ignore {
                        match &attr.meta {
                            Meta::NameValue(nv) => {
                                if let Expr::Lit(value) = &nv.value {
                                    if let Lit::Str(value) = &value.lit {
                                        match value.value().as_str() {
                                            IGNORE_INTERPOLATION_AT_START => {
                                                quote! {
                                                    result.#i = if t > 0.0 {
                                                        other.#i.clone()
                                                    } else {
                                                        self.#i.clone()
                                                    };
                                                }
                                            }
                                            IGNORE_INTERPOLATION_AT_MIDDLE => {
                                                quote! {
                                                    result.#i = if t >= 0.5 {
                                                        other.#i.clone()
                                                    } else {
                                                        self.#i.clone()
                                                    };
                                                }
                                            }
                                            IGNORE_INTERPOLATION_AT_END => {
                                                quote! {
                                                    result.#i = if t >= 1.0 {
                                                        other.#i.clone()
                                                    } else {
                                                        self.#i.clone()
                                                    };
                                                }
                                            }
                                            _ => {
                                                return TokenStream::from(
                                                    syn::Error::new(
                                                        attr.path().span(),
                                                        "'no_interpolation' must have a value like `no_interpolation = flip_start | flip_middle | flip_end`",
                                                    ).to_compile_error(),
                                                ).into();
                                            },
                                        }
                                    }else {
                                        return TokenStream::from(
                                            syn::Error::new(
                                                attr.path().span(),
                                                "'no_interpolation' must have a value like `no_interpolation = flip_start | flip_middle | flip_end`",
                                            ).to_compile_error(),
                                        ).into();
                                    }
                                } else {
                                    return TokenStream::from(
                                        syn::Error::new(
                                            attr.path().span(),
                                            "'no_interpolation' must have a value like `no_interpolation = flip_start | flip_middle | flip_end`",
                                        ).to_compile_error(),
                                    ).into();
                                }
                            }
                            _ => {
                                return TokenStream::from(
                                    syn::Error::new(
                                        attr.path().span(),
                                        "'no_interpolation' must have a value like `no_interpolation = flip_immediately | flip_middle | flip_end`",
                                    ).to_compile_error(),
                                ).into();
                            }
                        }
                    }
                    else {
                        if none_use_default {
                            quote! {
                                result.#i = match (self.#i, other.#i) {
                                    (Some(a), Some(b)) => Some(a.interpolate(&b, t)),
                                    (Some(a), None) => Some(a.interpolate(&Default::default(), t)),
                                    (None, Some(b)) => Some(b.interpolate(&Default::default(), 1.0 - t)),
                                    (None, None) => None,
                                };
                            }
                        } else if let Some(attr) = none_use {
                            match &attr.meta {
                                Meta::List(l) => {
                                    // consider improving errors, since it only accepts a list with a single value
                                    let value = &l.tokens;

                                    quote! {
                                        result.#field = match (self.#field, other.#field) {
                                            (Some(a), Some(b)) => Some(a.interpolate(&b, t)),
                                            (Some(a), None) => Some(a.interpolate(&#value, t)),
                                            (None, Some(b)) => Some(b.interpolate(&#value, 1.0 - t)),
                                            (None, None) => Some(#value),
                                        };
                                    }
                                }
                                _ => {
                                    return TokenStream::from(
                                        syn::Error::new(
                                            attr.path().span(),
                                            "'interpolate_none_as_value' must have a value like `interpolate_none_as_value(42u32)`",
                                        ).to_compile_error(),
                                    ).into();
                                }
                            }
                        } else {
                            quote! {
                                result.#i = self.#i.interpolate(&other.#i, t);
                            }
                        }
                    }
                });

                quote! {
                    let mut result = self.clone();
                    #(#interpolation)*
                    result
                }
            }
            syn::Fields::Unit => {
                return TokenStream::from(
                    syn::Error::new(
                        name.span(),
                        "'Interpolatable' can only be derived for Named or Tuple Structs",
                    )
                    .to_compile_error(),
                );
            }
        }
    } else {
        return TokenStream::from(
            syn::Error::new(
                name.span(),
                "'Interpolatable' can only be derived for Named or Tuple Structs",
            )
            .to_compile_error(),
        );
    };

    let (impl_generics, type_generics, where_clause) = ast.generics.split_for_impl();

    let output = quote! {
        impl #impl_generics ::vn_ui_animation::Interpolatable for #name #type_generics #where_clause {
            fn interpolate(&self, other: &Self, t: f32) -> Self {
                #inner
            }
        }
    };

    output.into()
}

fn helper_attr<'a>(field: &'a Field, attr: &str) -> Option<&'a Attribute> {
    field
        .attrs
        .iter()
        .find(|a| a.meta.path().segments[0].ident.to_string() == attr)
}
