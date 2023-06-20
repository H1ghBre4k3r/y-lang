use super::{Position, Rule};
use log::trace;
use pest::iterators::Pair;
use unescape::unescape;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Str<T> {
    pub value: String,
    pub position: Position,
    pub info: T,
}

impl Str<()> {
    pub fn from_pair(pair: Pair<Rule>, file: &str) -> Str<()> {
        assert_eq!(pair.as_rule(), Rule::string);
        trace!("creating Str from pair '{pair}'");

        let (line, col) = pair.line_col();

        Str {
            value: unescape(pair.clone().into_inner().next().unwrap().as_str())
                .expect("Invalid character escaped"),
            position: (file.to_string(), line, col),
            info: (),
        }
    }
}
