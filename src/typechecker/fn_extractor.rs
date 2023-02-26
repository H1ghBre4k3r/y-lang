use crate::ast::Ast;

use super::{error::TypeError, typescope::TypeScope, Typechecker};

pub fn extract_exports(ast: &Ast<()>) -> Result<TypeScope, TypeError> {
    Typechecker::extract_exports(ast)
}
