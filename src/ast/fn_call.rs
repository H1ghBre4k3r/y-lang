use pest::iterators::Pair;

use super::{Expression, Ident, Position, Rule};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct FnCall<T> {
    pub ident: Ident<T>,
    pub params: Vec<Expression<T>>,
    pub position: Position,
    pub info: T,
}

impl FnCall<()> {
    pub fn from_pair(pair: Pair<Rule>) -> FnCall<()> {
        assert_eq!(pair.as_rule(), Rule::fnCall);

        let position = pair.line_col();

        let mut inner = pair.into_inner();

        let ident = inner.next().unwrap();

        let mut params = vec![];

        for param in inner {
            params.push(Expression::from_pair(param));
        }

        FnCall {
            ident: Ident::from_pair(ident),
            params,
            position,
            info: (),
        }
    }
}
