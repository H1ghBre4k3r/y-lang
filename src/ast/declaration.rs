use pest::iterators::Pair;

use super::{Ident, Position, Rule, TypeAnnotation};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Declaration {
    pub ident: Ident<()>,
    pub type_annotation: TypeAnnotation,
    pub position: Position,
}

impl Declaration {
    pub fn from_pair(pair: Pair<Rule>) -> Declaration {
        assert_eq!(pair.as_rule(), Rule::declaration);

        let position = pair.line_col();

        let mut inner = pair.into_inner();

        let ident = inner.next().unwrap();
        let ident = Ident::from_pair(ident);

        let type_annotation = inner.next().unwrap();
        let type_annotation = TypeAnnotation::from_pair(type_annotation);

        Declaration {
            position,
            ident,
            type_annotation,
        }
    }
}
