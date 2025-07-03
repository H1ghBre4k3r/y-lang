use crate::{typechecker::ValidatedTypeInformation, Ast};

use super::OptimizerPass;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ConstantInlining;

impl OptimizerPass for ConstantInlining {
    fn run(&self, ast: Ast<ValidatedTypeInformation>) -> Ast<ValidatedTypeInformation> {
        ast.into_iter().map(|stmt| stmt).collect()
    }
}
