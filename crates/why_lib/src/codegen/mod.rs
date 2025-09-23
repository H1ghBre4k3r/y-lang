mod context;
mod expressions;
mod statements;
mod util;

pub use self::context::*;
pub use self::util::*;

pub trait CodeGen<'ctx> {
    type ReturnValue;
    fn codegen(&self, ctx: &CodegenContext<'ctx>) -> Self::ReturnValue;
}
