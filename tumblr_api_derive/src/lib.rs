use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse_macro_input, spanned::Spanned, DeriveInput, Error, parenthesized};

#[proc_macro_derive(Builder, attributes(builder))]
pub fn builder_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match impl_builder_derive(&input) {
        Ok(token_stream) => token_stream,
        Err(err) => err.to_compile_error().into(),
    }

    // let name = input.ident;

    // let expanded = quote! {
    //     impl #name {
    //         // ...
    //     }
    // };

    // TokenStream::from(expanded)
}

#[derive(Debug, PartialEq, Eq)]
struct BuilderFieldInfo {
    mode: BuilderFieldSetMode,
}

#[derive(Debug, PartialEq, Eq)]
enum BuilderFieldSetMode {
    Ctor(BuilderFieldSetViaCtor),
}

// TODO give this a better name
#[derive(Debug, PartialEq, Eq)]
struct BuilderFieldSetViaCtor {
    into: bool,
}

fn impl_builder_derive(ast: &DeriveInput) -> Result<TokenStream, Error> {
    match &ast.data {
        syn::Data::Enum(_) => Err(Error::new(ast.span(), "not implemented for enums")),
        syn::Data::Union(_) => Err(Error::new(ast.span(), "not implemented for unions")),
        syn::Data::Struct(data) => {
            // let mut ctor_args = Vec::new();
            let mut builder_funcs = Vec::new();
            // handle top-level attrs
            for attr in &ast.attrs {
                if attr.path().is_ident("builder") {
                    attr.parse_nested_meta(|meta| {
                        Err(meta.error("unknown argument"))
                    })?;
                }
            }
            // handle fields
            let mut field_infos = Vec::new();
            for field in &data.fields {
                let mut info = None;
                // handle field attrs
                for attr in &field.attrs {
                    if attr.path().is_ident("builder") {
                        if info.is_some() {
                            return Err(Error::new(attr.span(), "already attached a builder attr to this field"));
                        }
                        let mut mode = None;
                        attr.parse_nested_meta(|meta| {
                            if meta.path.is_ident("ctor") {
                                // let mut do_into = None;
                                let content;
                                parenthesized!(content in meta.input);
                                mode = Some(BuilderFieldSetMode::Ctor(BuilderFieldSetViaCtor { into: true }));
                                return Ok(());
                            }
                            if meta.path.is_ident("setter") {
                                return Ok(());
                            }
                            Err(meta.error(format!("unknown argument {}", meta.path.to_token_stream())))
                        })?;
                        let mode = mode.ok_or_else(|| Error::new(attr.span(), "builder attr must specify either ctor or setter"))?;
                        info = Some(BuilderFieldInfo {
                            mode,
                        });
                    }
                }
                let info = info.ok_or_else(|| Error::new(field.span(), "field missing a builder attribute"))?;
                field_infos.push(info);
            }
            // generate ctor
            // {
            //     let ctor_fields = 
            // }
            builder_funcs.push(quote!()); // TODO load bearing for type stuff currently. remove me later!
            //
            let name = &ast.ident;
            Ok(quote! {
                impl #name {
                    #(#builder_funcs)*
                }
            }.into())
            // Err(Error::new(ast.span(), format!("idk {}", &ast.attrs.len())))
        },
    }
    // todo!();
    // let name = ast.ident;
    // let expanded = quote! {
    //     impl #name {
    //         // ...
    //     }
    // };
}
