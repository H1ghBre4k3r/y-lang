use super::Expression;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Postfix {
    Call {
        expr: Box<Expression>,
        args: Vec<Expression>,
    },
    Index {
        expr: Box<Expression>,
        index: Box<Expression>,
    },
}
