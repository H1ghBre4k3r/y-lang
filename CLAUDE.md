# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Y Lang is an experimental programming language implementation in Rust, focusing on functional programming concepts where everything is treated as an expression. This is a personal project by Louis Meyer to "restart" his programming language (formerly known as `Y`).

## Build System & Commands

The project uses Just as a task runner alongside Cargo. Common commands:

**Just Commands** (preferred):
- `just build` or `just b` - Build the project
- `just build-release` or `just br` - Build in release mode  
- `just test` or `just t` - Run tests across the workspace
- `just bins` - Build all binaries
- `just bins-release` - Build all binaries in release mode
- `just watch` - Watch for changes and rebuild binaries

**Direct Cargo Commands**:
- `cargo build` - Build the workspace
- `cargo test --workspace` - Run all tests
- `cargo run --bin yc -- <file>` - Run the compiler on a file
- `cargo run --bin yfmt -- <file>` - Run the formatter on a file
- `cargo run --bin yls` - Run the language server

## Project Architecture

### Workspace Structure
- **Root crate** (`y-lang`): Contains CLI interface and three binaries
- **why_lib**: Core language implementation library
- **lex_derive**: Procedural macro crate for lexer generation

### Binaries
- **yc**: Main compiler binary (`src/bin/yc.rs`)
- **yfmt**: Code formatter (`src/bin/yfmt.rs`)  
- **yls**: Language server (`src/bin/yls.rs`)

### Core Library Architecture (`crates/why_lib/`)

The language implementation follows a traditional compiler pipeline:

1. **Grammar** (`src/grammar.rs`): Uses rust-sitter to define language grammar with procedural macros. The grammar defines the concrete syntax tree structure.

2. **Lexer** (`src/lexer/`): Tokenizes source code using pattern matching and regex-based token recognition.

3. **Parser** (`src/parser/`): Transforms tokens into an Abstract Syntax Tree (AST). Contains:
   - `ast/`: AST node definitions organized by statement and expression types
   - `combinators.rs`: Parser combinator utilities
   - `parse_state.rs`: Manages parsing state and error handling

4. **Type Checker** (`src/typechecker/`): Performs semantic analysis and type checking:
   - Type inference and validation
   - Scope management and symbol resolution
   - Validates main function signature
   - Two-phase checking: shallow check (declarations) then full check

5. **Formatter** (`src/formatter/`): Pretty-prints AST back to source code

### Key Design Patterns

**Grammar-Driven Development**: The project uses rust-sitter macros to define grammar rules directly in Rust, which generates both parser and tree-sitter grammar files.

**Expression-Centric Language**: The language treats most constructs as expressions (if/else, blocks, function calls, etc.) rather than statements.

**Two-Phase Type Checking**: First pass handles struct declarations and function signatures, second pass performs full type checking with proper dependency resolution.

**Workspace Organization**: Clean separation between language implementation (why_lib) and CLI tools, allowing the core library to be reused.

## Language Features

Based on the grammar and AST, Y Lang supports:
- Variables with optional mutability (`let mut`)
- Functions as first-class values with lambda syntax
- Control flow: if/else expressions, while loops
- Data structures: structs with field declarations
- Type annotations and inference
- Binary operations with operator precedence
- Arrays and property access
- Comments and string/character literals

## Development Workflow

When modifying the language:

1. **Grammar Changes**: Edit `crates/why_lib/src/grammar.rs` - the build system will regenerate parsers automatically via `build.rs`

2. **AST Changes**: Update corresponding AST nodes in `crates/why_lib/src/parser/ast/`

3. **Type System Changes**: Modify type checking logic in `crates/why_lib/src/typechecker/`

4. **Testing**: The project has unit tests throughout - run `just test` to ensure changes don't break existing functionality

5. **Formatting**: Use `just build` to verify everything compiles after changes

## Important Notes

- The project is in active development - documentation and functionality are incomplete
- Grammar uses rust-sitter which provides both Rust parsing and tree-sitter compatibility
- Main function validation is enforced by the type checker
- Error handling uses custom error types with span information for precise error reporting
- The formatter can both pretty-print to stdout and write to files