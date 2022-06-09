use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::{
    parse_macro_input, Data, DataStruct, DeriveInput, Field, Fields, GenericArgument, Ident,
    PathArguments, PathSegment, Type,
};

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
        let ty = unwrap_option(&field.ty);
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
        let ty = unwrap_option(&field.ty);
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
    let unwrapped = fields_map(fields, |field| {
        let ident = &field.ident;
        let err_msg = format!("{:?} is required", &ident);
        if is_option(&field.ty) {
            quote! {
                let #ident = self.#ident.clone();
            }
        } else {
            quote! {
                let #ident = if let Some(x) = &self.#ident {
                    x.clone()
                } else {
                    return Err(#err_msg.into());
                };
            }
        }
    });

    let idents = fields_map(fields, |field| {
        let ident = &field.ident;
        quote! { #ident, }
    });

    quote! {
        #unwrapped
        Ok(#target { #idents })
    }
}

fn is_option(ty: &Type) -> bool {
    first_path_segment(ty)
        .map(|seg| seg.ident == "Option")
        .unwrap_or(false)
}

fn unwrap_option(ty: &Type) -> &Type {
    first_path_segment(ty)
        .and_then(|seg| (seg.ident == "Option").then(|| &seg.arguments))
        .and_then(|args| first_generic_argument(args))
        .and_then(|arg| get_generic_type(arg))
        .unwrap_or(ty)
}

fn first_path_segment(ty: &Type) -> Option<&PathSegment> {
    match ty {
        Type::Path(path) => path.path.segments.first(),
        _ => None,
    }
}

fn first_generic_argument(args: &PathArguments) -> Option<&GenericArgument> {
    match args {
        PathArguments::AngleBracketed(args) => args.args.first(),
        _ => None,
    }
}

fn get_generic_type(arg: &GenericArgument) -> Option<&Type> {
    match arg {
        GenericArgument::Type(ty) => Some(ty),
        _ => None,
    }
}
