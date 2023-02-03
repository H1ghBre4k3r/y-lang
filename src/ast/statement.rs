use pest::iterators::Pair;

use super::{Expression, Intrinsic, Rule};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Statement {
    Expression(Expression),
    Intrinsic(Intrinsic),
}

impl Statement {
    pub fn from_pair(pair: Pair<Rule>) -> Statement {
        match pair.as_rule() {
            Rule::declaration | Rule::assignment => {
                Statement::Intrinsic(Intrinsic::from_pair(pair))
            }
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
