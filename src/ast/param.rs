use pest::iterators::Pair;

use super::{Ident, Position, Rule, TypeAnnotation};

#[derive(Debug, Clone)]
pub struct Param {
    pub ident: Ident,
    pub type_annotation: TypeAnnotation,
    pub position: Position,
}

impl Param {
    pub fn from_pair(pair: Pair<Rule>) -> Param {
        assert_eq!(pair.as_rule(), Rule::parameter);

        let position = pair.line_col();

        let mut inner = pair.into_inner();

        let ident = inner.next().unwrap();
        let ident = Ident::from_pair(ident);

        let type_annotation = inner.next().unwrap();
        let type_annotation = TypeAnnotation::from_pair(type_annotation);

        Param {
            ident,
            type_annotation,
            position,
        }
    }
}
