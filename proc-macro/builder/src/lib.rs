#![feature(if_let_guard)]
#![feature(let_chains)]

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::{
    parse_macro_input, Attribute, Data, DataStruct, DeriveInput, Expr, ExprAssign, Field,
    GenericArgument, Ident, Lit, PathArguments, Type,
};

#[proc_macro_derive(Builder, attributes(builder))]
pub fn derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match input.data {
        Data::Struct(ref data) => StructBuilder::derive(&input, data),
        _ => unimplemented!(),
    }
}

struct StructBuilder<'a> {
    input: &'a DeriveInput,
    fields: Vec<FieldType<'a>>,
}

impl<'a> StructBuilder<'a> {
    fn derive(input: &DeriveInput, data: &DataStruct) -> TokenStream {
        let generator = StructBuilder::analyze(input, data);
        generator.generate()
    }

    fn analyze(input: &'a DeriveInput, data: &'a DataStruct) -> Self {
        let fields = data.fields.iter().map(FieldType::new).collect();
        Self { input, fields }
    }

    fn generate(&self) -> TokenStream {
        let target = &self.input.ident;
        let builder = format_ident!("{}Builder", target);

        let partial = self.gen_partial();
        let init = self.gen_init();
        let setters = self.gen_setters();
        let build = self.gen_build();

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

    fn gen_partial(&self) -> TokenStream2 {
        fields_map(&self.fields, |field| match field {
            FieldType::Normal { ident, ty } => quote! { #ident: std::option::Option<#ty>, },
            FieldType::Option { ident, ty } => quote! { #ident: std::option::Option<#ty>, },
            FieldType::Each { ident, ty, .. } => quote! { #ident: std::vec::Vec<#ty>, },
        })
    }

    fn gen_init(&self) -> TokenStream2 {
        fields_map(&self.fields, |field| match field {
            FieldType::Normal { ident, .. } => quote! { #ident: None, },
            FieldType::Option { ident, .. } => quote! { #ident: None, },
            FieldType::Each { ident, .. } => quote! { #ident: Vec::new(), },
        })
    }

    fn gen_setters(&self) -> TokenStream2 {
        fields_map(&self.fields, |field| match field {
            FieldType::Normal { ident, ty } => quote! {
                pub fn #ident(&mut self, #ident: #ty) -> &mut Self {
                    self.#ident = Some(#ident);
                    self
                }
            },
            FieldType::Option { ident, ty } => quote! {
                pub fn #ident(&mut self, #ident: #ty) -> &mut Self {
                    self.#ident = Some(#ident);
                    self
                }
            },
            FieldType::Each { ident, ty, each } => quote! {
                pub fn #each(&mut self, #each: #ty) -> &mut Self {
                    self.#ident.push(#each);
                    self
                }
            },
        })
    }

    fn gen_build(&self) -> TokenStream2 {
        let unwrapped = fields_map(&self.fields, |field| {
            let ident = field.ident();
            let err_msg = format!("{:?} is required", &ident);

            match field {
                FieldType::Normal { ident, .. } => quote! {
                    let #ident = if let Some(x) = &self.#ident {
                        x.clone()
                    } else {
                        return Err(#err_msg.into());
                    };
                },
                FieldType::Option { ident, .. } => quote! {
                   let #ident = self.#ident.clone();
                },
                FieldType::Each { ident, .. } => quote! {
                   let #ident = self.#ident.clone();
                },
            }
        });

        let target = &self.input.ident;
        let idents = fields_map(&self.fields, |field| {
            let ident = field.ident();
            quote! { #ident, }
        });

        quote! {
            #unwrapped

            Ok(#target {
                #idents
            })
        }
    }
}

enum FieldType<'a> {
    Normal {
        ident: &'a Option<Ident>,
        ty: &'a Type,
    },
    Option {
        ident: &'a Option<Ident>,
        ty: &'a Type,
    },
    Each {
        ident: &'a Option<Ident>,
        ty: &'a Type,
        each: Ident,
    },
}

impl<'a> FieldType<'a> {
    fn new(field: &'a Field) -> Self {
        Self::check_option(field)
            .or(Self::check_vec(field))
            .unwrap_or({
                let (ident, ty) = (&field.ident, &field.ty);
                Self::Normal { ident, ty }
            })
    }

    fn check_option(field: &'a Field) -> Option<Self> {
        let (ident, ty) = (&field.ident, &field.ty);
        first_generic_arg(ty, "Option").map(|ty| Self::Option { ident, ty })
    }

    fn check_vec(field: &'a Field) -> Option<Self> {
        let (ident, ty) = (&field.ident, &field.ty);
        first_generic_arg(ty, "Vec").map(|arg_type| {
            if let Some(attr) = field.attrs.first() {
                if let Some(each) = attribute_each(attr) {
                    return Self::Each {
                        ident,
                        ty: arg_type, // TODO
                        each,
                    };
                }
            }
            Self::Normal { ident, ty }
        })
    }

    fn ident(&self) -> &'a Option<Ident> {
        match self {
            Self::Normal { ident, .. } => ident,
            Self::Option { ident, .. } => ident,
            Self::Each { ident, .. } => ident,
        }
    }
}

fn fields_map<F>(fields: &[FieldType], f: F) -> TokenStream2
where
    F: FnMut(&FieldType) -> TokenStream2,
{
    fields.iter().flat_map(f).collect()
}

fn first_generic_arg<'a>(target: &'a Type, ty: &str) -> Option<&'a Type> {
    match target {
        Type::Path(path) => path.path.segments.first(),
        _ => None,
    }
    .and_then(|seg| (seg.ident == ty).then(|| &seg.arguments))
    .and_then(|args| match args {
        PathArguments::AngleBracketed(args) => args.args.first(),
        _ => None,
    })
    .and_then(|arg| match arg {
        GenericArgument::Type(ty) => Some(ty),
        _ => None,
    })
}

// TODO
fn attribute_each(attr: &Attribute) -> Option<Ident> {
    attr.parse_args::<ExprAssign>()
        .ok()
        .and_then(|ExprAssign { left, right, .. }| {
            if let Expr::Path(path) = *left
            && let Some(ident) = path.path.segments.first().map(|seg| &seg.ident)
            && ident == "each"
            {
            } else {
                return None
            }

            if let Expr::Lit(lit) = *right
            && let Lit::Str(lit) = lit.lit
            {
                Some(format_ident!("{}", lit.value()))
            } else {
                None
            }
        })
}
