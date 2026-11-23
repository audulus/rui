//! Procedural macros for rui. Use them through the `rui` crate.

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{format_ident, quote};
use syn::{
    Attribute, Error, Expr, Fields, Ident, ItemStruct, Meta, Result, Type, Visibility,
    parse_macro_input,
};

/// Turns a struct into a rui component with state fields.
///
/// See the `rui::component` docs for usage.
#[proc_macro_attribute]
pub fn component(args: TokenStream, input: TokenStream) -> TokenStream {
    if !args.is_empty() {
        return Error::new(
            proc_macro2::TokenStream::from(args).into_iter().next().unwrap().span(),
            "#[component] takes no arguments",
        )
        .to_compile_error()
        .into();
    }
    let item = parse_macro_input!(input as ItemStruct);
    expand(item).unwrap_or_else(Error::into_compile_error).into()
}

struct StateField {
    vis: Visibility,
    ident: Ident,
    ty: Type,
    attrs: Vec<Attribute>,
    /// `None` means `Default::default()`.
    init: Option<Expr>,
}

fn expand(mut item: ItemStruct) -> Result<proc_macro2::TokenStream> {
    let name = item.ident.clone();

    let Fields::Named(fields) = &mut item.fields else {
        return Err(Error::new_spanned(
            &item.ident,
            "#[component] needs a struct with named fields",
        ));
    };

    // Split the fields into props (kept on the struct) and state.
    let mut state_fields = Vec::new();
    let mut props = syn::punctuated::Punctuated::<syn::Field, syn::token::Comma>::new();
    for field in std::mem::take(&mut fields.named) {
        let mut init = None;
        let mut is_state = false;
        let mut attrs = Vec::new();
        for attr in field.attrs {
            if attr.path().is_ident("state") {
                if is_state {
                    return Err(Error::new_spanned(attr, "duplicate #[state] attribute"));
                }
                is_state = true;
                match attr.meta {
                    Meta::Path(_) => {}
                    Meta::NameValue(nv) => init = Some(nv.value),
                    Meta::List(list) => {
                        return Err(Error::new_spanned(
                            list,
                            "use `#[state]` or `#[state = initial_value]`",
                        ));
                    }
                }
            } else {
                attrs.push(attr);
            }
        }
        if is_state {
            state_fields.push(StateField {
                vis: field.vis,
                ident: field.ident.expect("named field"),
                ty: field.ty,
                attrs,
                init,
            });
        } else {
            props.push(syn::Field { attrs, ..field });
        }
    }
    fields.named = props;

    if state_fields.is_empty() {
        return Err(Error::new_spanned(
            &item.ident,
            "#[component] needs at least one `#[state]` field; \
             for a component without state, implement `Component` directly",
        ));
    }
    if !item.generics.params.is_empty() {
        return Err(Error::new_spanned(
            &item.generics,
            "#[component] does not support generic structs yet",
        ));
    }

    let vis = &item.vis;
    let state_name = format_ident!("{}State", name);
    let set_name = format_ident!("{}Set", name);

    let state_defs = state_fields.iter().map(|f| {
        let StateField {
            vis,
            ident,
            ty,
            attrs,
            ..
        } = f;
        quote! { #(#attrs)* #vis #ident: #ty }
    });

    let inits = state_fields.iter().map(|f| {
        let ident = &f.ident;
        match &f.init {
            Some(expr) => quote! { #ident: #expr },
            None => quote! { #ident: ::core::default::Default::default() },
        }
    });

    let lens_names: Vec<Ident> = state_fields
        .iter()
        .map(|f| {
            Ident::new(
                &format!("__{}{}Lens", name, to_camel(&f.ident.to_string())),
                Span::call_site(),
            )
        })
        .collect();

    let lens_defs = state_fields.iter().zip(&lens_names).map(|(f, lens)| {
        let ident = &f.ident;
        let ty = &f.ty;
        quote! {
            #[doc(hidden)]
            #[derive(Clone, Copy)]
            #vis struct #lens;

            impl ::rui::Lens<#state_name, #ty> for #lens {
                fn focus<'a>(&self, data: &'a #state_name) -> &'a #ty {
                    &data.#ident
                }
                fn focus_mut<'a>(&self, data: &'a mut #state_name) -> &'a mut #ty {
                    &mut data.#ident
                }
            }
        }
    });

    let set_defs = state_fields.iter().zip(&lens_names).map(|(f, lens)| {
        let StateField { vis, ident, ty, .. } = f;
        quote! {
            #vis #ident: ::rui::Map<::rui::StateHandle<#state_name>, #lens, #ty, #state_name>
        }
    });

    let set_inits = state_fields.iter().zip(&lens_names).map(|(f, lens)| {
        let ident = &f.ident;
        quote! { #ident: ::rui::Map::new(handle, #lens) }
    });

    let state_doc = format!("State of [`{name}`], declared by its `#[state]` fields.");
    let set_doc = format!(
        "Bindings to each `#[state]` field of [`{name}`], for changing state \
         from event handlers (`cx[set.field] = ...`) or passing to child views."
    );

    Ok(quote! {
        #item

        #[doc = #state_doc]
        #[derive(Clone)]
        #vis struct #state_name {
            #(#state_defs,)*
        }

        #[doc = #set_doc]
        #[derive(Clone, Copy)]
        #vis struct #set_name {
            #(#set_defs,)*
        }

        #(#lens_defs)*

        impl ::rui::StatefulComponent for #name {
            type State = #state_name;

            fn init(&self) -> #state_name {
                #state_name { #(#inits,)* }
            }

            fn body(
                self,
                state: #state_name,
                handle: ::rui::StateHandle<#state_name>,
            ) -> impl ::rui::View {
                let set = #set_name { #(#set_inits,)* };
                #name::body(self, state, set)
            }
        }
    })
}

fn to_camel(s: &str) -> String {
    s.split('_')
        .filter(|p| !p.is_empty())
        .map(|p| {
            let mut c = p.chars();
            match c.next() {
                Some(first) => first.to_uppercase().chain(c).collect(),
                None => String::new(),
            }
        })
        .collect()
}
