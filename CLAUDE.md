# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a compiler for the Y programming language, written in Rust. The project uses a workspace structure with two main crates:
- `why_lib`: The core compiler library containing parsing, type checking, and code generation
- `lex_derive`: A procedural macro crate for lexer generation

## Build System and Commands

The project uses Just as a command runner. Key commands:

- `just build` or `just b` - Build the project
- `just build-release` or `just br` - Build in release mode  
- `just test` or `just t` - Run all tests
- `just bins` - Build all binaries
- `just watch` - Watch for changes and rebuild automatically
- `just install` - Install the compiler locally

Standard Cargo commands also work:
- `cargo build --workspace` - Build all crates
- `cargo test --workspace` - Run tests across all crates
- `cargo run --bin <binary_name>` - Run a specific binary

## Architecture

### Compiler Pipeline

The compiler follows a multi-stage pipeline:

1. **Lexing/Parsing** (`crates/why_lib/src/grammar.rs`) - Uses rust-sitter for parsing
2. **AST Generation** (`crates/why_lib/src/parser/ast/`) - Abstract syntax tree with separate modules for expressions and statements
3. **Type Checking** (`crates/why_lib/src/typechecker/`) - Type inference and validation
4. **Code Generation** (`crates/why_lib/src/codegen/`) - LLVM IR generation

### Key Components

- **Module System** (`crates/why_lib/src/lib.rs`) - The `Module<T>` struct represents compilation stages with generic type parameters
- **Parser** (`crates/why_lib/src/parser/`) - Converts parsed programs to typed AST nodes
- **Type System** - Uses `TypeInformation` and `ValidatedTypeInformation` for type checking phases
- **LLVM Integration** - Uses Inkwell for LLVM IR generation with support for multiple output formats

### Language Features

The Y language supports:
- Expression-oriented syntax (everything returns values)
- Functions as first-class citizens with lambda support: `\(x, y) => x + y`
- Struct types with instance methods
- Pattern matching with `match` expressions
- Type inference with optional explicit type annotations
- Control flow with `if` expressions and `while` loops

### File Structure

- `examples/` - Contains `.why` source files demonstrating language features
- `docs/src/` - Language documentation and specification
- Output files are generated in `out/` directory with hash-based names

### Dependencies

Key dependencies include:
- `rust-sitter` - Parser generation
- `inkwell` - LLVM bindings for code generation
- `tower-lsp-server` - Language Server Protocol support
- `clap` - Command line argument parsing