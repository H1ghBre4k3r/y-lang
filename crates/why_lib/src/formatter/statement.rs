use crate::{
    formatter::{Format, FormatterContext},
    parser::ast::{
        Assignment, Constant, Declaration, Initialisation, Instance, LValue, MethodDeclaration,
        Statement, StructDeclaration, StructFieldDeclaration, TopLevelStatement, WhileLoop,
    },
};

impl Format for Statement<()> {
    fn format(&self, ctx: &mut FormatterContext) -> Result<(), std::fmt::Error> {
        match self {
            Statement::Function(function) => function.format(ctx),
            Statement::WhileLoop(while_loop) => while_loop.format(ctx),
            Statement::Initialization(init) => {
                init.format(ctx)?;
                ctx.write(";")
            }
            Statement::Constant(constant) => {
                constant.format(ctx)?;
                ctx.write(";")
            }
            Statement::Assignment(assignment) => {
                assignment.format(ctx)?;
                ctx.write(";")
            }
            Statement::Expression(expr) => {
                expr.format(ctx)?;
                ctx.write(";")
            }
            Statement::YieldingExpression(expr) => expr.format(ctx),
            Statement::Return(expr) => {
                ctx.write("return ")?;
                expr.format(ctx)?;
                ctx.write(";")
            }
            Statement::Comment(comment) => ctx.write(comment),
            Statement::Declaration(declaration) => {
                declaration.format(ctx)?;
                ctx.write(";")
            }
            Statement::StructDeclaration(struct_decl) => struct_decl.format(ctx),
        }
    }
}

impl Format for TopLevelStatement<()> {
    fn format(&self, ctx: &mut FormatterContext) -> Result<(), std::fmt::Error> {
        match self {
            TopLevelStatement::Comment(comment) => ctx.write(comment),
            TopLevelStatement::Function(function) => function.format(ctx),
            TopLevelStatement::Constant(constant) => {
                constant.format(ctx)?;
                ctx.write(";")
            }
            TopLevelStatement::Declaration(declaration) => {
                declaration.format(ctx)?;
                ctx.write(";")
            }
            TopLevelStatement::StructDeclaration(struct_decl) => struct_decl.format(ctx),
            TopLevelStatement::Instance(instance) => instance.format(ctx),
        }
    }
}

impl Format for Constant<()> {
    fn format(&self, ctx: &mut FormatterContext) -> Result<(), std::fmt::Error> {
        ctx.write("const ")?;
        self.id.format(ctx)?;
        ctx.write(": ")?;
        self.type_name.format(ctx)?;
        ctx.write(" = ")?;
        self.value.format(ctx)
    }
}

impl Format for Assignment<()> {
    fn format(&self, ctx: &mut FormatterContext) -> Result<(), std::fmt::Error> {
        self.lvalue.format(ctx)?;
        ctx.write(" = ")?;
        self.rvalue.format(ctx)
    }
}

impl Format for LValue<()> {
    fn format(&self, ctx: &mut FormatterContext) -> Result<(), std::fmt::Error> {
        match self {
            LValue::Id(id) => id.format(ctx),
            LValue::Postfix(postfix) => postfix.format(ctx),
        }
    }
}

impl Format for Initialisation<()> {
    fn format(&self, ctx: &mut FormatterContext) -> Result<(), std::fmt::Error> {
        ctx.write("let ")?;
        if self.mutable {
            ctx.write("mut ")?;
        }
        self.id.format(ctx)?;
        if let Some(type_name) = &self.type_name {
            ctx.write(": ")?;
            type_name.format(ctx)?;
        }
        ctx.write(" = ")?;
        self.value.format(ctx)
    }
}

impl Format for Declaration<()> {
    fn format(&self, ctx: &mut FormatterContext) -> Result<(), std::fmt::Error> {
        ctx.write("declare ")?;
        self.name.format(ctx)?;
        ctx.write(": ")?;
        self.type_name.format(ctx)
    }
}

impl Format for StructDeclaration<()> {
    fn format(&self, ctx: &mut FormatterContext) -> Result<(), std::fmt::Error> {
        ctx.write("struct ")?;
        self.id.format(ctx)?;
        ctx.write(" {")?;

        if !self.fields.is_empty() {
            ctx.write_newline()?;
            ctx.with_indent(|ctx| {
                for field in &self.fields {
                    ctx.write_indent()?;
                    field.format(ctx)?;
                    ctx.write(";")?;
                    ctx.write_newline()?;
                }
                Ok(())
            })?;
        }

        ctx.write("}")
    }
}

impl Format for StructFieldDeclaration<()> {
    fn format(&self, ctx: &mut FormatterContext) -> Result<(), std::fmt::Error> {
        self.name.format(ctx)?;
        ctx.write(": ")?;
        self.type_name.format(ctx)
    }
}

impl Format for Instance<()> {
    fn format(&self, ctx: &mut FormatterContext) -> Result<(), std::fmt::Error> {
        ctx.write("instance ")?;
        self.name.format(ctx)?;
        ctx.write(" {")?;

        let total_items = self.functions.len() + self.declarations.len();
        if total_items > 0 {
            ctx.indent();
            ctx.write_newline()?;
            let mut item_count = 0;

            for function in &self.functions {
                if item_count > 0 {
                    ctx.write_newline()?;
                }
                ctx.write_indent()?;
                function.format(ctx)?;
                ctx.write_newline()?;
                item_count += 1;
            }

            for declaration in &self.declarations {
                if item_count > 0 {
                    ctx.write_newline()?;
                }
                ctx.write_indent()?;
                declaration.format(ctx)?;
                ctx.write_newline()?;
                item_count += 1;
            }
            ctx.dedent();
        }

        ctx.write("}")
    }
}

impl Format for MethodDeclaration<()> {
    fn format(&self, ctx: &mut FormatterContext) -> Result<(), std::fmt::Error> {
        ctx.write("declare ")?;
        self.id.format(ctx)?;
        ctx.write("(")?;

        ctx.write_separated(&self.parameter_types, ", ", |ctx, type_name| {
            type_name.format(ctx)
        })?;

        ctx.write("): ")?;
        self.return_type.format(ctx)?;
        ctx.write(";")
    }
}

impl Format for WhileLoop<()> {
    fn format(&self, ctx: &mut FormatterContext) -> Result<(), std::fmt::Error> {
        ctx.write("while (")?;
        self.condition.format(ctx)?;
        ctx.write(") ")?;
        self.block.format(ctx)
    }
}
