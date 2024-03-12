use proc_macro::TokenStream;
use quote::quote;
use syn::{
    punctuated::Punctuated, token::Comma, DataEnum, DeriveInput, Expr, ExprLit, Lit, Variant,
};

pub fn impl_token_macro(ast: syn::DeriveInput) -> TokenStream {
    let DeriveInput { ident, data, .. } = ast;

    let syn::Data::Enum(DataEnum { variants, .. }) = data else {
        panic!()
    };

    let terminal_variants_tuples = variants
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

                let Ok(Expr::Lit(ExprLit {
                    lit: Lit::Str(literal),
                    ..
                })) = attr.parse_args::<Expr>()
                else {
                    panic!("missing matcher for #[terminal] {ident}");
                };

                if *attr_ident == "terminal" {
                    return Some((
                        Variant {
                            attrs: vec![],
                            ident,
                            fields: syn::Fields::Unit,
                            discriminant,
                        },
                        literal,
                    ));
                }
            }

            None
        })
        .collect::<Vec<_>>();

    let literal_variants_tuples = variants
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

                if *attr_ident != "literal" {
                    continue;
                }

                let Ok(Expr::Lit(ExprLit {
                    lit: Lit::Str(literal),
                    ..
                })) = attr.parse_args::<Expr>()
                else {
                    panic!("missing matcher for #[literal] {ident}");
                };

                return Some((
                    Variant {
                        attrs: vec![],
                        ident,
                        fields: syn::Fields::Unit,
                        discriminant,
                    },
                    literal,
                ));
            }

            None
        })
        .collect::<Vec<_>>();

    let matches_terminal_enum = terminal_variants_tuples.iter().map(|(variant, _)| {
        let Variant {
            ident: var_ident, ..
        } = variant;
        quote! {
            (Terminal::#var_ident, #ident::#var_ident { .. }) => true,
        }
    });

    let matches_enum_terminal = terminal_variants_tuples.iter().map(|(variant, _)| {
        let Variant {
            ident: var_ident, ..
        } = variant;
        quote! {
            (#ident::#var_ident { .. }, Terminal::#var_ident) => true,
        }
    });

    let matches_to_token = terminal_variants_tuples.iter().map(|(variant, _)| {
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

    let terminal_insertions = terminal_variants_tuples.iter().map(|(variant, literal)| {
        let Variant {
            ident: var_ident, ..
        } = variant;

        let literal = literal.value();

        quote! {
            terminal!(entries, #var_ident, #literal);
        }
    });

    let literal_insertions = literal_variants_tuples.iter().map(|(variant, literal)| {
        let Variant {
            ident: var_ident, ..
        } = variant;

        let literal = literal.value();

        quote! {
            literal!(entries, #var_ident, #literal);
        }
    });

    let terminal_variants_tuples = terminal_variants_tuples
        .iter()
        .map(|(variant, _)| variant.clone())
        .collect::<Punctuated<Variant, Comma>>();

    let gen = quote! {
        #[derive(Debug, Clone, PartialEq, Eq)]
        pub enum Terminal {
            #terminal_variants_tuples
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

        pub trait GetPosition {
            fn position(&self) -> Position;
        }

        impl GetPosition for #ident {
            fn position(&self) -> Position {
                match self {
                    #(#matches_get_position)*
                }
            }
        }

        macro_rules! terminal {
            ($entries:ident, $name:ident, $value:expr) => {
                Self::insert(
                    &mut $entries,
                    Regex::new(&$value.escape_unicode().to_string()).unwrap(),
                    |_, position| Token::$name { position },
                );
            };
        }

        macro_rules! literal {
            ($entries:ident, $name:ident, $value:expr) => {
                Self::insert(
                    &mut $entries,
                    Regex::new($value).unwrap(),
                    |matched, position| Token::$name {
                        position,
                        value: matched.as_str().parse().unwrap(),
                    },
                );
            };
        }

        type Entries = Vec<(Regex, Box<dyn Fn(Match, (usize, usize)) -> Token>)>;

        pub struct Lexikon {
            entries: Entries,
        }

        impl<'a> Lexikon {
            pub fn new() -> Lexikon {
                let mut entries = vec![];

                #(#terminal_insertions)*

                #(#literal_insertions)*

                Lexikon { entries }
            }

            fn insert<F: Fn(Match, (usize, usize)) -> Token + 'static>(entries: &mut Entries, reg: Regex, f: F) {
                entries.push((reg, Box::new(f)))
            }

            pub fn find_longest_match(
                &self,
                pattern: &'a str,
                position: (usize, usize),
            ) -> (usize, Option<Token>) {
                let mut longest = (0, None);

                for (reg, mapper) in &self.entries {
                    let Some(res) = reg.captures_at(pattern, 0).and_then(|res| res.get(0)) else {
                        continue;
                    };

                    let len = res.len();

                    if len > longest.0 && res.start() == 0 {
                        longest = (len, Some(mapper(res, position)));
                    }
                }

                longest
            }
        }


    };

    gen.into()
}
