use pest::iterators::Pair;

use super::{Position, Rule};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Import {
    pub path: String,
    pub position: Position,
    // pub document: Option<Box<>>,
}

impl Import {
    pub fn from_pair(pair: Pair<Rule>, file: &str) -> Self {
        assert_eq!(pair.as_rule(), Rule::importDirective);
        let (line, col) = pair.line_col();

        let mut inner = pair.into_inner();

        let input_path = inner.next().unwrap_or_else(|| {
            panic!("No valid input path given!");
        });

        let path = input_path.as_str();

        Import {
            path: path.to_owned(),
            position: (file.to_owned(), line, col),
        }
    }
}
