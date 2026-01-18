use proc_macro::TokenStream;
use quote::quote;
use syn::spanned::Spanned;
use syn::{Attribute, DeriveInput, Field, Meta, parse_macro_input};

const IGNORE_INTERPOLATION: &str = "no_interpolation";
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
                    let ignore = helper_attr(field, IGNORE_INTERPOLATION).is_some();
                    let none_use_default = helper_attr(field, INTERPOLATE_NONE_AS_DEFAULT).is_some();
                    let none_use = helper_attr(field, INTERPOLATE_NONE_AS);

                    if !ignore {
                        let field = &field.ident;
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
                    } else {
                        quote! {}
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
                    let ignore = helper_attr(field, IGNORE_INTERPOLATION).is_some();
                    let none_use_default = helper_attr(field, INTERPOLATE_NONE_AS_DEFAULT).is_some();
                    let none_use = helper_attr(field, INTERPOLATE_NONE_AS);

                    if !ignore {
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
                    } else {
                        quote! {}
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
