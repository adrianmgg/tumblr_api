use darling::{util::Flag, FromDeriveInput, FromField, FromMeta};
use quote::{format_ident, quote, ToTokens};
use syn::{parse_macro_input, spanned::Spanned, DeriveInput, Ident, Visibility};

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
    ident: Ident,
    // generics: syn::Generics,
    vis: Visibility,
    data: darling::ast::Data<(), BuilderFieldReceiver>,
    builder_class: Option<Ident>,
    build_fn: Option<BuildFnInfo>,
    #[darling(default, rename = "ctor")]
    ctor_options: CtorOptions,
}

#[derive(Debug, FromMeta, Default)]
struct CtorOptions {
    #[darling(rename = "vis")]
    visibility: Option<Visibility>,
}

#[derive(Debug, FromField)]
#[darling(attributes(builder))]
struct BuilderFieldReceiver {
    ident: Option<Ident>,
    ty: syn::Type,
    // vis: Visibility,
    #[darling(rename = "set")]
    set_mode: BuilderFieldSetMode,
}

#[derive(Debug, FromMeta)]
enum BuilderFieldSetMode {
    Ctor(darling::util::Override<BuilderFieldSetViaCtor>),
    Setter(BuilderFieldSetViaSetter),
    #[darling(rename = "no")]
    NotSettable,
}

// TODO give this a better name
#[derive(Debug, FromMeta, Default)]
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
    doc: Option<String>,
    strip_option: Flag,
}

