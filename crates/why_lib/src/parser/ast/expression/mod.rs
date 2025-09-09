mod array;
mod binary;
mod block;
mod character;
mod function;
mod id;
mod if_expression;
mod lambda;
mod num;
mod postfix;
mod prefix;
mod string;
mod struct_initialisation;

pub use self::array::*;
pub use self::binary::*;
pub use self::block::*;
pub use self::character::*;
pub use self::function::*;
pub use self::id::*;
pub use self::if_expression::*;
pub use self::lambda::*;
pub use self::num::*;
pub use self::postfix::*;
pub use self::prefix::*;
pub use self::string::*;
pub use self::struct_initialisation::*;

use crate::grammar;
use crate::grammar::FromGrammar;
use crate::lexer::Span;

use super::AstNode;

// TODO: introduce Expression::Bool(_)
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Expression<T> {
    Id(Id<T>),
    Num(Num<T>),
    Character(Character<T>),
    AstString(AstString<T>),
    Function(Function<T>),
    Lambda(Lambda<T>),
    If(If<T>),
    Block(Block<T>),
    Parens(Box<Expression<T>>),
    Postfix(Postfix<T>),
    Prefix(Prefix<T>),
    Binary(Box<BinaryExpression<T>>),
    Array(Array<T>),
    StructInitialisation(StructInitialisation<T>),
}

impl<T> Expression<T>
where
    T: Clone,
{
    pub fn get_info(&self) -> T {
        match self {
            Expression::Id(Id { name: _, info, .. }) => info.clone(),
            Expression::Num(num) => num.get_info(),
            Expression::Character(Character { info, .. }) => info.clone(),
            Expression::AstString(AstString { info, .. }) => info.clone(),
            Expression::Function(Function { info, .. }) => info.clone(),
            Expression::Lambda(Lambda { info, .. }) => info.clone(),
            Expression::If(If { info, .. }) => info.clone(),
            Expression::Block(Block { info, .. }) => info.clone(),
            Expression::Parens(expr) => expr.get_info(),
            Expression::Postfix(postfix) => postfix.get_info(),
            Expression::Prefix(prefix) => prefix.get_info(),
            Expression::Binary(binary) => binary.get_info(),
            Expression::Array(arr) => arr.get_info(),
            Expression::StructInitialisation(StructInitialisation { info, .. }) => info.clone(),
        }
    }

    pub fn position(&self) -> Span {
        match self {
            Expression::Id(Id { position, .. }) => position.clone(),
            Expression::Num(num) => num.position(),
            Expression::Character(Character { position, .. }) => position.clone(),
            Expression::AstString(AstString { position, .. }) => position.clone(),
            Expression::Function(Function { position, .. }) => position.clone(),
            Expression::Lambda(Lambda { position, .. }) => position.clone(),
            Expression::If(If { position, .. }) => position.clone(),
            Expression::Block(Block { position, .. }) => position.clone(),
            Expression::Parens(expr) => expr.position(),
            Expression::Postfix(postfix_expr) => postfix_expr.position(),
            Expression::Prefix(prefix_expr) => prefix_expr.position(),
            Expression::Binary(binary_exp) => binary_exp.position(),
            Expression::Array(arr) => arr.position(),
            Expression::StructInitialisation(StructInitialisation { position, .. }) => {
                position.clone()
            }
        }
    }
}

impl FromGrammar<grammar::Expression> for Expression<()> {
    fn transform(item: rust_sitter::Spanned<grammar::Expression>, source: &str) -> Self {
        let rust_sitter::Spanned { value, span: _ } = item;

        match value {
            grammar::Expression::Identifier(identifier) => {
                Expression::Id(Id::transform(identifier, source))
            }
            grammar::Expression::Number(number) => Expression::Num(Num::transform(number, source)),
            grammar::Expression::String(string_literal) => {
                Expression::AstString(AstString::transform(string_literal, source))
            }
            grammar::Expression::Character(character_literal) => {
                Expression::Character(Character::transform(character_literal, source))
            }
            grammar::Expression::IfExpression(if_expression) => {
                Expression::If(If::transform(if_expression, source))
            }
            grammar::Expression::Parenthesized(parenthesized_expression) => {
                Expression::Parens(Box::new(Expression::transform(
                    *parenthesized_expression.value.inner,
                    source,
                )))
            }
            grammar::Expression::BinaryExpression(binary_expression) => Expression::Binary(
                Box::new(BinaryExpression::transform(binary_expression, source)),
            ),
            grammar::Expression::Block(block) => Expression::Block(Block::transform(block, source)),
            grammar::Expression::Lambda(lambda) => {
                Expression::Lambda(Lambda::transform(lambda, source))
            }
            grammar::Expression::Postfix(postfix) => {
                Expression::Postfix(Postfix::transform(postfix, source))
            }
            grammar::Expression::Prefix(prefix) => {
                Expression::Prefix(Prefix::transform(prefix, source))
            }
            grammar::Expression::Array(array) => Expression::Array(Array::transform(array, source)),
            grammar::Expression::StructInitialisation(struct_initialisation) => {
                Expression::StructInitialisation(StructInitialisation::transform(
                    struct_initialisation,
                    source,
                ))
            }
        }
    }
}

impl From<Expression<()>> for AstNode {
    fn from(value: Expression<()>) -> Self {
        AstNode::Expression(value)
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::ast::{BinaryOperator, Expression};
    use crate::parser::test_helpers::*;

    #[test]
    fn test_parse_basic_expressions() {
        // Test identifier
        let result = parse_expression("some_id").unwrap();
        assert!(matches!(result, Expression::Id(ref id) if id.name == "some_id"));

        // Test number
        let result = parse_expression("42").unwrap();
        assert!(matches!(
            result,
            Expression::Num(crate::parser::ast::Num::Integer(42, (), _))
        ));

        // Test string
        let result = parse_expression("\"hello\"").unwrap();
        assert!(matches!(result, Expression::AstString(_)));
    }

    #[test]
    fn test_parse_complex_expressions() {
        // Test binary expression
        let result = parse_expression("1 + 2").unwrap();
        if let Expression::Binary(binary) = result {
            assert!(matches!(binary.operator, BinaryOperator::Add));
        } else {
            panic!("Expected binary expression");
        }

        // Test function call
        let result = parse_expression("foo()").unwrap();
        assert!(matches!(result, Expression::Postfix(_)));

        // Test array literal
        let result = parse_expression("&[1, 2, 3]").unwrap();
        assert!(matches!(result, Expression::Array(_)));
    }
}
