use once_cell::sync::Lazy;
use pest::{
    iterators::Pair,
    pratt_parser::{Assoc, Op, PrattParser},
};

use super::{
    Array, BinaryExpr, Block, Boolean, Character, FnDef, Ident, If, Integer, Position, PostfixExpr,
    PrefixExpr, Rule, Str,
};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Expression<T> {
    If(If<T>),
    Binary(BinaryExpr<T>),
    Prefix(PrefixExpr<T>),
    Postfix(PostfixExpr<T>),
    Integer(Integer<T>),
    Character(Character<T>),
    Ident(Ident<T>),
    Str(Str<T>),
    FnDef(FnDef<T>),
    Block(Block<T>),
    Boolean(Boolean<T>),
    Array(Array<T>),
}

static PRATT_PARSER: Lazy<PrattParser<Rule>> = Lazy::new(|| {
    PrattParser::new()
        .op(Op::infix(Rule::lessThan, Assoc::Left)
            | Op::infix(Rule::greaterThan, Assoc::Left)
            | Op::infix(Rule::equal, Assoc::Left))
        .op(Op::infix(Rule::plus, Assoc::Left) | Op::infix(Rule::minus, Assoc::Left))
        .op(Op::infix(Rule::times, Assoc::Left) | Op::infix(Rule::dividedBy, Assoc::Left))
        .op(Op::prefix(Rule::unaryMinus) | Op::prefix(Rule::not))
        .op(Op::postfix(Rule::call))
        .op(Op::postfix(Rule::indexing))
});

impl Expression<()> {
    pub fn from_pair(pair: Pair<Rule>, file: &str) -> Expression<()> {
        PRATT_PARSER
            .map_primary(|primary| match primary.as_rule() {
                Rule::expr => Expression::from_pair(primary, file),
                Rule::decimalNumber | Rule::hexNumber => {
                    Expression::Integer(Integer::from_pair(primary, file))
                }
                Rule::character => Expression::Character(Character::from_pair(primary, file)),
                Rule::ident => Expression::Ident(Ident::from_pair(primary, file)),
                Rule::string => Expression::Str(Str::from_pair(primary, file)),
                Rule::fnDef => Expression::FnDef(FnDef::from_pair(primary, file)),
                Rule::ifStmt => Expression::If(If::from_pair(primary, file)),
                Rule::block => Expression::Block(Block::from_pair(primary, file)),
                Rule::boolean => Expression::Boolean(Boolean::from_pair(primary, file)),
                Rule::array => Expression::Array(Array::from_pair(primary, file)),
                rule => unreachable!("Unexpected rule {:?} while parsing primary", rule),
            })
            .map_prefix(|op, rhs| Expression::Prefix(PrefixExpr::from_op_rhs(op, rhs, file)))
            .map_postfix(|lhs, op| Expression::Postfix(PostfixExpr::from_lhs_op(lhs, op, file)))
            .map_infix(|lhs, op, rhs| {
                Expression::Binary(BinaryExpr::from_lhs_op_rhs(lhs, op, rhs, file))
            })
            .parse(pair.into_inner())
    }
}

impl<T> Expression<T>
where
    T: Clone,
{
    pub fn position(&self) -> Position {
        match self {
            Expression::If(If { position, .. })
            | Expression::Binary(BinaryExpr { position, .. })
            | Expression::Prefix(PrefixExpr { position, .. })
            | Expression::Postfix(PostfixExpr { position, .. })
            | Expression::Integer(Integer { position, .. })
            | Expression::Character(Character { position, .. })
            | Expression::Ident(Ident { position, .. })
            | Expression::Str(Str { position, .. })
            | Expression::FnDef(FnDef { position, .. })
            | Expression::Block(Block { position, .. })
            | Expression::Boolean(Boolean { position, .. })
            | Expression::Array(Array { position, .. }) => position.to_owned(),
        }
    }

    pub fn info(&self) -> T {
        match self {
            Expression::If(If { info, .. })
            | Expression::Binary(BinaryExpr { info, .. })
            | Expression::Prefix(PrefixExpr { info, .. })
            | Expression::Postfix(PostfixExpr { info, .. })
            | Expression::Integer(Integer { info, .. })
            | Expression::Character(Character { info, .. })
            | Expression::Ident(Ident { info, .. })
            | Expression::Str(Str { info, .. })
            | Expression::FnDef(FnDef { info, .. })
            | Expression::Block(Block { info, .. })
            | Expression::Boolean(Boolean { info, .. })
            | Expression::Array(Array { info, .. }) => info.clone(),
        }
    }
}
