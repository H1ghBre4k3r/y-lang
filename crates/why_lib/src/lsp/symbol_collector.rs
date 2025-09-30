use std::sync::Arc;

use tower_lsp_server::lsp_types::{SymbolKind, Uri};

use crate::lexer::Span;
use crate::lsp::{PositionUtils, SymbolIndex, Definition, Reference, SymbolId};
use crate::typechecker::{Context, TypeInformation};
use crate::parser::ast::*;

/// Enhanced context that collects symbols during type checking
#[derive(Debug, Clone)]
pub struct SymbolCollectingContext {
    pub context: Context,
    pub symbol_index: Arc<SymbolIndex>,
    pub position_utils: Arc<PositionUtils>,
    pub uri: Uri,
}

impl SymbolCollectingContext {
    pub fn new(context: Context, text: &str, uri: Uri) -> Self {
        Self {
            context,
            symbol_index: Arc::new(SymbolIndex::new()),
            position_utils: Arc::new(PositionUtils::new(text)),
            uri,
        }
    }

    /// Get the underlying symbol index
    pub fn get_symbol_index(&self) -> Arc<SymbolIndex> {
        self.symbol_index.clone()
    }

    /// Add a symbol definition to the index
    fn add_definition(&self, name: String, kind: SymbolKind, span: &Span, type_info: Option<&TypeInformation>) {
        let symbol_id = self.symbol_index.next_symbol_id();
        let range = self.position_utils.span_to_range(span);

        let definition = Definition {
            symbol_id,
            name,
            kind,
            range,
            selection_range: range, // For now, use the same range
            type_info: type_info.map(|info| format!("{:?}", info)), // Convert to string for now
            parent: None, // TODO: Handle parent-child relationships
            uri: self.uri.clone(),
        };

        self.symbol_index.add_definition(definition);
    }

    /// Add a symbol reference to the index
    fn add_reference(&self, symbol_id: SymbolId, span: &Span, is_definition: bool) {
        let range = self.position_utils.span_to_range(span);

        let reference = Reference {
            symbol_id,
            range,
            uri: self.uri.clone(),
            is_definition,
        };

        self.symbol_index.add_reference(reference);
    }

    /// Find symbol ID by name (for tracking references)
    fn find_symbol_by_name(&self, name: &str) -> Option<SymbolId> {
        let symbol_ids = self.symbol_index.find_symbols_by_name(name);
        symbol_ids.first().copied() // Return the first match for now
    }
}

/// Trait for AST nodes that can collect symbols
pub trait SymbolCollectable {
    fn collect_symbols(&self, ctx: &SymbolCollectingContext);
}

// Implement symbol collection for top-level statements
impl SymbolCollectable for TopLevelStatement<TypeInformation> {
    fn collect_symbols(&self, ctx: &SymbolCollectingContext) {
        match self {
            TopLevelStatement::Comment(_) => {}, // No symbols to collect
            TopLevelStatement::Function(func) => func.collect_symbols(ctx),
            TopLevelStatement::Constant(constant) => constant.collect_symbols(ctx),
            TopLevelStatement::Declaration(decl) => decl.collect_symbols(ctx),
            TopLevelStatement::StructDeclaration(struct_decl) => struct_decl.collect_symbols(ctx),
            TopLevelStatement::Instance(instance) => instance.collect_symbols(ctx),
        }
    }
}

impl SymbolCollectable for Statement<TypeInformation> {
    fn collect_symbols(&self, ctx: &SymbolCollectingContext) {
        match self {
            Statement::Function(function) => function.collect_symbols(ctx),
            Statement::WhileLoop(while_loop) => while_loop.collect_symbols(ctx),
            Statement::Initialization(init) => init.collect_symbols(ctx),
            Statement::Constant(constant) => constant.collect_symbols(ctx),
            Statement::Assignment(assignment) => assignment.collect_symbols(ctx),
            Statement::Expression(expr) => expr.collect_symbols(ctx),
            Statement::YieldingExpression(expr) => expr.collect_symbols(ctx),
            Statement::Return(expr) => expr.collect_symbols(ctx),
            Statement::Comment(_) => {}, // No symbols to collect
            Statement::Declaration(decl) => decl.collect_symbols(ctx),
            Statement::StructDeclaration(struct_decl) => struct_decl.collect_symbols(ctx),
        }
    }
}

