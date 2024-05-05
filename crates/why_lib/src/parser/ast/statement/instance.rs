use crate::{
    lexer::{Span, Token},
    parser::{
        ast::{AstNode, Function, Id},
        combinators::Comb,
        FromTokens, ParseError, ParseState,
    },
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Instance<T> {
    pub id: Id<T>,
    pub functions: Vec<Function<()>>,
    pub info: T,
    pub position: Span,
}

impl FromTokens<Token> for Instance<()> {
    fn parse(tokens: &mut ParseState<Token>) -> Result<AstNode, ParseError> {
        let position = tokens.span()?;

        let matcher =
            Comb::INSTANCE_KEYWORD >> Comb::ID >> Comb::LBRACE >> (Comb::FUNCTION ^ Comb::RBRACE);
        let mut result = matcher.parse(tokens)?.into_iter();

        let Some(AstNode::Id(id)) = result.next() else {
            unreachable!();
        };

        let mut functions = vec![];

        while let Some(AstNode::Function(function)) = result.next() {
            functions.push(function);
        }

        assert!(result.next().is_none());

        let Span { end, .. } = tokens.prev_span()?;

        Ok(Instance {
            id,
            functions,
            info: (),
            position: Span {
                start: position.start,
                end,
                source: position.source,
            },
        }
        .into())
    }
}

impl From<Instance<()>> for AstNode {
    fn from(value: Instance<()>) -> Self {
        AstNode::Instance(value)
    }
}
