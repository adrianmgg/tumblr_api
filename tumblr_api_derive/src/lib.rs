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
    vis: syn::Visibility,
    data: darling::ast::Data<(), BuilderFieldReceiver>,
    builder_class: Option<syn::Ident>,
}

#[derive(Debug, FromField)]
#[darling(attributes(builder))]
struct BuilderFieldReceiver {
    ident: Option<syn::Ident>,
    ty: syn::Type,
    vis: syn::Visibility,
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
    into: Flag,
    #[darling(multiple)]
    wrap_with: Vec<syn::Expr>,
    arg_type: Option<syn::Type>,
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

        // whose impl we're putting the builder setter methods into
        let builder_setters_impl_target = match &self.builder_class {
            Some(builder_class) => builder_class,
            None => &self.ident,
        };

        // create seperate builder struct if requested
        if let Some(builder_class) = &self.builder_class {
            let builder_class_fields: Vec<_> = fields
                .iter()
                .map(|BuilderFieldReceiver { ident, ty, .. }| quote! { #ident: #ty })
                .collect();
            let vis = &self.vis;
            let self_ident = &self.ident;
            ret.extend(quote!{
                #vis struct #builder_class {
                    #(#builder_class_fields),*
                }
            });
            let build_fn_field_sets: Vec<_> = fields
                .iter()
                .map(|BuilderFieldReceiver { ident, .. }| quote!{ #ident: self.#ident })
                .collect();
            ret.extend(quote!{
                impl #builder_class {
                    pub fn build(self) -> #self_ident {
                        #self_ident {
                            #(#build_fn_field_sets),*
                        }
                    }
                }
            });
        }

        // ctor related stuff
        // TODO add option to manually set this name?
        let ctor_func_name = match &self.builder_class {
            Some(_) => quote!{ builder },
            None => quote!{ new },
        };
        let ctor_return_type = match &self.builder_class {
            Some(ident) => ident,
            None => &self.ident,
        };
        let mut ctor_params = Vec::new();
        let mut ctor_generic_params = Vec::new();
        let mut ctor_where_clauses = Vec::new();
        let mut ctor_self_field_sets = Vec::new();
        let mut cur_ctor_typevar_num: u32 = 1;
        // setters
        let mut setter_funcs = Vec::new();
        //
        for field in fields {
            let ident = &field.ident;
            let ty = &field.ty;
            match &field.set_mode {
                BuilderFieldSetMode::Ctor(ctor) => {
                    if ctor.into.is_present() {
                        let prefixed = format_ident!("T{}", &cur_ctor_typevar_num);
                        ctor_params.push(quote!{ #ident: #prefixed });
                        ctor_generic_params.push(quote!{ #prefixed });
                        ctor_where_clauses.push(quote!{ #prefixed: ::core::convert::Into::<#ty> });
                        ctor_self_field_sets.push(quote!{ #ident: #ident.into() });
                        cur_ctor_typevar_num += 1;
                    }
                    else {
                        ctor_params.push(quote!{ #ident: #ty });
                        ctor_self_field_sets.push(quote!{ #ident: #ident });
                    }
                },
                BuilderFieldSetMode::Setter(setter) => {
                    // ctor-related stuff
                    ctor_self_field_sets.push(quote!{ #ident: ::core::default::Default::default() });
                    // build the setter(s?)
                    let mut setter_where_clauses = Vec::new();
                    let mut setter_type_vars = Vec::new();
                    let mut setter_val = quote!{ #ident };
                    if setter.into.is_present() {
                        setter_val = quote!{ #setter_val.into() };
                    }
                    for wrap_with in &setter.wrap_with {
                        setter_val = quote!{ (#wrap_with)(#setter_val) };
                    }
                    let mut setter_arg_type = if let Some(arg_type) = &setter.arg_type {
                        quote!{ #arg_type }
                    } else {
                        quote!{ #ty }
                    };
                    if setter.into.is_present() {
                        let typevar = quote!{T};
                        setter_type_vars.push(typevar.clone());
                        setter_where_clauses.push(quote!{ #typevar: ::core::convert::Into::<#setter_arg_type> });
                        setter_arg_type = typevar;
                    }
                    setter_funcs.push(quote!{
                        #[allow(clippy::missing_const_for_fn)]
                        #[must_use]
                        pub fn #ident <#(#setter_type_vars),*>(mut self, #ident: #setter_arg_type) -> Self
                        where #(#setter_where_clauses),*
                        {
                            self.#ident = #setter_val;
                            self
                        }
                    });
                },
            }
        }

        let ident = &self.ident;
        ret.extend(quote! {
            impl #ident {
                pub fn #ctor_func_name <#(#ctor_generic_params),*>(#(#ctor_params),*) -> #ctor_return_type
                where #(#ctor_where_clauses),*
                {
                    #ctor_return_type {
                        #(#ctor_self_field_sets),*
                    }
                }
            }
            impl #builder_setters_impl_target {
                #(#setter_funcs)*
            }
        });

        Ok(ret)
    }
}
