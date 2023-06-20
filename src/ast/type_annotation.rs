use log::trace;
use pest::iterators::Pair;

use super::{Position, Rule, Type};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct TypeAnnotation {
    pub value: Type,
    pub position: Position,
}

impl TypeAnnotation {
    pub fn from_pair(pair: Pair<Rule>, file: &str) -> TypeAnnotation {
        trace!("creating TypeAnnotation from pair '{pair}'");

        let (line, col) = pair.line_col();

        let mut inner = pair.into_inner();

        TypeAnnotation {
            value: Type::from_pair(inner.next().unwrap()),
            position: (file.to_owned(), line, col),
        }
    }
}
