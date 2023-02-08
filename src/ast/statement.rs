use pest::iterators::Pair;

use super::{Expression, Intrinsic, Rule};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Statement<T> {
    Expression(Expression<T>),
    Intrinsic(Intrinsic<T>),
}

impl Statement<()> {
    pub fn from_pair(pair: Pair<Rule>) -> Statement<()> {
        match pair.as_rule() {
            Rule::definition | Rule::assignment => Statement::Intrinsic(Intrinsic::from_pair(pair)),
            Rule::ifStmt
            | Rule::binaryExpr
            | Rule::fnDef
            | Rule::integer
            | Rule::ident
            | Rule::string
            | Rule::fnCall
            | Rule::block
            | Rule::boolean => Statement::Expression(Expression::from_pair(pair)),
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
        }
    }
}
