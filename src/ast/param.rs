use pest::iterators::Pair;

use super::{Ident, Position, Rule, TypeAnnotation};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Param<T> {
    pub ident: Ident<T>,
    pub type_annotation: TypeAnnotation,
    pub position: Position,
}

impl Param<()> {
    pub fn from_pair(pair: Pair<Rule>, file: &str) -> Param<()> {
        assert_eq!(pair.as_rule(), Rule::parameter);

        let (line, col) = pair.line_col();

        let mut inner = pair.into_inner();

        let ident = inner.next().unwrap();
        let ident = Ident::from_pair(ident, file);

        let type_annotation = inner.next().unwrap();
        let type_annotation = TypeAnnotation::from_pair(type_annotation, file);

        Param {
            ident,
            type_annotation,
            position: (file.to_owned(), line, col),
        }
    }
}

impl<T> Param<T>
where
    T: Clone,
{
    pub fn info(&self) -> T {
        self.ident.info.clone()
    }
}