#[derive(Debug, FromMeta)]
struct BuildFnInfo {
    name: Option<Ident>,
    into: Flag,
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
            // create build method
            if let Some(build_fn) = &self.build_fn {
                let build_fn_name = match &build_fn.name {
                    Some(ident) => ident.clone(),
                    None => format_ident!("build"),
                };
                let build_fn_field_sets: Vec<_> = fields
                    .iter()
                    .map(|BuilderFieldReceiver { ident, .. }| quote!{ #ident: self.#ident })
                    .collect();
                let mut build_fn_generic_params = Vec::new();
                let mut build_fn_where_clauses = Vec::new();
                let mut build_fn_return_type = quote!{ #self_ident };
                let mut build_fn_body = quote!{
                    #self_ident {
                        #(#build_fn_field_sets),*
                    }
                };
                if build_fn.into.is_present() {
                    build_fn_generic_params.push(quote!{ T });
                    build_fn_where_clauses.push(quote!{ #build_fn_return_type: ::core::convert::Into<T> });
                    build_fn_return_type = quote!{ T };
                    build_fn_body = quote!{ (#build_fn_body).into() }
                }
                ret.extend(quote! {
                    impl #builder_class {
                        pub fn #build_fn_name<#(#build_fn_generic_params),*>(self) -> #build_fn_return_type
                        where #(#build_fn_where_clauses),*
                        {
                            #build_fn_body
                        }
                    }
                });
            }
        } else if self.build_fn.is_some() {
            // TODO span this properly
            return Err(darling::Error::custom("build_fn not yet supported without builder_class"));
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
        let ctor_visibility = match &self.ctor_options.visibility {
            Some(visibility) => quote!{ #visibility },
            None => quote!{ pub },
        };
        // setters
        let mut setter_funcs = Vec::new();
        //
        for field in fields {
            let ident = &field.ident;
            let field_type = &field.ty;
            match &field.set_mode {
                BuilderFieldSetMode::Ctor(ctor) => {
                    let ctor_into = match ctor { // TODO
                        darling::util::Override::Inherit => BuilderFieldSetViaCtor::default().into,
                        darling::util::Override::Explicit(c) => c.into,
                    };
                    if ctor_into.is_present() {
                        let prefixed = format_ident!("T{}", &cur_ctor_typevar_num);
                        ctor_params.push(quote!{ #ident: #prefixed });
                        ctor_generic_params.push(quote!{ #prefixed });
                        ctor_where_clauses.push(quote!{ #prefixed: ::core::convert::Into::<#field_type> });
                        ctor_self_field_sets.push(quote!{ #ident: #ident.into() });
                        cur_ctor_typevar_num += 1;
                    }
                    else {
                        ctor_params.push(quote!{ #ident: #field_type });
                        ctor_self_field_sets.push(quote!{ #ident: #ident });
                    }
                },
                BuilderFieldSetMode::Setter(setter) => {
                    // TODO ough rewrite this part
                    let mut arg_type = setter.arg_type.clone();
                    let mut wrap_with = setter.wrap_with.clone();
                    if setter.strip_option.is_present() {
                        // TODO do proper errors for these rather than just panicking
                        // TODO wait should we be prepending or appending do wrap_with
                        assert!(arg_type.is_none());
                        // assert!(wrap_with.is_empty());
                        arg_type = Some(strip_option_from(field_type)?);
                        wrap_with.push(syn::parse_quote!(::core::option::Option::Some));
                    }
                    // ctor-related stuff
                    ctor_self_field_sets.push(quote!{ #ident: ::core::default::Default::default() });
                    // build the setter(s?)
                    let mut setter_where_clauses = Vec::new();
                    let mut setter_type_vars = Vec::new();
                    let mut setter_val = quote!{ #ident };
                    if setter.into.is_present() {
                        setter_val = quote!{ #setter_val.into() };
                    }
                    for w in &wrap_with {
                        setter_val = quote!{ (#w)(#setter_val) };
                    }
                    let mut setter_arg_type = if let Some(ty) = arg_type {
                        quote!{ #ty }
                    } else {
                        quote!{ #field_type }
                    };
                    if setter.into.is_present() {
                        let typevar = quote!{T};
                        setter_type_vars.push(typevar.clone());
                        setter_where_clauses.push(quote!{ #typevar: ::core::convert::Into::<#setter_arg_type> });
                        setter_arg_type = typevar;
                    }
                    let setter_doc_attr = match &setter.doc {
                        Some(doc) => quote!{ #[doc = #doc] },
                        None => quote!{},
                    };
                    setter_funcs.push(quote!{
                        #[allow(clippy::missing_const_for_fn)]
                        #[must_use]
                        #setter_doc_attr
                        pub fn #ident <#(#setter_type_vars),*>(mut self, #ident: #setter_arg_type) -> Self
                        where #(#setter_where_clauses),*
                        {
                            self.#ident = #setter_val;
                            self
                        }
                    });
                },
                BuilderFieldSetMode::NotSettable => {
                    ctor_self_field_sets.push(quote!{ #ident: ::core::default::Default::default() });
                }, 
            }
        }

        let ident = &self.ident;
        ret.extend(quote! {
            impl #ident {
                #ctor_visibility fn #ctor_func_name <#(#ctor_generic_params),*>(#(#ctor_params),*) -> #ctor_return_type
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

// TODO ough implement this better
fn strip_option_from(ty: &syn::Type) -> Result<syn::Type, darling::Error> {
    let a = ty.to_token_stream()
        .into_iter()
        .collect::<Vec<_>>();

    if a.len() >= 4 {
        use proc_macro2::TokenTree;
        let first = a.get(0).unwrap();
        let next = a.get(1).unwrap();
        let last = a.last().unwrap();
        if let (TokenTree::Ident(first), TokenTree::Punct(next), TokenTree::Punct(last)) =
            (first, next, last)
        {
            let first = first.to_string();
            let next = next.as_char();
            let last = last.as_char();
            if first == "Option" && next == '<' && last == '>' {
                let a = &a[2..a.len()-1];
                let r: syn::Type = syn::parse_quote!(#(#a)*);
                return Ok(r);
            }
        }
    }

    Err(darling::Error::custom("unable to strip Option from provided type").with_span(&ty.span()))
}
