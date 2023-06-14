use log::trace;
use pest::iterators::Pair;

use super::{Ident, Position, Rule, TypeAnnotation};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Declaration {
    pub ident: Ident<()>,
    pub type_annotation: TypeAnnotation,
    pub position: Position,
}

impl Declaration {
    pub fn from_pair(pair: Pair<Rule>, file: &str) -> Declaration {
        assert_eq!(pair.as_rule(), Rule::declaration);
        trace!("creating Declaration from pair '{pair:?}'");

        let (line, col) = pair.line_col();

        let mut inner = pair.into_inner();

        let ident = inner.next().unwrap();
        let ident = Ident::from_pair(ident, file);

        let type_annotation = inner.next().unwrap();
        let type_annotation = TypeAnnotation::from_pair(type_annotation, file);

        Declaration {
            position: (file.to_owned(), line, col),
            ident,
            type_annotation,
        }
    }
}
