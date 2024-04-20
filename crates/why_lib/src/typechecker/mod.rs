mod context;
mod error;
mod scope;
mod typed_ast;
mod types;

use std::{cell::RefCell, error::Error, fmt::Debug, rc::Rc};

use crate::parser::ast::Statement;

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

pub type TypeResult<T> = Result<T, TypeCheckError>;

#[derive(Debug, Clone, Default)]
pub struct TypeChecker {
    context: Context,
    statements: Vec<Statement<()>>,
}

trait TypeCheckable {
    type Output;

    fn check(self, ctx: &mut Context) -> TypeResult<Self::Output>;

    fn revert(this: &Self::Output) -> Self;
}

trait TypedConstruct
where
    Self: Debug,
{
    fn update_type(&mut self, type_id: Type) -> TypeResult<()> {
        unimplemented!(
            "TypedConstruct::update_type({type_id:?}) is not implemented for {:?}",
            self
        )
    }

    fn validate(&self) -> Result<(), Box<dyn Error>> {
        unimplemented!("TypedConstruct::validate is not implemented for {self:?}")
    }
}

impl TypeChecker {
    pub fn new(statements: Vec<Statement<()>>) -> TypeChecker {
        TypeChecker {
            statements,
            ..Default::default()
        }
    }

    pub fn check(mut self) -> TypeResult<Vec<Statement<TypeInformation>>> {
        let mut checked = vec![];

        for stm in self.statements.into_iter() {
            checked.push(stm.check(&mut self.context)?);
        }

        Ok(checked)
    }
}
