mod constant_folding;
mod constant_inlining;

use constant_folding::ConstantFolding;
use constant_inlining::ConstantInlining;

use crate::{typechecker::ValidatedTypeInformation, Ast};

pub trait OptimizerPass {
    fn run(&self, ast: Ast<ValidatedTypeInformation>) -> Ast<ValidatedTypeInformation>;
}

pub fn optimize(ast: Ast<ValidatedTypeInformation>) -> Ast<ValidatedTypeInformation> {
    let default_passes: Vec<Box<&dyn OptimizerPass>> =
        vec![Box::new(&ConstantInlining), Box::new(&ConstantFolding)];

    let mut ast = ast;

    for pass in &default_passes {
        ast = pass.run(ast);
    }

    ast
}
