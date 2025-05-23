mod context;
mod error;
mod scope;
mod typed_ast;
mod types;

use std::{cell::RefCell, error::Error, fmt::Debug, rc::Rc};
use std::fmt::{Display, Formatter};
use anyhow::anyhow;
use crate::lexer::Span;
use crate::parser::ast::TopLevelStatement;

pub use self::error::TypeCheckError;
use self::{context::Context, types::Type};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypeInformation {
    pub type_id: Rc<RefCell<Option<Type>>>,
    pub context: Context,
}

impl TypeInformation {
    pub fn has_type(&self) -> bool {
        self.type_id.borrow().is_some()
    }
}

#[derive(Debug)]
pub struct ValidatedTypeInformation {
    pub type_id: Type,
    pub context: Context,
}

impl TypeInformation {
    fn validate(self, position: &Span) -> Result<ValidatedTypeInformation, TypeValidationError> {
        let TypeInformation { type_id, context } = self;
        let verified_type_information = if let Some(type_id) = type_id.borrow().clone() {
            Ok(ValidatedTypeInformation { type_id, context })
        } else {
            Err(TypeValidationError(position.clone()))
        };

        verified_type_information
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypeValidationError(Span);

impl Display for TypeValidationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.0.to_string("Type must be known at compile time!").as_str())
    }
}

impl Error for TypeValidationError {}

pub type TypeResult<T> = Result<T, TypeCheckError>;

#[derive(Debug, Clone, Default)]
pub struct TypeChecker {
    context: Context,
    statements: Vec<TopLevelStatement<()>>,
}

trait TypeCheckable {
    type Typed;

    fn check(self, ctx: &mut Context) -> TypeResult<Self::Typed>;

    fn revert(this: &Self::Typed) -> Self;
}

trait ShallowCheck {
    fn shallow_check(&self, ctx: &mut Context) -> TypeResult<()>;
}

trait TypedConstruct
where
    Self: Debug,
{

    type Validated;

    fn update_type(&mut self, type_id: Type) -> TypeResult<()> {
        unimplemented!(
            "TypedConstruct::update_type({type_id:?}) is not implemented for {:?}",
            self
        )
    }

    fn validate(self) -> Result<Self::Validated, TypeValidationError>;
}

impl TypeChecker {
    pub fn new(statements: Vec<TopLevelStatement<()>>) -> TypeChecker {
        TypeChecker {
            statements,
            ..Default::default()
        }
    }

    fn shallow_check(&mut self) -> TypeResult<()> {
        let struct_declarations = self
            .statements
            .iter()
            .filter(|stm| matches!(stm, TopLevelStatement::StructDeclaration(_)))
            .collect::<Vec<_>>();

        let other_tl_statements = self
            .statements
            .iter()
            .filter(|stm| !matches!(stm, TopLevelStatement::StructDeclaration(_)))
            .collect::<Vec<_>>();

        for s in struct_declarations.iter() {
            s.shallow_check(&mut self.context)?;
        }

        for s in other_tl_statements.iter() {
            s.shallow_check(&mut self.context)?;
        }

        Ok(())
    }

    pub fn check(mut self) -> TypeResult<Vec<TopLevelStatement<TypeInformation>>> {
        self.shallow_check()?;

        let mut checked = vec![];

        for stm in self.statements.into_iter() {
            checked.push(stm.check(&mut self.context)?);
        }

        Ok(checked)
    }

    pub fn validate(statements: Vec<TopLevelStatement<TypeInformation>>) -> Result<Vec<TopLevelStatement<ValidatedTypeInformation>>, TypeValidationError> {
        let mut validated = vec![];

        for stm in statements {
            validated.push(stm.validate()?);
        }

        Ok(validated)
    }
}
