use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, parse_macro_input};

#[proc_macro_derive(UiElement, attributes(id, params))]
pub fn ui_element(item: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(item as DeriveInput);
    let name = &ast.ident;

    let fields = match &ast.data {
        syn::Data::Struct(data) => &data.fields,
        _ => {
            return syn::Error::new(
                name.span(),
                "'UiElement' can only be derived for Named Structs",
            )
            .to_compile_error()
            .into();
        }
    };

    let id_field = fields
        .iter()
        .find(|f| f.attrs.iter().any(|a| a.meta.path().is_ident("id")))
        .map(|f| f.ident.as_ref().unwrap());

    let params_field = fields
        .iter()
        .find(|f| f.attrs.iter().any(|a| a.meta.path().is_ident("params")))
        .map(|f| f.ident.as_ref().unwrap());

    let id_impl = if let Some(id_field) = id_field {
        quote! {
            fn id_impl(&self) -> ::vn_ui_definitions::ElementId {
                self.#id_field
            }
        }
    } else {
        quote! {}
    };

    let (impl_generics, type_generics, where_clause) = ast.generics.split_for_impl();

    let component_impl = if let Some(params_field) = params_field {
        quote! {
            impl #impl_generics #name #type_generics #where_clause {
                fn params(&self, state: &<Self as ::vn_ui_definitions::ElementImpl>::State, ctx: &mut ::vn_ui_definitions::UiContext) -> <Self as ::vn_ui_definitions::Component>::Params {
                    self.#params_field.call(::vn_ui_definitions::StateToParamsArgs {
                        state,
                        id: self.id_impl(),
                        ctx,
                    })
                }
            }
        }
    } else {
        quote! {}
    };

    let output = quote! {
        impl #impl_generics ::vn_ui_definitions::ElementImpl for #name #type_generics #where_clause {
            type State = <Self as ::vn_ui_definitions::Component>::State;
            type Message = <Self as ::vn_ui_definitions::Component>::Message;

            #id_impl

            fn layout_impl(
                &mut self,
                ctx: &mut ::vn_ui_definitions::UiContext,
                state: &Self::State,
                constraints: ::vn_ui_definitions::SizeConstraints,
            ) -> ::vn_ui_definitions::ElementSize {
                let params = self.params(state, ctx);
                <Self as ::vn_ui_definitions::Component>::layout(self, ctx, state, &params, constraints)
            }

            fn draw_impl(
                &mut self,
                ctx: &mut ::vn_ui_definitions::UiContext,
                state: &Self::State,
                origin: (f32, f32),
                size: ::vn_ui_definitions::ElementSize,
                scene: &mut dyn ::vn_scene::Scene,
            ) {
                let params = self.params(state, ctx);
                <Self as ::vn_ui_definitions::Component>::draw(self, ctx, state, &params, origin, size, scene);
            }

            fn handle_event_impl(
                &mut self,
                ctx: &mut ::vn_ui_definitions::UiContext,
                state: &Self::State,
                event: &::vn_ui_definitions::InteractionEvent,
            ) -> Vec<Self::Message> {
                let params = self.params(state, ctx);
                <Self as ::vn_ui_definitions::Component>::handle_event(self, ctx, state, &params, event)
            }
        }

        #component_impl

        ::vn_ui_definitions::into_box_impl!(#name);
    };

    output.into()
}
