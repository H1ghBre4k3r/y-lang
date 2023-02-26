use pest::iterators::Pair;

use super::{CompilerDirective, Expression, Import, Intrinsic, Rule};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Statement<T> {
    Import(Import),
    Expression(Expression<T>),
    Intrinsic(Intrinsic<T>),
    CompilerDirective(CompilerDirective<T>),
}

impl Statement<()> {
    pub fn from_pair(pair: Pair<Rule>, file: &str) -> Statement<()> {
        match pair.as_rule() {
            Rule::importDirective => Statement::Import(Import::from_pair(pair, file)),
            Rule::declaration | Rule::definition | Rule::assignment => {
                Statement::Intrinsic(Intrinsic::from_pair(pair, file))
            }
            Rule::expr => Statement::Expression(Expression::from_pair(pair, file)),
            Rule::compiler_directive => {
                Statement::CompilerDirective(CompilerDirective::from_pair(pair, file))
            }
            rule => unreachable!("Can not parse rule {rule:?} as expression"),
        }
    }
}

impl<T> Statement<T>
where
    T: Clone + Default,
{
    pub fn info(&self) -> T {
        match self {
            Statement::Expression(expression) => expression.info(),
            Statement::Intrinsic(intrinsic) => intrinsic.info(),
            Statement::CompilerDirective(compiler_directive) => compiler_directive.info(),
            _ => unreachable!(),
        }
    }
}
