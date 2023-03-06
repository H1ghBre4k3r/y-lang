use super::{Expression, Position, Rule, Statement};
use pest::iterators::Pair;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ConditionalStatement<T> {
    pub directive: Expression<()>,
    pub statement: Box<Statement<T>>,
    pub position: Position,
    pub is_valid: bool,
}

impl ConditionalStatement<()> {
    pub fn from_pair(pair: Pair<Rule>, file: &str) -> ConditionalStatement<()> {
        assert_eq!(pair.as_rule(), Rule::conditional_statement);

        let (line, col) = pair.line_col();

        let mut inner = pair.into_inner();
        let directive = inner.next().unwrap();
        let directive = Expression::from_pair(directive, file);

        let statement = inner.next().unwrap();
        let statement = Statement::from_pair(statement, file);

        ConditionalStatement {
            directive,
            statement: Box::new(statement),
            position: (file.to_owned(), line, col),
            is_valid: true,
        }
    }
}

impl<T> ConditionalStatement<T>
where
    T: Clone + Default,
{
    pub fn info(&self) -> T {
        self.statement.info()
    }
}
