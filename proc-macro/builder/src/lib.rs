use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::{parse_macro_input, Data, DataStruct, DeriveInput, Field, Fields, Ident};

#[proc_macro_derive(Builder)]
pub fn derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match input.data {
        Data::Struct(ref data) => struct_builder(&input, data),
        _ => unimplemented!(),
    }
}

fn struct_builder(input: &DeriveInput, data: &DataStruct) -> TokenStream {
    let target = &input.ident;
    let builder = format_ident!("{}Builder", target);

    let partial = partial(&data.fields);
    let init = init(&data.fields);
    let setters = setters(&data.fields);
    let build = build(&target, &data.fields);

    TokenStream::from(quote! {
        impl #target {
            pub fn builder() -> #builder {
                #builder {
                    #init
                }
            }
        }

        pub struct #builder {
            #partial
        }

        impl #builder {
            #setters

            pub fn build(&mut self)
                -> std::result::Result<#target, std::boxed::Box<dyn std::error::Error>>
            {
                #build
            }
        }
    })
}

fn fields_map<F>(fields: &Fields, f: F) -> TokenStream2
where
    F: FnMut(&Field) -> TokenStream2,
{
    fields.iter().map(f).flatten().collect()
}

fn partial(fields: &Fields) -> TokenStream2 {
    fields_map(fields, |field| {
        let ty = &field.ty;
        let ident = &field.ident;
        quote! { #ident: std::option::Option<#ty>, }
    })
}

fn init(fields: &Fields) -> TokenStream2 {
    fields_map(fields, |field| {
        let ident = &field.ident;
        quote! { #ident: None, }
    })
}

fn setters(fields: &Fields) -> TokenStream2 {
    fields_map(fields, |field| {
        let ty = &field.ty;
        let ident = &field.ident;
        quote! {
            pub fn #ident(&mut self, #ident: #ty) -> &mut Self {
                self.#ident = Some(#ident);
                self
            }
        }
    })
}

fn build(target: &Ident, fields: &Fields) -> TokenStream2 {
    let idents = fields_map(fields, |field| {
        let ident = &field.ident;
        quote! { #ident, }
    });

    let self_idents = fields_map(fields, |field| {
        let ident = &field.ident;
        quote! { &self.#ident, }
    });

    let some_idents = fields_map(fields, |field| {
        let ident = &field.ident;
        quote! { Some(#ident), }
    });

    let clone_idents = fields_map(fields, |field| {
        let ident = &field.ident;
        quote! { #ident.clone(), }
    });

    quote! {
        let (#idents) = match (#self_idents) {
            (#some_idents) => (#clone_idents),
            _ => return Err("not set".into()),
        };

        Ok(#target { #idents })
    }
}
