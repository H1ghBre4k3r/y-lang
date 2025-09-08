use crate::{
    grammar::{self, FromGrammar},
    lexer::Span,
};

use super::{Expression, Id};

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Postfix<T> {
    Call {
        expr: Box<Expression<T>>,
        args: Vec<Expression<T>>,
        info: T,
        position: Span,
    },
    Index {
        expr: Box<Expression<T>>,
        index: Box<Expression<T>>,
        info: T,
        position: Span,
    },
    PropertyAccess {
        expr: Box<Expression<T>>,
        property: Id<T>,
        info: T,
        position: Span,
    },
}

impl<T> Postfix<T>
where
    T: Clone,
{
    pub fn get_info(&self) -> T {
        match self {
            Postfix::Call { info, .. } => info.clone(),
            Postfix::Index { info, .. } => info.clone(),
            Postfix::PropertyAccess { info, .. } => info.clone(),
        }
    }

    pub fn position(&self) -> Span {
        match self {
            Postfix::Call { position, .. } => position.clone(),
            Postfix::Index { position, .. } => position.clone(),
            Postfix::PropertyAccess { position, .. } => position.clone(),
        }
    }
}

impl FromGrammar<grammar::Postfix> for Postfix<()> {
    fn transform(item: rust_sitter::Spanned<grammar::Postfix>, source: &str) -> Self {
        let rust_sitter::Spanned { value, span } = item;

        match value {
            grammar::Postfix::Call(call_expr) => Postfix::Call {
                expr: Box::new(Expression::transform(*call_expr.expression, source)),
                args: call_expr
                    .args
                    .into_iter()
                    .map(|arg| Expression::transform(arg, source))
                    .collect(),
                info: (),
                position: Span::new(span, source),
            },
            grammar::Postfix::Index(index_expr) => Postfix::Index {
                expr: Box::new(Expression::transform(*index_expr.expression, source)),
                index: Box::new(Expression::transform(*index_expr.index, source)),
                info: (),
                position: Span::new(span, source),
            },
            grammar::Postfix::PropertyAccess(prop_access) => Postfix::PropertyAccess {
                expr: Box::new(Expression::transform(*prop_access.expression, source)),
                property: Id::transform(prop_access.property, source),
                info: (),
                position: Span::new(span, source),
            },
        }
    }
}
