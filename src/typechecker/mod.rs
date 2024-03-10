mod scope;
mod types;

use std::{error::Error, fmt::Display};

use crate::parser::ast::{Expression, Num, Statement};

use self::{scope::Scope, types::Type};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypeInformation {
    pub type_id: Option<Type>,
}

#[derive(Clone, Debug)]
pub struct TypeError {}

pub type TypeResult<T> = Result<T, TypeError>;

impl Display for TypeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl Error for TypeError {}

#[derive(Debug, Clone)]
pub struct TypeChecker {
    scope: Scope,
}

impl Default for TypeChecker {
    fn default() -> Self {
        TypeChecker {
            scope: Scope::new(),
        }
    }
}

impl TypeChecker {
    pub fn new() -> TypeChecker {
        Default::default()
    }

    pub fn check(
        &mut self,
        statements: Vec<Statement<()>>,
    ) -> TypeResult<Vec<Statement<TypeInformation>>> {
        let mut checked = vec![];

        for stm in statements.into_iter() {
            checked.push(self.check_statement(stm)?);
        }

        Ok(checked)
    }

    fn check_statement(
        &mut self,
        statement: Statement<()>,
    ) -> TypeResult<Statement<TypeInformation>> {
        match statement {
            Statement::Function(_) => todo!(),
            Statement::If(_) => todo!(),
            Statement::WhileLoop(_) => todo!(),
            Statement::Initialization(_) => todo!(),
            Statement::Constant(_) => todo!(),
            Statement::Assignment(_) => todo!(),
            Statement::Expression(expr) => Ok(Statement::Expression(self.check_expression(expr)?)),
            Statement::YieldingExpression(_) => todo!(),
            Statement::Return(_) => todo!(),
            Statement::Comment(_) => todo!(),
            Statement::Declaration(_) => todo!(),
            Statement::StructDeclaration(_) => todo!(),
        }
    }

    fn check_expression(
        &mut self,
        expression: Expression<()>,
    ) -> TypeResult<Expression<TypeInformation>> {
        match expression {
            Expression::Id(_) => todo!(),
            Expression::Num(num) => Ok(Expression::Num(self.check_num(num)?)),
            Expression::Function(_) => todo!(),
            Expression::Lambda(_) => todo!(),
            Expression::If(_) => todo!(),
            Expression::Block(_) => todo!(),
            Expression::Parens(_) => todo!(),
            Expression::Postfix(_) => todo!(),
            Expression::Prefix(_) => todo!(),
            Expression::Binary(_) => todo!(),
            Expression::Array(_) => todo!(),
            Expression::StructInitialisation(_) => todo!(),
        }
    }

    fn check_num(&mut self, num: Num<()>) -> TypeResult<Num<TypeInformation>> {
        match num {
            Num::Integer(val, _) => Ok(Num::Integer(
                val,
                TypeInformation {
                    type_id: Some(Type::Integer),
                },
            )),
            Num::FloatingPoint(val, _) => Ok(Num::FloatingPoint(
                val,
                TypeInformation {
                    type_id: Some(Type::Float),
                },
            )),
        }
    }
}
