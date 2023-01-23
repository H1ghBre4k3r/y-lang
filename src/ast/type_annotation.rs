use pest::iterators::Pair;

use super::{Position, Rule};

#[derive(Debug, Clone)]
pub struct TypeAnnotation {
    pub value: String,
    pub position: Position,
}

impl TypeAnnotation {
    pub fn from_pair(pair: Pair<Rule>) -> TypeAnnotation {
        assert_eq!(pair.as_rule(), Rule::typeAnnotation);

        let position = pair.line_col();

        let mut inner = pair.into_inner();

        let type_pair = inner.next().unwrap();
        assert_eq!(type_pair.as_rule(), Rule::typeName);

        let type_name = type_pair.as_str();

        TypeAnnotation {
            value: type_name.to_owned(),
            position,
        }
    }
}
