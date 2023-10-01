use crate::{
    lexer::{Token, Tokens},
    parser::{
        ast::{AstNode, Expression, Id, TypeName},
        combinators::Comb,
        FromTokens, ParseError,
    },
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Initialization {
    pub id: Id,
    pub mutable: bool,
    pub type_name: Option<TypeName>,
    pub value: Expression,
}

impl FromTokens<Token> for Initialization {
    fn parse(tokens: &mut Tokens<Token>) -> Result<AstNode, ParseError>
    where
        Self: Sized,
    {
        Comb::LET.parse(tokens)?;

        let mutable = matches!(tokens.peek(), Some(Token::Mut { .. }));

        let matcher = !Comb::MUT
            >> Comb::ID
            >> !(Comb::COLON >> Comb::TYPE_NAME)
            >> Comb::EQ
            >> Comb::EXPR
            >> Comb::SEMI;

        let result = matcher.parse(tokens)?;

        let Some(AstNode::Id(id)) = result.get(0) else {
            unreachable!()
        };

        let mut type_name = None;

        let value: Expression;

        match result.get(1) {
            Some(AstNode::TypeName(type_)) => {
                type_name = Some(type_.clone());

                let Some(AstNode::Expression(expr)) = result.get(2) else {
                    unreachable!()
                };
                value = expr.clone();
            }
            Some(AstNode::Expression(expr)) => {
                value = expr.clone();
            }
            _ => unreachable!(),
        }

        Ok(Initialization {
            id: id.clone(),
            mutable,
            value: value.clone(),
            type_name,
        }
        .into())
    }
}

impl From<Initialization> for AstNode {
    fn from(value: Initialization) -> Self {
        AstNode::Initialization(value)
    }
}

#[cfg(test)]
mod tests {}
