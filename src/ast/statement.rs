use pest::iterators::Pair;

use super::{Expression, Import, Intrinsic, Rule};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Statement<T> {
    Import(Import),
    Expression(Expression<T>),
    Intrinsic(Intrinsic<T>),
}

impl Statement<()> {
    pub fn from_pair(pair: Pair<Rule>, file: &str) -> Statement<()> {
        match pair.as_rule() {
            Rule::importStmt => Statement::Import(Import::from_pair(pair, file)),
            Rule::declaration | Rule::definition | Rule::assignment => {
                Statement::Intrinsic(Intrinsic::from_pair(pair, file))
            }
            Rule::ifStmt
            | Rule::binaryExpr
            | Rule::fnDef
            | Rule::integer
            | Rule::ident
            | Rule::string
            | Rule::fnCall
            | Rule::block
            | Rule::boolean => Statement::Expression(Expression::from_pair(pair, file)),
            _ => todo!(),
        }
    }
}

impl<T> Statement<T>
where
    T: Clone,
{
    pub fn info(&self) -> T {
        match self {
            Statement::Expression(expression) => expression.info(),
            Statement::Intrinsic(intrinsic) => intrinsic.info(),
            _ => unreachable!(),
        }
    }
}
