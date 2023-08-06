#![allow(unused)] // TODO remove me

use darling::{util::Flag, FromDeriveInput, FromField, FromMeta};
// use proc_macro::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::{parse_macro_input, spanned::Spanned, DeriveInput};

#[proc_macro_derive(Builder, attributes(builder))]
pub fn builder_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match builder_derive_impl(&input) {
        Err(err) => err.write_errors().into(),
        Ok(token_stream) => token_stream.into(),
    }
}

fn builder_derive_impl(input: &DeriveInput) -> Result<proc_macro2::TokenStream, darling::Error> {
    BuilderInputReceiver::from_derive_input(input)?.do_thing()
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
    Setter(BuilderFieldSetViaSetter),
}

// TODO give this a better name
#[derive(Debug, FromMeta)]
struct BuilderFieldSetViaCtor {
    into: Flag,
}

// TODO give this a better name also
#[derive(Debug, FromMeta)]
struct BuilderFieldSetViaSetter {
}

impl BuilderInputReceiver {
    fn do_thing(&self) -> Result<proc_macro2::TokenStream, darling::Error> {
        let mut ret = proc_macro2::TokenStream::new();

        let fields = self
            .data
            .as_ref()
            .take_struct()
            .ok_or_else(|| darling::Error::custom("expected a struct").with_span(&self.ident))?
            .fields;

        let mut ctor_params = Vec::new();
        let mut ctor_generic_params = Vec::new();
        let mut ctor_where_clauses = Vec::new();
        let mut ctor_self_field_sets = Vec::new();
        let mut cur_ctor_typevar_num: u32 = 1;
        for field in fields {
            let ident = &field.ident;
            let ty = &field.ty;
            match &field.set_mode {
                BuilderFieldSetMode::Ctor(ctor) => {
                    if ctor.into.is_present() {
                        let prefixed = format_ident!("T{}", &cur_ctor_typevar_num);
                        ctor_params.push(quote!{ #ident: #prefixed });
                        ctor_generic_params.push(quote!{ #prefixed });
                        ctor_where_clauses.push(quote!{ #prefixed: Into<#ty> });
                        ctor_self_field_sets.push(quote!{ #ident: #ident.into() });
                        cur_ctor_typevar_num += 1;
                    }
                    else {
                        ctor_params.push(quote!{ #ident: #ty });
                        ctor_self_field_sets.push(quote!{ #ident: #ident });
                    }
                },
                BuilderFieldSetMode::Setter(setter) => {
                    ctor_self_field_sets.push(quote!{ #ident: ::core::default::Default::default() });
                },
            }
        }

        let ident = &self.ident;
        ret.extend(quote! {
            impl #ident {
                fn new<#(#ctor_generic_params),*>(#(#ctor_params),*) -> Self
                where #(#ctor_where_clauses),*
                {
                    Self {
                        #(#ctor_self_field_sets),*
                    }
                }
            }
        });

        Ok(ret)
    }
}
