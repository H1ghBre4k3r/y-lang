use log::trace;
use pest::iterators::Pair;

use super::{Position, Rule};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Import {
    pub path: String,
    pub position: Position,
}

impl Import {
    pub fn from_pair(pair: Pair<Rule>, file: &str) -> Self {
        assert_eq!(pair.as_rule(), Rule::importDirective);
        trace!("creating Import from pair '{pair}'");

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

    pub fn is_wildcard(&self) -> bool {
        self.path.ends_with("::*")
    }
}
