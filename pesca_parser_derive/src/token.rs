use proc_macro::TokenStream;
use quote::quote;
use syn::{punctuated::Punctuated, token::Comma, DataEnum, DeriveInput, Variant};

pub fn impl_token_macro(ast: syn::DeriveInput) -> TokenStream {
    let DeriveInput { ident, data, .. } = ast;

    let syn::Data::Enum(DataEnum { variants, .. }) = data else {
        panic!()
    };

    let terminal_variants = variants
        .clone()
        .into_iter()
        .filter_map(|variant| {
            let Variant {
                attrs,
                ident,
                discriminant,
                ..
            } = variant;

            for attr in &attrs {
                let Some(attr_ident) = attr.path().get_ident() else {
                    continue;
                };
                if *attr_ident == "terminal" {
                    return Some(Variant {
                        attrs: vec![],
                        ident,
                        fields: syn::Fields::Unit,
                        discriminant,
                    });
                }
            }

            None
        })
        .collect::<Punctuated<Variant, Comma>>();

    let matches_terminal_enum = terminal_variants.iter().map(|variant| {
        let Variant {
            ident: var_ident, ..
        } = variant;
        quote! {
            (Terminal::#var_ident, #ident::#var_ident { .. }) => true,
        }
    });

    let matches_enum_terminal = terminal_variants.iter().map(|variant| {
        let Variant {
            ident: var_ident, ..
        } = variant;
        quote! {
            (#ident::#var_ident { .. }, Terminal::#var_ident) => true,
        }
    });

    let matches_to_token = terminal_variants.iter().map(|variant| {
        let Variant {
            ident: var_ident, ..
        } = variant;
        quote! {
            Terminal::#var_ident => #ident::#var_ident { position },
        }
    });

    let matches_get_position = variants.iter().map(|variant| {
        let Variant {
            ident: var_ident, ..
        } = variant;
        quote! {
            #ident::#var_ident { position, .. } => *position,
        }
    });

    let gen = quote! {
        #[derive(Debug, Clone, PartialEq, Eq)]
        pub enum Terminal {
            #terminal_variants
        }

        impl PartialEq<#ident> for Terminal {
            fn eq(&self, rhs: &#ident) -> bool {
                match (self, rhs) {
                    #(#matches_terminal_enum)*
                    _ => false
                }
            }
        }

        impl PartialEq<Terminal> for #ident {
            fn eq(&self, rhs: &Terminal) -> bool {
                match (self, rhs) {
                    #(#matches_enum_terminal)*
                    _ => false
                }
            }
        }

        impl Eq for #ident {}

        impl Terminal {
            pub fn to_token(&self, position: Position) -> #ident {
                match self {
                    #(#matches_to_token)*
                }
            }
        }

        impl #ident {
            pub fn position(&self) -> Position {
                match self {
                    #(#matches_get_position)*
                }
            }
        }
    };

    gen.into()
}
