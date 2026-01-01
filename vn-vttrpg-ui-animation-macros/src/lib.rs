use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, parse_macro_input};

const IGNORE_INTERPOLATION: &str = "no_interpolation";

#[proc_macro_derive(Interpolatable, attributes(no_interpolation))]
pub fn interpolate(item: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(item as DeriveInput);
    let name = &ast.ident;

    let inner = if let syn::Data::Struct(data) = &ast.data {
        match data.fields {
            syn::Fields::Named(ref fields) => {
                let interpolation = fields.named.iter().map(|field| {
                    let ignore = field.attrs.iter().find(|a|
                        a.meta.path().segments[0].ident.to_string() == IGNORE_INTERPOLATION
                    ).is_some();

                    if !ignore {
                        let field = &field.ident;
                        quote! {
                            result.#field = self.#field.interpolate(&other.#field, t);
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
                    let ignore = field.attrs.iter().find(|a|
                        a.meta.path().segments[0].ident.to_string() == IGNORE_INTERPOLATION
                    ).is_some();

                    if !ignore {
                        quote! {
                            result.#i = self.#i.interpolate(&other.#i, t);
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
        impl #impl_generics ::vn_vttrpg_ui_animation::Interpolatable for #name #type_generics #where_clause {
            fn interpolate(&self, other: &Self, t: f32) -> Self {
                #inner
            }
        }
    };

    output.into()
}
