use std::sync::Arc;

use tower_lsp_server::lsp_types::{SymbolKind, Uri};

use crate::lexer::Span;
use crate::lsp::{PositionUtils, SymbolIndex, Definition, SymbolId};
use crate::typechecker::{Context, TypeInformation};
use crate::parser::ast::*;

/// Simplified symbol collecting context for basic functionality
#[derive(Debug, Clone)]
pub struct SimpleSymbolCollector {
    pub context: Context,
    pub symbol_index: Arc<SymbolIndex>,
    pub position_utils: Arc<PositionUtils>,
    pub uri: Uri,
}

impl SimpleSymbolCollector {
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

    /// Collect symbols from a list of top-level statements
    pub fn collect_from_statements(&self, statements: &[TopLevelStatement<TypeInformation>]) {
        for statement in statements {
            self.collect_from_top_level_statement(statement);
        }
    }

    fn collect_from_top_level_statement(&self, statement: &TopLevelStatement<TypeInformation>) {
        match statement {
            TopLevelStatement::Function(func) => self.collect_from_function(func),
            TopLevelStatement::StructDeclaration(struct_decl) => self.collect_from_struct(struct_decl),
            TopLevelStatement::Constant(constant) => self.collect_from_constant(constant),
            TopLevelStatement::Declaration(decl) => self.collect_from_declaration(decl),
            _ => {}, // Handle other cases later
        }
    }

    fn collect_from_function(&self, func: &Function<TypeInformation>) {
        let symbol_id = self.symbol_index.next_symbol_id();
        let range = self.position_utils.span_to_range(&func.position);

        let definition = Definition {
            symbol_id,
            name: func.id.name.clone(),
            kind: SymbolKind::FUNCTION,
            range,
            selection_range: self.position_utils.span_to_range(&func.id.position),
            type_info: Some(format!("function")), // Simplified for now
            parent: None,
            uri: self.uri.clone(),
        };

        self.symbol_index.add_definition(definition);

        // Collect parameters
        for param in &func.parameters {
            self.collect_from_function_parameter(param);
        }
    }

    fn collect_from_function_parameter(&self, param: &FunctionParameter<TypeInformation>) {
        let symbol_id = self.symbol_index.next_symbol_id();
        let range = self.position_utils.span_to_range(&param.position);

        let definition = Definition {
            symbol_id,
            name: param.name.name.clone(),
            kind: SymbolKind::VARIABLE,
            range,
            selection_range: self.position_utils.span_to_range(&param.name.position),
            type_info: Some(format!("parameter")),
            parent: None,
            uri: self.uri.clone(),
        };

        self.symbol_index.add_definition(definition);
    }

    fn collect_from_struct(&self, struct_decl: &StructDeclaration<TypeInformation>) {
        let symbol_id = self.symbol_index.next_symbol_id();
        let range = self.position_utils.span_to_range(&struct_decl.position);

        let definition = Definition {
            symbol_id,
            name: struct_decl.id.name.clone(),
            kind: SymbolKind::STRUCT,
            range,
            selection_range: self.position_utils.span_to_range(&struct_decl.id.position),
            type_info: Some(format!("struct")),
            parent: None,
            uri: self.uri.clone(),
        };

        self.symbol_index.add_definition(definition);

        // Collect fields
        for field in &struct_decl.fields {
            self.collect_from_struct_field(field);
        }
    }

    fn collect_from_struct_field(&self, field: &StructFieldDeclaration<TypeInformation>) {
        let symbol_id = self.symbol_index.next_symbol_id();
        let range = self.position_utils.span_to_range(&field.position);

        let definition = Definition {
            symbol_id,
            name: field.name.name.clone(),
            kind: SymbolKind::FIELD,
            range,
            selection_range: self.position_utils.span_to_range(&field.name.position),
            type_info: Some(format!("field")),
            parent: None,
            uri: self.uri.clone(),
        };

        self.symbol_index.add_definition(definition);
    }

    fn collect_from_constant(&self, constant: &Constant<TypeInformation>) {
        let symbol_id = self.symbol_index.next_symbol_id();
        let range = self.position_utils.span_to_range(&constant.position);

        let definition = Definition {
            symbol_id,
            name: constant.id.name.clone(),
            kind: SymbolKind::CONSTANT,
            range,
            selection_range: self.position_utils.span_to_range(&constant.id.position),
            type_info: Some(format!("constant")),
            parent: None,
            uri: self.uri.clone(),
        };

        self.symbol_index.add_definition(definition);
    }

    fn collect_from_declaration(&self, decl: &Declaration<TypeInformation>) {
        let symbol_id = self.symbol_index.next_symbol_id();
        let range = self.position_utils.span_to_range(&decl.position);

        let definition = Definition {
            symbol_id,
            name: decl.name.name.clone(),
            kind: SymbolKind::VARIABLE,
            range,
            selection_range: self.position_utils.span_to_range(&decl.name.position),
            type_info: Some(format!("variable")),
            parent: None,
            uri: self.uri.clone(),
        };

        self.symbol_index.add_definition(definition);
    }
}