use crate::{
    lexer::{Token, Tokens},
    parser::{
        ast::{AstNode, Statement},
        combinators::Comb,
        FromTokens, ParseError,
    },
};

use super::Id;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Function {
    parameters: Vec<Parameter>,
    return_type: String,
    statements: Vec<Statement>,
}

impl FromTokens<Token> for Function {
    fn parse(tokens: &mut Tokens<Token>) -> Result<AstNode, ParseError> {
        let matcher = Comb::FN_KEYWORD
            >> Comb::LPAREN
            >> !(Comb::PARAMETER >> ((Comb::COMMA >> Comb::PARAMETER) ^ ()))
            >> Comb::RPAREN
            >> Comb::COLON
            >> Comb::ID
            >> Comb::LBRACE
            >> (Comb::STATEMENT ^ ())
            >> Comb::RBRACE;

        let mut result = matcher.parse(tokens)?.into_iter().peekable();

        let mut parameters = vec![];

        while let Some(AstNode::Parameter(param)) =
            result.next_if(|item| matches!(item, AstNode::Parameter(_)))
        {
            parameters.push(param);
        }

        let Some(AstNode::Id(return_type)) = result.next() else {
            unreachable!();
        };

        let mut statements = vec![];

        while let Some(AstNode::Statement(param)) =
            result.next_if(|item| matches!(item, AstNode::Statement(_)))
        {
            statements.push(param);
        }

        Ok(Function {
            parameters,
            return_type: return_type.0,
            statements,
        }
        .into())
    }
}

impl From<Function> for AstNode {
    fn from(value: Function) -> Self {
        AstNode::Function(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Parameter {
    name: Id,
    type_: String,
}

impl FromTokens<Token> for Parameter {
    fn parse(tokens: &mut Tokens<Token>) -> Result<AstNode, ParseError> {
        let matcher = Comb::ID >> Comb::COLON >> Comb::ID;
        let result = matcher.parse(tokens)?;

        let [AstNode::Id(name), AstNode::Id(type_)] = result.as_slice() else {
            unreachable!();
        };

        Ok(Parameter {
            name: name.clone(),
            type_: type_.0.clone(),
        }
        .into())
    }
}

impl From<Parameter> for AstNode {
    fn from(value: Parameter) -> Self {
        AstNode::Parameter(value)
    }
}
