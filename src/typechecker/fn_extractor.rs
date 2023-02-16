use crate::ast::Ast;

use super::{error::TypeError, typescope::TypeScope, Typechecker};

pub fn extract_function_declarations(ast: &Ast<()>) -> Result<TypeScope, TypeError> {
    Typechecker::extract_function_types(ast)
}
