use super::{Expression, Position, Rule, Statement};
use pest::iterators::Pair;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct CompilerDirective<T> {
    pub directive: Expression<()>,
    pub statement: Option<Box<Statement<T>>>,
    pub position: Position,
}

impl CompilerDirective<()> {
    pub fn from_pair(pair: Pair<Rule>, file: &str) -> CompilerDirective<()> {
        assert_eq!(pair.as_rule(), Rule::compiler_directive);

        let (line, col) = pair.line_col();

        let mut inner = pair.into_inner();
        let directive = inner.next().unwrap();
        let directive = Expression::from_pair(directive, file);

        let statement = inner.next().unwrap();
        let statement = Statement::from_pair(statement, file);

        CompilerDirective {
            directive,
            statement: Some(Box::new(statement)),
            position: (file.to_owned(), line, col),
        }
    }
}

impl<T> CompilerDirective<T>
where
    T: Clone + Default,
{
    pub fn info(&self) -> T {
        match &self.statement {
            Some(statement) => statement.info(),
            _ => T::default(),
        }
    }
}
