use crate::{
    formatter::{Format, FormatterContext},
    parser::ast::{
        Array, AstString, BinaryExpression, BinaryOperator, Block, Bool, Character, Expression,
        Function, FunctionParameter, Id, If, Lambda, LambdaParameter, Num, Postfix, Prefix,
        Statement, StructFieldInitialisation, StructInitialisation, TypeName,
    },
};
use std::fmt::Write;

impl Format for Expression<()> {
    fn format(&self, ctx: &mut FormatterContext) -> Result<(), std::fmt::Error> {
        match self {
            Expression::Id(id) => id.format(ctx),
            Expression::Num(num) => num.format(ctx),
            Expression::Bool(bool) => bool.format(ctx),
            Expression::Character(character) => character.format(ctx),
            Expression::AstString(string) => string.format(ctx),
            Expression::Function(function) => function.format(ctx),
            Expression::Lambda(lambda) => lambda.format(ctx),
            Expression::If(if_expr) => if_expr.format(ctx),
            Expression::Block(block) => block.format(ctx),
            Expression::Parens(expr) => {
                ctx.write("(")?;
                expr.format(ctx)?;
                ctx.write(")")
            }
            Expression::Postfix(postfix) => postfix.format(ctx),
            Expression::Prefix(prefix) => prefix.format(ctx),
            Expression::Binary(binary) => binary.format(ctx),
            Expression::Array(array) => array.format(ctx),
            Expression::StructInitialisation(struct_init) => struct_init.format(ctx),
        }
    }
}

impl Format for Id<()> {
    fn format(&self, ctx: &mut FormatterContext) -> Result<(), std::fmt::Error> {
        ctx.write(&self.name)
    }
}

impl Format for Num<()> {
    fn format(&self, ctx: &mut FormatterContext) -> Result<(), std::fmt::Error> {
        match self {
            Num::Integer(value, _, _) => write!(ctx.output, "{value}"),
            Num::FloatingPoint(value, _, _) => write!(ctx.output, "{value}"),
        }
    }
}

impl Format for Character<()> {
    fn format(&self, ctx: &mut FormatterContext) -> Result<(), std::fmt::Error> {
        write!(ctx.output, "'{}'", escape_char(self.character))
    }
}

impl Format for Bool<()> {
    fn format(&self, ctx: &mut FormatterContext) -> Result<(), std::fmt::Error> {
        ctx.write(if self.value { "true" } else { "false" })
    }
}

impl Format for AstString<()> {
    fn format(&self, ctx: &mut FormatterContext) -> Result<(), std::fmt::Error> {
        write!(ctx.output, "\"{}\"", escape_string(&self.value))
    }
}

impl Format for Function<()> {
    fn format(&self, ctx: &mut FormatterContext) -> Result<(), std::fmt::Error> {
        ctx.write("fn ")?;
        self.id.format(ctx)?;
        ctx.write("(")?;

        ctx.write_separated(&self.parameters, ", ", |ctx, param| param.format(ctx))?;

        ctx.write("): ")?;
        self.return_type.format(ctx)?;
        ctx.write(" ")?;
        self.body.format(ctx)
    }
}

impl Format for FunctionParameter<()> {
    fn format(&self, ctx: &mut FormatterContext) -> Result<(), std::fmt::Error> {
        self.name.format(ctx)?;
        ctx.write(": ")?;
        self.type_name.format(ctx)
    }
}

impl Format for Lambda<()> {
    fn format(&self, ctx: &mut FormatterContext) -> Result<(), std::fmt::Error> {
        ctx.write("\\(")?;
        ctx.write_separated(&self.parameters, ", ", |ctx, param| param.format(ctx))?;
        ctx.write(") => ")?;
        self.expression.format(ctx)
    }
}

impl Format for LambdaParameter<()> {
    fn format(&self, ctx: &mut FormatterContext) -> Result<(), std::fmt::Error> {
        self.name.format(ctx)
    }
}

impl Format for If<()> {
    fn format(&self, ctx: &mut FormatterContext) -> Result<(), std::fmt::Error> {
        ctx.write("if (")?;
        self.condition.format(ctx)?;
        ctx.write(") ")?;
        self.then_block.format(ctx)?;

        if !self.else_block.statements.is_empty() {
            ctx.write(" else ")?;
            self.else_block.format(ctx)?;
        }

        Ok(())
    }
}

// Helper function to count blank lines between statements
fn count_blank_lines_between_statements(first: &Statement<()>, second: &Statement<()>) -> usize {
    let first_end_line = get_statement_end_line(first);
    let second_start_line = get_statement_start_line(second);

    if second_start_line > first_end_line {
        second_start_line - first_end_line - 1
    } else {
        0
    }
}

