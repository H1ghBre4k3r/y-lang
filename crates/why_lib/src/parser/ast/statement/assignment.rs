use crate::{
    lexer::{Span, Token},
    parser::{
        ast::{AstNode, Expression, Id, Postfix},
        combinators::Comb,
        FromTokens, ParseError, ParseState,
    },
};

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Assignment<T> {
    pub lvalue: LValue<T>,
    pub rvalue: Expression<T>,
    pub info: T,
    pub position: Span,
}

impl FromTokens<Token> for Assignment<()> {
    fn parse(tokens: &mut ParseState<Token>) -> Result<AstNode, ParseError> {
        let position = tokens.span()?;

        let lvalue = LValue::parse(tokens)?;

        Comb::ASSIGN.parse(tokens)?;

        let matcher = Comb::EXPR;

        let result = matcher.parse(tokens).inspect_err(|e| {
            tokens.add_error(e.clone());
        })?;

        let Some(AstNode::Expression(rvalue)) = result.first() else {
            unreachable!()
        };

        Ok(Assignment {
            lvalue,
            rvalue: rvalue.clone(),
            info: (),
            position: position.merge(&rvalue.position()),
        }
        .into())
    }
}

impl From<Assignment<()>> for AstNode {
    fn from(value: Assignment<()>) -> Self {
        AstNode::Assignment(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum LValue<T> {
    Id(Id<T>),
    Postfix(Postfix<T>),
}

impl LValue<()> {
    fn parse(tokens: &mut ParseState<Token>) -> Result<LValue<()>, ParseError> {
        let position = tokens.span()?;
        let error = ParseError {
            position: Some(position),
            message: "Expected Id or postfix expression as lvalue!".to_string(),
        };

        let matcher = Comb::EXPR;

        let result = matcher.parse(tokens)?;

        let Some(lvalue) = result.first() else {
            unreachable!()
        };

        let AstNode::Expression(lvalue) = lvalue else {
            return Err(error);
        };

        let postfix = match lvalue {
            Expression::Id(id) => return Ok(LValue::Id(id.clone())),
            Expression::Postfix(postfix) => postfix.clone(),
            _ => return Err(error),
        };

        Self::check_postfix(&postfix)?;

        Ok(LValue::Postfix(postfix))
    }

    fn check_postfix(postfix: &Postfix<()>) -> Result<(), ParseError> {
        let position = postfix.position();
        let val = match postfix {
            Postfix::Call { .. } => {
                return Err(ParseError {
                    position: Some(position),
                    message: "Unexpected postfix call in lvalue".into(),
                })
            }
            Postfix::Index { expr, .. } => expr,
            Postfix::PropertyAccess { expr, .. } => expr,
        }
        .clone();

        match *val {
            Expression::Id(_) => Ok(()),
            Expression::Postfix(postfix) => Self::check_postfix(&postfix),
            other => Err(ParseError {
                position: Some(position),
                message: format!("Unexpected {other:?} in lvalue"),
            }),
        }
    }
}

impl<T> LValue<T>
where
    T: Clone,
{
    pub fn position(&self) -> Span {
        match self {
            LValue::Id(id) => id.position.clone(),
            LValue::Postfix(postfix) => postfix.position(),
        }
    }

    pub fn get_info(&self) -> T {
        match self {
            LValue::Id(id) => id.info.clone(),
            LValue::Postfix(postfix) => postfix.get_info(),
        }
    }

    pub fn get_original_variable_name(&self) -> Id<T> {
        let postfix = match self {
            LValue::Id(id) => return id.clone(),
            LValue::Postfix(postfix) => postfix.clone(),
        };

        Self::get_original_variable_name_from_postfix(&postfix)
    }

    pub fn get_original_variable_name_from_postfix(postfix: &Postfix<T>) -> Id<T> {
        let expr = match postfix {
            Postfix::Index { expr, .. } | Postfix::PropertyAccess { expr, .. } => expr.clone(),
            _ => unreachable!(),
        };

        match *expr {
            Expression::Id(id) => id,
            Expression::Postfix(postfix) => Self::get_original_variable_name_from_postfix(&postfix),
            _ => unreachable!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        lexer::{Lexer, Span},
        parser::{
            ast::{Expression, Id, LValue, Num, Postfix},
            FromTokens,
        },
    };

    use super::Assignment;

    #[test]
    fn test_parse_simple_assignment() {
        let mut tokens = Lexer::new("a = 42").lex().expect("should work").into();

        let result = Assignment::parse(&mut tokens);

        assert_eq!(
            result,
            Ok(Assignment {
                lvalue: LValue::Id(Id {
                    name: "a".into(),
                    position: Span::default(),
                    info: ()
                }),
                rvalue: Expression::Num(Num::Integer(42, (), Span::default())),
                position: Span::default(),
                info: ()
            }
            .into())
        )
    }

    #[test]
    fn test_indexed_assign() {
        let mut tokens = Lexer::new("a[123] = 42").lex().expect("should work").into();

        let result = Assignment::parse(&mut tokens);

        assert_eq!(
            result,
            Ok(Assignment {
                lvalue: LValue::Postfix(Postfix::Index {
                    expr: Box::new(Expression::Id(Id {
                        name: "a".into(),
                        position: Span::default(),
                        info: ()
                    })),
                    index: Box::new(Expression::Num(Num::Integer(123, (), Span::default()))),
                    info: (),
                    position: Span::default()
                }),
                rvalue: Expression::Num(Num::Integer(42, (), Span::default())),
                position: Span::default(),
                info: ()
            }
            .into())
        )
    }

    #[test]
    fn test_property_access_assign() {
        let mut tokens = Lexer::new("a.b = 42").lex().expect("should work").into();

        let result = Assignment::parse(&mut tokens);

        assert_eq!(
            result,
            Ok(Assignment {
                lvalue: LValue::Postfix(Postfix::PropertyAccess {
                    expr: Box::new(Expression::Id(Id {
                        name: "a".into(),
                        position: Span::default(),
                        info: ()
                    })),
                    property: Id {
                        name: "b".into(),
                        position: Span::default(),
                        info: ()
                    },
                    info: (),
                    position: Span::default()
                }),
                rvalue: Expression::Num(Num::Integer(42, (), Span::default())),
                position: Span::default(),
                info: ()
            }
            .into())
        )
    }

    #[test]
    fn test_combined_index_and_property_assign() {
        let mut tokens = Lexer::new("a[123].b = 42")
            .lex()
            .expect("should work")
            .into();

        let result = Assignment::parse(&mut tokens);

        assert_eq!(
            result,
            Ok(Assignment {
                lvalue: LValue::Postfix(Postfix::PropertyAccess {
                    expr: Box::new(Expression::Postfix(Postfix::Index {
                        expr: Box::new(Expression::Id(Id {
                            name: "a".into(),
                            position: Span::default(),
                            info: ()
                        })),
                        index: Box::new(Expression::Num(Num::Integer(123, (), Span::default()))),
                        info: (),
                        position: Span::default()
                    })),
                    property: Id {
                        name: "b".into(),
                        position: Span::default(),
                        info: ()
                    },
                    info: (),
                    position: Span::default()
                }),
                rvalue: Expression::Num(Num::Integer(42, (), Span::default())),
                position: Span::default(),
                info: ()
            }
            .into())
        )
    }

    #[test]
    fn test_error_on_invalid_lvalue() {
        let mut tokens = Lexer::new("a() = 42").lex().expect("should work").into();

        let result = Assignment::parse(&mut tokens);

        assert!(result.is_err())
    }
}
