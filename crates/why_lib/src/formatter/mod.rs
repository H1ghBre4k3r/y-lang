pub mod context;
pub mod expression;
pub mod statement;

pub use context::*;

use crate::parser::ast::{Expression, Statement, TopLevelStatement};

pub trait Format {
    fn format(&self, ctx: &mut FormatterContext) -> Result<(), std::fmt::Error>;
}

pub fn format_expression(expr: &Expression<()>) -> Result<String, std::fmt::Error> {
    let mut ctx = FormatterContext::new();
    expr.format(&mut ctx)?;
    Ok(ctx.output)
}

pub fn format_statement(stmt: &Statement<()>) -> Result<String, std::fmt::Error> {
    let mut ctx = FormatterContext::new();
    stmt.format(&mut ctx)?;
    Ok(ctx.output)
}

pub fn format_top_level_statement(stmt: &TopLevelStatement<()>) -> Result<String, std::fmt::Error> {
    let mut ctx = FormatterContext::new();
    stmt.format(&mut ctx)?;
    Ok(ctx.output)
}

pub fn format_program(statements: &[TopLevelStatement<()>]) -> Result<String, std::fmt::Error> {
    let mut ctx = FormatterContext::new();

    for (i, stmt) in statements.iter().enumerate() {
        if i > 0 {
            ctx.write("\n\n")?;
        }
        stmt.format(&mut ctx)?;
    }

    Ok(ctx.output)
}