impl SymbolCollectable for Function<TypeInformation> {
    fn collect_symbols(&self, ctx: &SymbolCollectingContext) {
        // Add function definition
        ctx.add_definition(
            self.id.name.clone(),
            SymbolKind::FUNCTION,
            &self.position,
            Some(&self.info),
        );

        // Collect symbols from parameters
        for param in &self.parameters {
            param.collect_symbols(ctx);
        }

        // Collect symbols from body
        self.body.collect_symbols(ctx);
    }
}

impl SymbolCollectable for FunctionParameter<TypeInformation> {
    fn collect_symbols(&self, ctx: &SymbolCollectingContext) {
        // Add parameter as a variable
        ctx.add_definition(
            self.name.name.clone(),
            SymbolKind::VARIABLE,
            &self.position,
            Some(&self.info),
        );
    }
}

impl SymbolCollectable for StructDeclaration<TypeInformation> {
    fn collect_symbols(&self, ctx: &SymbolCollectingContext) {
        // Add struct definition
        ctx.add_definition(
            self.name.clone(),
            SymbolKind::STRUCT,
            &self.position,
            Some(&self.info),
        );

        // Collect symbols from fields
        for field in &self.fields {
            field.collect_symbols(ctx);
        }
    }
}

impl SymbolCollectable for StructFieldDeclaration<TypeInformation> {
    fn collect_symbols(&self, ctx: &SymbolCollectingContext) {
        // Add field definition
        ctx.add_definition(
            self.name.clone(),
            SymbolKind::FIELD,
            &self.position,
            Some(&self.info),
        );
    }
}

impl SymbolCollectable for Initialisation<TypeInformation> {
    fn collect_symbols(&self, ctx: &SymbolCollectingContext) {
        // Add variable definition
        ctx.add_definition(
            self.name.clone(),
            SymbolKind::VARIABLE,
            &self.position,
            Some(&self.info),
        );

        // Collect symbols from initializer expression
        self.value.collect_symbols(ctx);
    }
}

impl SymbolCollectable for Constant<TypeInformation> {
    fn collect_symbols(&self, ctx: &SymbolCollectingContext) {
        // Add constant definition
        ctx.add_definition(
            self.name.clone(),
            SymbolKind::CONSTANT,
            &self.position,
            Some(&self.info),
        );

        // Collect symbols from value expression
        self.value.collect_symbols(ctx);
    }
}

impl SymbolCollectable for Declaration<TypeInformation> {
    fn collect_symbols(&self, ctx: &SymbolCollectingContext) {
        // Add variable declaration
        ctx.add_definition(
            self.name.clone(),
            SymbolKind::VARIABLE,
            &self.position,
            Some(&self.info),
        );
    }
}

impl SymbolCollectable for Assignment<TypeInformation> {
    fn collect_symbols(&self, ctx: &SymbolCollectingContext) {
        // The target should be collected as a reference, not a definition
        self.target.collect_symbols(ctx);
        self.value.collect_symbols(ctx);
    }
}

impl SymbolCollectable for WhileLoop<TypeInformation> {
    fn collect_symbols(&self, ctx: &SymbolCollectingContext) {
        self.condition.collect_symbols(ctx);
        self.body.collect_symbols(ctx);
    }
}

impl SymbolCollectable for Instance<TypeInformation> {
    fn collect_symbols(&self, ctx: &SymbolCollectingContext) {
        self.expression.collect_symbols(ctx);
    }
}

impl SymbolCollectable for MethodDeclaration<TypeInformation> {
    fn collect_symbols(&self, ctx: &SymbolCollectingContext) {
        // Add method definition
        ctx.add_definition(
            self.name.clone(),
            SymbolKind::METHOD,
            &self.position,
            Some(&self.info),
        );

        // Collect symbols from parameters
        for param in &self.parameters {
            param.collect_symbols(ctx);
        }

        // Collect symbols from body
        if let Some(body) = &self.body {
            body.collect_symbols(ctx);
        }
    }
}