// Helper function to get the start line of a statement
fn get_statement_start_line(stmt: &Statement<()>) -> usize {
    match stmt {
        Statement::Function(func) => func.position.start.0,
        Statement::WhileLoop(while_loop) => while_loop.position.start.0,
        Statement::Initialization(init) => init.position.start.0,
        Statement::Constant(constant) => constant.position.start.0,
        Statement::Assignment(assignment) => assignment.position.start.0,
        Statement::Expression(expr) => get_expression_start_line(expr),
        Statement::YieldingExpression(expr) => get_expression_start_line(expr),
        Statement::Return(expr) => get_expression_start_line(expr),
        Statement::Comment(_) => 0, // Comments don't have position info
        Statement::Declaration(decl) => decl.position.start.0,
        Statement::StructDeclaration(decl) => decl.position.start.0,
    }
}

// Helper function to get the end line of a statement
fn get_statement_end_line(stmt: &Statement<()>) -> usize {
    match stmt {
        Statement::Function(func) => func.position.end.0,
        Statement::WhileLoop(while_loop) => while_loop.position.end.0,
        Statement::Initialization(init) => init.position.end.0,
        Statement::Constant(constant) => constant.position.end.0,
        Statement::Assignment(assignment) => assignment.position.end.0,
        Statement::Expression(expr) => get_expression_end_line(expr),
        Statement::YieldingExpression(expr) => get_expression_end_line(expr),
        Statement::Return(expr) => get_expression_end_line(expr),
        Statement::Comment(_) => 0, // Comments don't have position info
        Statement::Declaration(decl) => decl.position.end.0,
        Statement::StructDeclaration(decl) => decl.position.end.0,
    }
}

// Helper function to get the start line of an expression
fn get_expression_start_line(expr: &Expression<()>) -> usize {
    match expr {
        Expression::Id(id) => id.position.start.0,
        Expression::Num(num) => num.position().start.0,
        Expression::Bool(bool) => bool.position.start.0,
        Expression::Character(char) => char.position.start.0,
        Expression::AstString(string) => string.position.start.0,
        Expression::Function(func) => func.position.start.0,
        Expression::Lambda(lambda) => lambda.position.start.0,
        Expression::If(if_expr) => if_expr.position.start.0,
        Expression::Block(block) => block.position.start.0,
        Expression::Parens(expr) => get_expression_start_line(expr),
        Expression::Postfix(postfix) => postfix.position().start.0,
        Expression::Prefix(prefix) => prefix.position().start.0,
        Expression::Binary(binary) => binary.position.start.0,
        Expression::Array(array) => array.position().start.0,
        Expression::StructInitialisation(struct_init) => struct_init.position.start.0,
    }
}

// Helper function to get the end line of an expression
fn get_expression_end_line(expr: &Expression<()>) -> usize {
    match expr {
        Expression::Id(id) => id.position.end.0,
        Expression::Num(num) => num.position().end.0,
        Expression::Bool(bool) => bool.position.end.0,
        Expression::Character(char) => char.position.end.0,
        Expression::AstString(string) => string.position.end.0,
        Expression::Function(func) => func.position.end.0,
        Expression::Lambda(lambda) => lambda.position.end.0,
        Expression::If(if_expr) => if_expr.position.end.0,
        Expression::Block(block) => block.position.end.0,
        Expression::Parens(expr) => get_expression_end_line(expr),
        Expression::Postfix(postfix) => postfix.position().end.0,
        Expression::Prefix(prefix) => prefix.position().end.0,
        Expression::Binary(binary) => binary.position.end.0,
        Expression::Array(array) => array.position().end.0,
        Expression::StructInitialisation(struct_init) => struct_init.position.end.0,
    }
}

impl Format for Block<()> {
    fn format(&self, ctx: &mut FormatterContext) -> Result<(), std::fmt::Error> {
        ctx.write("{")?;

        if !self.statements.is_empty() {
            ctx.write_newline()?;
            ctx.with_indent(|ctx| {
                for (i, stmt) in self.statements.iter().enumerate() {
                    // Format the current statement
                    ctx.write_indent()?;
                    stmt.format(ctx)?;

                    // Add newline after statement (but don't add extra blank lines yet)
                    if i < self.statements.len() - 1 {
                        let blank_lines =
                            count_blank_lines_between_statements(stmt, &self.statements[i + 1]);

                        // If there were blank lines after this statement, preserve one
                        if blank_lines > 0 {
                            ctx.write_newline()?;
                            ctx.write_newline()?; // Add one blank line
                        } else {
                            ctx.write_newline()?; // Just separate with single newline
                        }
                    } else {
                        // Last statement, just add newline
                        ctx.write_newline()?;
                    }
                }
                Ok(())
            })?;
            ctx.write_indent()?;
        }

        ctx.write("}")
    }
}

