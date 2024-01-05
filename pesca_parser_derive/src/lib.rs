mod loose_eq;
mod token;

use loose_eq::*;
use token::*;

use proc_macro::TokenStream;

#[proc_macro_derive(Token, attributes(terminal, literal))]
pub fn derive_token(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();

    impl_token_macro(ast)
}

#[proc_macro_derive(LooseEq, attributes(terminal))]
pub fn derive_loose_eq(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();

    impl_loose_eq_macro(ast)
}
