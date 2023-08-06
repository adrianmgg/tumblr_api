#![allow(unused)] // TODO remove me

use darling::{FromDeriveInput, FromMeta, FromField, util::Flag};
// use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse_macro_input, spanned::Spanned, DeriveInput};

#[proc_macro_derive(Builder, attributes(builder))]
pub fn builder_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match BuilderInputReceiver::from_derive_input(&input) {
        Err(err) => err.write_errors().into(),
        Ok(receiver) => quote!(#receiver).into(),
    }
}

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(builder), supports(struct_named))]
struct BuilderInputReceiver {
    ident: syn::Ident,
    generics: syn::Generics,
    data: darling::ast::Data<(), BuilderFieldReceiver>,
}

#[derive(Debug, FromField)]
#[darling(attributes(builder))]
struct BuilderFieldReceiver {
    ident: Option<syn::Ident>,
    ty: syn::Type,
    #[darling(rename = "set")]
    set_mode: BuilderFieldSetMode,
}

#[derive(Debug, FromMeta)]
enum BuilderFieldSetMode {
    Ctor(BuilderFieldSetViaCtor),
}

// TODO give this a better name
#[derive(Debug, FromMeta)]
struct BuilderFieldSetViaCtor {
    into: Flag,
}

impl ToTokens for BuilderInputReceiver {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let a = quote! {};
        tokens.extend(a);
    }
}
