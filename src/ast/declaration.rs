use pest::iterators::Pair;

use super::{Expression, Ident, Position, Rule};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Declaration {
    pub ident: Ident,
    pub value: Expression,
    pub position: Position,
}

impl Declaration {
    pub fn from_pair(pair: Pair<Rule>) -> Declaration {
        let mut inner = pair.clone().into_inner();

        let ident = Ident::from_pair(inner.next().unwrap_or_else(|| {
            panic!(
                "Expected lvalue in declaration '{}' at {}:{}",
                pair.as_str(),
                pair.line_col().0,
                pair.line_col().1
            )
        }));

        let value = inner.next().unwrap_or_else(|| {
            panic!(
                "Expected rvalue in declaration '{}' at {}:{}",
                pair.as_str(),
                pair.line_col().0,
                pair.line_col().1
            )
        });
        let value = Expression::from_pair(value);

        Declaration {
            ident,
            value,
            position: pair.line_col(),
        }
    }
}
