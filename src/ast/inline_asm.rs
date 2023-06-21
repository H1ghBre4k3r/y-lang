use pest::iterators::Pair;
use tracing::trace;

use super::{Position, Rule};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct InlineAssembly<T> {
    pub statements: Vec<String>,
    pub position: Position,
    pub info: T,
}

impl InlineAssembly<()> {
    pub fn from_pair(pair: Pair<Rule>, file: &str) -> InlineAssembly<()> {
        trace!("creating InlineAssembly from pair '{pair}'");

        let (line, col) = pair.line_col();

        let mut inner = pair.into_inner();
        let raw_assembly = inner
            .next()
            .unwrap_or_else(|| panic!("Expected content in inline assembly"))
            .as_str();

        let assembly_statements = raw_assembly
            .lines()
            .map(|line| line.trim_matches(' ').to_owned())
            .collect::<Vec<_>>();

        InlineAssembly {
            statements: assembly_statements,
            position: (file.to_owned(), line, col),
            info: (),
        }
    }
}

impl<T> InlineAssembly<T>
where
    T: Clone,
{
    pub fn info(&self) -> T {
        self.info.clone()
    }
}
