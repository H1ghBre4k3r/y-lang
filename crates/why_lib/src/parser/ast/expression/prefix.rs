use crate::{
    grammar::{self, FromGrammar},
    lexer::Span,
};

use super::Expression;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Prefix<T> {
    Negation {
        expr: Box<Expression<T>>,
        position: Span,
    },
    Minus {
        expr: Box<Expression<T>>,
        position: Span,
    },
}

impl<T> Prefix<T>
where
    T: Clone,
{
    pub fn get_info(&self) -> T {
        match self {
            Prefix::Negation { expr, .. } => expr.get_info(),
            Prefix::Minus { expr, .. } => expr.get_info(),
        }
    }

    pub fn position(&self) -> Span {
        match self {
            Prefix::Negation { position, .. } => position.clone(),
            Prefix::Minus { position, .. } => position.clone(),
        }
    }
}

impl FromGrammar<grammar::Prefix> for Prefix<()> {
    fn transform(item: rust_sitter::Spanned<grammar::Prefix>, source: &str) -> Self {
        let rust_sitter::Spanned { value, span } = item;
        
        match value {
            grammar::Prefix::Negation { expression, .. } => {
                Prefix::Negation {
                    expr: Box::new(Expression::transform(*expression, source)),
                    position: Span::new(span, source),
                }
            }
            grammar::Prefix::Minus { expression, .. } => {
                Prefix::Minus {
                    expr: Box::new(Expression::transform(*expression, source)),
                    position: Span::new(span, source),
                }
            }
        }
    }
}
