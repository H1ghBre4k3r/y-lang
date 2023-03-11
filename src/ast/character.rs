use pest::iterators::Pair;

use super::{Position, Rule};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Character<T> {
    pub value: char,
    pub position: Position,
    pub info: T,
}

impl Character<()> {
    pub fn from_pair(pair: Pair<Rule>, file: &str) -> Character<()> {
        assert_eq!(pair.as_rule(), Rule::character);
        let (line, col) = pair.line_col();

        Character {
            value: pair
                .into_inner()
                .next()
                .unwrap()
                .as_str()
                .parse::<char>()
                .unwrap(),
            position: (file.to_owned(), line, col),
            info: (),
        }
    }
}
