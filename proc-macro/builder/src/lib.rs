use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};
use quote::quote;

#[proc_macro_derive(Builder)]
pub fn derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let ident = input.ident;
    let expanded = quote! {
        impl #ident {
            pub fn hello() {
                println!("Hello, world!");
            }
        }
    };
    TokenStream::from(expanded)
}
