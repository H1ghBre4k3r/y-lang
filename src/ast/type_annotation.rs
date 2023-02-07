use pest::iterators::Pair;

use super::{Position, Rule, Type};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct TypeAnnotation {
    pub value: Type,
    pub position: Position,
}

impl TypeAnnotation {
    pub fn from_pair(pair: Pair<Rule>) -> TypeAnnotation {
        let position = pair.line_col();

        let mut inner = pair.into_inner();

        TypeAnnotation {
            value: Type::from_pair(inner.next().unwrap()),
            position,
        }
    }
}
