use crate::{
    formatter::{Format, FormatterContext},
    parser::ast::{
        Array, AstString, BinaryExpression, BinaryOperator, Block, Bool, Character, Expression,
        Function, FunctionParameter, Id, If, Lambda, LambdaParameter, Num, Postfix, Prefix,
        StructFieldInitialisation, StructInitialisation, TypeName,
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
        ctx.write(" {")?;

        if !self.body.statements.is_empty() {
            ctx.write_newline()?;
            ctx.with_indent(|ctx| {
                for (i, stmt) in self.body.statements.iter().enumerate() {
                    if i > 0 {
                        ctx.write_newline()?;
                    }
                    ctx.write_indent()?;
                    stmt.format(ctx)?;
                }
                Ok(())
            })?;
            ctx.write_newline()?;
            ctx.write_indent()?;
        }

        ctx.write("}")
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
        ctx.write(") {")?;

        if !self.then_block.statements.is_empty() {
            ctx.write_newline()?;
            ctx.with_indent(|ctx| {
                for stmt in &self.then_block.statements {
                    ctx.write_indent()?;
                    stmt.format(ctx)?;
                    ctx.write_newline()?;
                }
                Ok(())
            })?;
            ctx.write_indent()?; // Add indentation for closing brace
        }

        ctx.write("}")?;

        if !self.else_block.statements.is_empty() {
            ctx.write(" else {")?;
            ctx.write_newline()?;
            ctx.with_indent(|ctx| {
                for stmt in &self.else_block.statements {
                    ctx.write_indent()?;
                    stmt.format(ctx)?;
                    ctx.write_newline()?;
                }
                Ok(())
            })?;
            ctx.write_indent()?; // Add indentation for else closing brace
            ctx.write("}")?;
        }

        Ok(())
    }
}

impl Format for Block<()> {
    fn format(&self, ctx: &mut FormatterContext) -> Result<(), std::fmt::Error> {
        ctx.write("{")?;

        if !self.statements.is_empty() {
            ctx.write_newline()?;
            ctx.with_indent(|ctx| {
                for stmt in &self.statements {
                    ctx.write_indent()?;
                    stmt.format(ctx)?;
                    ctx.write_newline()?;
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
