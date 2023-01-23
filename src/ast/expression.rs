use log::error;
use pest::iterators::Pair;

use super::{BinaryOp, Block, FnCall, FnDef, Ident, If, Integer, Position, Rule, Str};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Expression {
    If(If),
    BinaryOp(BinaryOp),
    FnCall(FnCall),
    Integer(Integer),
    Ident(Ident),
    Str(Str),
    FnDef(FnDef),
    Block(Block),
}

impl Expression {
    pub fn from_pair(pair: Pair<Rule>) -> Expression {
        match pair.as_rule() {
            Rule::integer => Expression::Integer(Integer::from_pair(pair)),
            Rule::ident => Expression::Ident(Ident::from_pair(pair)),
            Rule::fnCall => Expression::FnCall(FnCall::from_pair(pair)),
            Rule::string => Expression::Str(Str::from_pair(pair)),
            Rule::binaryExpr => Expression::BinaryOp(BinaryOp::from_pair(pair)),
            Rule::fnDef => Expression::FnDef(FnDef::from_pair(pair)),
            Rule::ifStmt => Expression::If(If::from_pair(pair)),
            Rule::block => Expression::Block(Block::from_pair(pair)),
            _ => {
                error!(
                    "Unexpected expression '{}' at {}:{}",
                    pair.as_str(),
                    pair.line_col().0,
                    pair.line_col().1
                );
                std::process::exit(-1)
            }
        }
    }

    pub fn position(&self) -> Position {
        match self {
            Expression::If(If { position, .. })
            | Expression::BinaryOp(BinaryOp { position, .. })
            | Expression::FnCall(FnCall { position, .. })
            | Expression::Integer(Integer { position, .. })
            | Expression::Ident(Ident { position, .. })
            | Expression::Str(Str { position, .. })
            | Expression::FnDef(FnDef { position, .. })
            | Expression::Block(Block { position, .. }) => position.to_owned(),
        }
    }
}
