mod expression;
mod statement;

pub use self::expression::*;
pub use self::statement::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AstNode {
    Expression(Expression),
    Id(Id),
    Num(Num),
    Statement(Statement),
    Initialization(Initialization),
    Function(Function),
    Parameter(Parameter),
}
