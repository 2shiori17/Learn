use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::{parse_macro_input, Data, DataStruct, DeriveInput, Fields};

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
    })
}

fn partial(fields: &Fields) -> TokenStream2 {
    fields
        .iter()
        .map(|field| {
            let ty = &field.ty;
            let ident = &field.ident;
            quote! { #ident: std::option::Option<#ty>, }
        })
        .flatten()
        .collect()
}

fn init(fields: &Fields) -> TokenStream2 {
    fields
        .iter()
        .map(|field| {
            let ident = &field.ident;
            quote! { #ident: None, }
        })
        .flatten()
        .collect()
}
