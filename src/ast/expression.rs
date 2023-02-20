use once_cell::sync::Lazy;
use pest::{iterators::Pair, pratt_parser::{PrattParser, Assoc, Op}};

use super::{BinaryExpr, Block, Boolean, FnCall, FnDef, Ident, If, Integer, Position, Rule, Str};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Expression<T> {
    If(If<T>),
    Binary(BinaryExpr<T>),
    FnCall(FnCall<T>),
    Integer(Integer<T>),
    Ident(Ident<T>),
    Str(Str<T>),
    FnDef(FnDef<T>),
    Block(Block<T>),
    Boolean(Boolean<T>),
}

static PRATT_PARSER: Lazy<PrattParser<Rule>> = Lazy::new(|| {
    PrattParser::new()
        .op(
            Op::infix(Rule::lessThan, Assoc::Left)
          | Op::infix(Rule::greaterThan, Assoc::Left)
          | Op::infix(Rule::equal, Assoc::Left))
        .op(Op::infix(Rule::plus, Assoc::Left) | Op::infix(Rule::minus, Assoc::Left))
        .op(Op::infix(Rule::times, Assoc::Left) | Op::infix(Rule::dividedBy, Assoc::Left))
});

impl Expression<()> {
    pub fn from_pair(pair: Pair<Rule>, file: &str) -> Expression<()> {
        PRATT_PARSER
            .map_primary(|primary| match primary.as_rule() {
                Rule::integer => Expression::Integer(Integer::from_pair(primary, file)),
                Rule::ident => Expression::Ident(Ident::from_pair(primary, file)),
                Rule::fnCall => Expression::FnCall(FnCall::from_pair(primary, file)),
                Rule::string => Expression::Str(Str::from_pair(primary, file)),
                Rule::fnDef => Expression::FnDef(FnDef::from_pair(primary, file)),
                Rule::ifStmt => Expression::If(If::from_pair(primary, file)),
                Rule::block => Expression::Block(Block::from_pair(primary, file)),
                Rule::boolean => Expression::Boolean(Boolean::from_pair(primary, file)),
                rule => unreachable!("Unexpected rule {:?} while parsing primary", rule),
            })
            // TODO: Add map_prefix and map_postfix once such operators are added to the grammar
            // See https://github.com/pest-parser/pest/blob/18ca64fb/derive/examples/calc.rs#L44
            .map_infix(|lhs, op, rhs| Expression::Binary(BinaryExpr::from_lhs_op_rhs(lhs, op, rhs, file)))
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
            | Expression::FnCall(FnCall { position, .. })
            | Expression::Integer(Integer { position, .. })
            | Expression::Ident(Ident { position, .. })
            | Expression::Str(Str { position, .. })
            | Expression::FnDef(FnDef { position, .. })
            | Expression::Block(Block { position, .. })
            | Expression::Boolean(Boolean { position, .. }) => position.to_owned(),
        }
    }

    pub fn info(&self) -> T {
        match self {
            Expression::If(If { info, .. })
            | Expression::Binary(BinaryExpr { info, .. })
            | Expression::FnCall(FnCall { info, .. })
            | Expression::Integer(Integer { info, .. })
            | Expression::Ident(Ident { info, .. })
            | Expression::Str(Str { info, .. })
            | Expression::FnDef(FnDef { info, .. })
            | Expression::Block(Block { info, .. })
            | Expression::Boolean(Boolean { info, .. }) => info.clone(),
        }
    }
}
