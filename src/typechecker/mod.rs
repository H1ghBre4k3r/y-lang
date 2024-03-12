mod context;
mod error;
mod scope;
mod typed_ast;
mod types;

use crate::parser::ast::Statement;

use self::{context::Context, error::TypeError, types::Type};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypeInformation {
    pub type_id: Type,
}

pub type TypeResult<T> = Result<T, TypeError>;

#[derive(Debug, Clone, Default)]
pub struct TypeChecker {
    context: Context,
}

trait TypeCheckable {
    type Output;

    fn check(self, ctx: &mut Context) -> TypeResult<Self::Output>;
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