impl SymbolCollectable for Expression<TypeInformation> {
    fn collect_symbols(&self, ctx: &SymbolCollectingContext) {
        match self {
            Expression::Id(id) => id.collect_symbols(ctx),
            Expression::Num(_) => {}, // No symbols to collect
            Expression::Bool(_) => {}, // No symbols to collect
            Expression::Character(_) => {}, // No symbols to collect
            Expression::AstString(_) => {}, // No symbols to collect
            Expression::Lambda(lambda) => lambda.collect_symbols(ctx),
            Expression::If(if_expr) => if_expr.collect_symbols(ctx),
            Expression::Block(block) => block.collect_symbols(ctx),
            Expression::Array(array) => array.collect_symbols(ctx),
            Expression::Binary { left, right, .. } => {
                left.collect_symbols(ctx);
                right.collect_symbols(ctx);
            }
            Expression::Unary { target, .. } => target.collect_symbols(ctx),
            Expression::Call { target, arguments, .. } => {
                target.collect_symbols(ctx);
                for arg in arguments {
                    arg.collect_symbols(ctx);
                }
            }
            Expression::Postfix(postfix) => postfix.collect_symbols(ctx),
            Expression::StructInitialisation(struct_init) => struct_init.collect_symbols(ctx),
        }
    }
}

impl SymbolCollectable for Id<TypeInformation> {
    fn collect_symbols(&self, ctx: &SymbolCollectingContext) {
        // This is a reference to an existing symbol
        if let Some(symbol_id) = ctx.find_symbol_by_name(&self.name) {
            ctx.add_reference(symbol_id, &self.position, false);
        }
    }
}

impl SymbolCollectable for Lambda<TypeInformation> {
    fn collect_symbols(&self, ctx: &SymbolCollectingContext) {
        // Lambda parameters
        for param in &self.parameters {
            param.collect_symbols(ctx);
        }

        // Lambda body
        self.body.collect_symbols(ctx);
    }
}

impl SymbolCollectable for LambdaParameter<TypeInformation> {
    fn collect_symbols(&self, ctx: &SymbolCollectingContext) {
        // Add lambda parameter as a variable
        ctx.add_definition(
            self.name.clone(),
            SymbolKind::VARIABLE,
            &self.position,
            Some(&self.info),
        );
    }
}

impl SymbolCollectable for If<TypeInformation> {
    fn collect_symbols(&self, ctx: &SymbolCollectingContext) {
        self.condition.collect_symbols(ctx);
        self.then_branch.collect_symbols(ctx);
        if let Some(else_branch) = &self.else_branch {
            else_branch.collect_symbols(ctx);
        }
    }
}

impl SymbolCollectable for Block<TypeInformation> {
    fn collect_symbols(&self, ctx: &SymbolCollectingContext) {
        for statement in &self.statements {
            statement.collect_symbols(ctx);
        }
        if let Some(expr) = &self.expression {
            expr.collect_symbols(ctx);
        }
    }
}

impl SymbolCollectable for Array<TypeInformation> {
    fn collect_symbols(&self, ctx: &SymbolCollectingContext) {
        for element in &self.elements {
            element.collect_symbols(ctx);
        }
    }
}

impl SymbolCollectable for Postfix<TypeInformation> {
    fn collect_symbols(&self, ctx: &SymbolCollectingContext) {
        match self {
            Postfix::Index { expr, index, .. } => {
                expr.collect_symbols(ctx);
                index.collect_symbols(ctx);
            }
            Postfix::PropertyAccess { expr, property, .. } => {
                expr.collect_symbols(ctx);
                // Property access - this is a reference to a field or method
                if let Some(symbol_id) = ctx.find_symbol_by_name(&property.name) {
                    ctx.add_reference(symbol_id, &property.position, false);
                }
            }
            Postfix::Call { expr, args, .. } => {
                expr.collect_symbols(ctx);
                for arg in args {
                    arg.collect_symbols(ctx);
                }
            }
        }
    }
}

impl SymbolCollectable for StructInitialisation<TypeInformation> {
    fn collect_symbols(&self, ctx: &SymbolCollectingContext) {
        // Reference to the struct type
        if let Some(symbol_id) = ctx.find_symbol_by_name(&self.name) {
            ctx.add_reference(symbol_id, &self.position, false);
        }

        // Field initializations
        for field_init in &self.fields {
            field_init.collect_symbols(ctx);
        }
    }
}

impl SymbolCollectable for StructFieldInitialisation<TypeInformation> {
    fn collect_symbols(&self, ctx: &SymbolCollectingContext) {
        // Reference to the field name
        if let Some(symbol_id) = ctx.find_symbol_by_name(&self.name) {
            ctx.add_reference(symbol_id, &self.position, false);
        }

        // Collect symbols from the value expression
        self.value.collect_symbols(ctx);
    }
}