use crate::{typechecker::ValidatedTypeInformation, Ast};

use super::OptimizerPass;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ConstantFolding;

impl OptimizerPass for ConstantFolding {
    fn run(&self, ast: Ast<ValidatedTypeInformation>) -> Ast<ValidatedTypeInformation> {
        ast
    }
}
