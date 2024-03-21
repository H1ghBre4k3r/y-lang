mod context;
mod error;
mod scope;
mod typed_ast;
mod types;

use std::{cell::RefCell, error::Error, fmt::Debug, rc::Rc};

use crate::parser::ast::Statement;

use self::{
    context::Context,
    error::{TypeCheckError, TypeMismatch},
    types::Type,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypeInformation {
    pub type_id: Rc<RefCell<Option<Type>>>,
}

pub type TypeResult<T> = Result<T, TypeCheckError>;

#[derive(Debug, Clone, Default)]
pub struct TypeChecker {
    context: Context,
}

trait TypeCheckable {
    type Output;

    fn check(self, ctx: &mut Context) -> TypeResult<Self::Output>;
}

trait TypedConstruct
where
    Self: Debug,
{
    fn update_type(&mut self, type_id: Type) -> Result<(), TypeMismatch> {
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
    pub fn new() -> TypeChecker {
        Default::default()
    }

    pub fn check(
        &mut self,
        statements: Vec<Statement<()>>,
    ) -> TypeResult<Vec<Statement<TypeInformation>>> {
        let mut checked = vec![];

        for stm in statements.into_iter() {
            checked.push(stm.check(&mut self.context)?);
        }

        Ok(checked)
    }
}