impl Format for Postfix<()> {
    fn format(&self, ctx: &mut FormatterContext) -> Result<(), std::fmt::Error> {
        match self {
            Postfix::Call { expr, args, .. } => {
                expr.format(ctx)?;
                ctx.write("(")?;
                ctx.write_separated(args, ", ", |ctx, arg| arg.format(ctx))?;
                ctx.write(")")
            }
            Postfix::Index { expr, index, .. } => {
                expr.format(ctx)?;
                ctx.write("[")?;
                index.format(ctx)?;
                ctx.write("]")
            }
            Postfix::PropertyAccess { expr, property, .. } => {
                expr.format(ctx)?;
                ctx.write(".")?;
                property.format(ctx)
            }
        }
    }
}

impl Format for Prefix<()> {
    fn format(&self, ctx: &mut FormatterContext) -> Result<(), std::fmt::Error> {
        match self {
            Prefix::Minus { expr, .. } => {
                ctx.write("-")?;
                expr.format(ctx)
            }
            Prefix::Negation { expr, .. } => {
                ctx.write("!")?;
                expr.format(ctx)
            }
        }
    }
}

impl Format for BinaryExpression<()> {
    fn format(&self, ctx: &mut FormatterContext) -> Result<(), std::fmt::Error> {
        self.left.format(ctx)?;
        ctx.write(" ")?;
        self.operator.format(ctx)?;
        ctx.write(" ")?;
        self.right.format(ctx)
    }
}

impl Format for BinaryOperator {
    fn format(&self, ctx: &mut FormatterContext) -> Result<(), std::fmt::Error> {
        let op = match self {
            BinaryOperator::Add => "+",
            BinaryOperator::Substract => "-",
            BinaryOperator::Multiply => "*",
            BinaryOperator::Divide => "/",
            BinaryOperator::Equals => "==",
            BinaryOperator::NotEquals => "!=",
            BinaryOperator::GreaterThan => ">",
            BinaryOperator::LessThan => "<",
            BinaryOperator::GreaterOrEqual => ">=",
            BinaryOperator::LessOrEqual => "<=",
        };
        ctx.write(op)
    }
}

impl Format for Array<()> {
    fn format(&self, ctx: &mut FormatterContext) -> Result<(), std::fmt::Error> {
        match self {
            Array::Literal { values, .. } => {
                ctx.write("&[")?;
                ctx.write_separated(values, ", ", |ctx, value| value.format(ctx))?;
                ctx.write("]")
            }
            Array::Default {
                initial_value,
                length,
                ..
            } => {
                ctx.write("&[")?;
                initial_value.format(ctx)?;
                ctx.write("; ")?;
                length.format(ctx)?;
                ctx.write("]")
            }
        }
    }
}

impl Format for StructInitialisation<()> {
    fn format(&self, ctx: &mut FormatterContext) -> Result<(), std::fmt::Error> {
        self.id.format(ctx)?;
        ctx.write(" {")?;

        if !self.fields.is_empty() {
            ctx.write_newline()?;
            ctx.with_indent(|ctx| {
                for (i, field) in self.fields.iter().enumerate() {
                    ctx.write_indent()?;
                    field.format(ctx)?;
                    if i < self.fields.len() - 1 {
                        ctx.write(",")?;
                    }
                    ctx.write_newline()?;
                }
                Ok(())
            })?;
            ctx.write_indent()?;
        }

        ctx.write("}")
    }
}

impl Format for StructFieldInitialisation<()> {
    fn format(&self, ctx: &mut FormatterContext) -> Result<(), std::fmt::Error> {
        self.name.format(ctx)?;
        ctx.write(": ")?;
        self.value.format(ctx)
    }
}

impl Format for TypeName {
    fn format(&self, ctx: &mut FormatterContext) -> Result<(), std::fmt::Error> {
        match self {
            TypeName::Literal(name, _) => ctx.write(name),
            TypeName::Fn {
                params,
                return_type,
                ..
            } => {
                ctx.write("(")?;
                ctx.write_separated(params, ", ", |ctx, param| param.format(ctx))?;
                ctx.write(") -> ")?;
                return_type.format(ctx)
            }
            TypeName::Tuple(types, _) => {
                ctx.write("(")?;
                ctx.write_separated(types, ", ", |ctx, type_name| type_name.format(ctx))?;
                ctx.write(")")
            }
            TypeName::Array(inner, _) => {
                ctx.write("&[")?;
                inner.format(ctx)?;
                ctx.write("]")
            }
            TypeName::Reference(inner, _) => {
                ctx.write("&")?;
                inner.format(ctx)
            }
        }
    }
}

fn escape_char(c: char) -> String {
    match c {
        '\n' => "\\n".to_string(),
        '\t' => "\\t".to_string(),
        '\r' => "\\r".to_string(),
        '\\' => "\\\\".to_string(),
        '\'' => "\\'".to_string(),
        _ => c.to_string(),
    }
}

fn escape_string(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            '\n' => "\\n".to_string(),
            '\t' => "\\t".to_string(),
            '\r' => "\\r".to_string(),
            '\\' => "\\\\".to_string(),
            '"' => "\\\"".to_string(),
            _ => c.to_string(),
        })
        .collect()
}
