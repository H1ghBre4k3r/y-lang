use pest::iterators::Pair;

use super::{Expression, Ident, Position, Rule};

#[derive(Debug, Clone)]
pub struct FnCall {
    pub ident: Ident,
    pub params: Vec<Expression>,
    pub position: Position,
}

impl FnCall {
    pub fn from_pair(pair: Pair<Rule>) -> FnCall {
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
        }
    }
}