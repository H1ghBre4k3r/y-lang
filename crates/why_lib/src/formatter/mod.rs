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

    // For leading blank line removal, we need to check if there are actual blank lines at the start
    // This is complex because we don't have source position info for comments vs blank lines
    // For now, we'll keep all statements and let the blank line detection between them handle spacing
    let start_index = 0;
    let end_index = statements.len();

    let relevant_statements = &statements[start_index..end_index];

    for (i, stmt) in relevant_statements.iter().enumerate() {
        if i > 0 {
            let blank_lines = count_blank_lines_between(&relevant_statements[i - 1], stmt);
            let preserved_lines = if blank_lines == 0 {
                0
            } else {
                1 // Preserve single blank line, collapse multiple to one
            };

            // Always add at least one newline between statements
            ctx.write("\n")?;
            for _ in 0..preserved_lines {
                ctx.write("\n")?;
            }
        }
        stmt.format(&mut ctx)?;
    }

    Ok(ctx.output)
}

fn count_blank_lines_between(first: &TopLevelStatement<()>, second: &TopLevelStatement<()>) -> usize {
    let first_end_line = get_end_line(first);
    let second_start_line = get_start_line(second);

    if second_start_line > first_end_line {
        second_start_line - first_end_line - 1
    } else {
        0
    }
}

fn get_start_line(stmt: &TopLevelStatement<()>) -> usize {
    match stmt {
        TopLevelStatement::Comment(_) => 0, // Comments don't have position info
        TopLevelStatement::Function(func) => func.position.start.0,
        TopLevelStatement::Constant(constant) => constant.position.start.0,
        TopLevelStatement::Declaration(decl) => decl.position.start.0,
        TopLevelStatement::StructDeclaration(decl) => decl.position.start.0,
        TopLevelStatement::Instance(instance) => instance.position.start.0,
    }
}

fn get_end_line(stmt: &TopLevelStatement<()>) -> usize {
    match stmt {
        TopLevelStatement::Comment(_) => 0, // Comments don't have position info
        TopLevelStatement::Function(func) => func.position.end.0,
        TopLevelStatement::Constant(constant) => constant.position.end.0,
        TopLevelStatement::Declaration(decl) => decl.position.end.0,
        TopLevelStatement::StructDeclaration(decl) => decl.position.end.0,
        TopLevelStatement::Instance(instance) => instance.position.end.0,
    }
}
