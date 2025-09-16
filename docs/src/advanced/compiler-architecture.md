# Compiler Architecture

This document provides an overview of the Y language compiler architecture, showing how the various components work together to transform Y source code into executable binaries.

## High-Level Architecture

The Y compiler follows a traditional multi-stage pipeline:

```
┌─────────────┐    ┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│   Source    │ -> │   Lexing/   │ -> │   Type      │ -> │    Code     │
│    Code     │    │   Parsing   │    │  Checking   │    │ Generation  │
│   (.why)    │    │             │    │             │    │   (LLVM)    │
└─────────────┘    └─────────────┘    └─────────────┘    └─────────────┘
                          │                   │                   │
                          v                   v                   v
                   ┌─────────────┐    ┌─────────────┐    ┌─────────────┐
                   │ Untyped AST │    │ Typed AST   │    │  LLVM IR    │
                   │             │    │ + Metadata  │    │             │
                   └─────────────┘    └─────────────┘    └─────────────┘
```

## Project Structure

### Workspace Organization

```
y-lang/
├── crates/
│   ├── why_lib/          # Core compiler library
│   │   ├── src/
│   │   │   ├── lib.rs            # Module definitions
│   │   │   ├── grammar.rs        # Tree-sitter grammar integration
│   │   │   ├── lexer/            # Lexical analysis
│   │   │   ├── parser/           # Syntax analysis & AST generation
│   │   │   ├── typechecker/      # Type inference & validation
│   │   │   └── codegen/          # LLVM IR generation
│   │   └── Cargo.toml
│   └── lex_derive/       # Procedural macros for lexer
└── src/                  # Binary targets
    ├── bin/
    │   ├── yc.rs         # Main compiler binary
    │   ├── yls.rs        # Language server
    │   └── yfmt.rs       # Code formatter
    └── lib.rs
```

### Module Hierarchy

The `why_lib` crate contains the core compiler implementation:

- **`lib.rs`**: Defines the main `Module<T>` abstraction and compilation stages
- **`grammar.rs`**: Integration with Tree-sitter for parsing
- **`lexer/`**: Token generation (minimal, mostly handled by Tree-sitter)
- **`parser/`**: AST generation from parse trees
- **`typechecker/`**: Type inference, validation, and semantic analysis
- **`codegen/`**: LLVM IR generation and optimization

## Compilation Stages

### Stage 1: Lexing and Parsing

**Files**: `grammar.rs`, `parser/`

The Y compiler uses [Tree-sitter](https://tree-sitter.github.io/) for parsing, which provides:
- Robust error recovery
- Incremental parsing for language servers
- Clear separation between syntax and semantics

#### Process:
1. **Source Input**: Raw Y source code
2. **Tree-sitter Parsing**: Generates concrete syntax tree (CST)
3. **AST Conversion**: Transform CST to typed AST nodes

#### Key Components:
- **`Module<Untyped>`**: Represents untyped program structure
- **AST Nodes**: Located in `parser/ast/` (expressions, statements, types)
- **Position Tracking**: Every node includes source location information

### Stage 2: Type Checking

**Files**: `typechecker/`

The type checker performs semantic analysis and type inference:

#### Process:
1. **Scope Management**: Build symbol tables for variables, functions, types
2. **Type Inference**: Deduce types for expressions and variables
3. **Validation**: Check type compatibility, variable usage, function calls
4. **Metadata Attachment**: Add type information to AST nodes

#### Closure-Specific Processing:
- **Capture Analysis**: Identify free variables in lambda expressions
- **Environment Planning**: Determine what variables need to be captured
- **Type Resolution**: Ensure closure types are properly inferred

```rust
// Simplified type checking flow
impl TypeCheck for Lambda<Untyped> {
    fn type_check(self, ctx: &mut TypeContext) -> Result<Lambda<Typed>, Error> {
        // 1. Type check lambda parameters and body
        let typed_body = self.body.type_check(ctx)?;

        // 2. Analyze captures (if any)
        let captures = analyze_captures(&typed_body, &self.parameters);

        // 3. Store capture information for codegen
        store_lambda_captures(lambda_id, captures);

        // 4. Return typed lambda with function type
        Lambda {
            body: typed_body,
            info: TypeInformation::function(param_types, return_type)
        }
    }
}
```

### Stage 3: Code Generation

**Files**: `codegen/`

The code generator transforms typed AST into LLVM IR:

#### Process:
1. **Context Setup**: Initialize LLVM context, module, and builder
2. **Type Lowering**: Convert Y types to LLVM types
3. **Function Generation**: Create LLVM functions for Y functions and lambdas
4. **Expression Codegen**: Generate IR for expressions and statements
5. **Optimization**: Apply LLVM optimization passes

#### Closure Code Generation Flow:

```rust
// Simplified closure codegen flow
impl CodeGen for Lambda<Typed> {
    fn codegen(&self, ctx: &CodegenContext) -> BasicValueEnum {
        // 1. Retrieve capture information from typechecker
        let captures = get_lambda_captures(self.id);

        if captures.is_empty() {
            // 2a. Non-capturing: generate standard function + null env
            self.codegen_non_capturing_lambda(ctx)
        } else {
            // 2b. Capturing: generate closure with environment
            self.codegen_capturing_lambda(ctx, &captures)
        }
    }
}
```

## Key Abstractions

### Module<T> - Compilation Stage Abstraction

The `Module<T>` type represents a Y program at different compilation stages:

```rust
pub struct Module<T> {
    pub items: Vec<Statement<T>>,
    // ... other fields
}

// Compilation stages
type UntypedModule = Module<Untyped>;
type TypedModule = Module<Typed>;
```

This enables:
- **Type Safety**: Prevents mixing typed and untyped AST nodes
- **Stage Validation**: Ensures proper compilation order
- **Clear Interfaces**: Each stage has specific input/output types

### AST Node Type Parameters

AST nodes are generic over type information:

```rust
pub struct Expression<T> {
    pub kind: ExpressionKind<T>,
    pub info: T,
}

// Type information variants
pub struct Untyped;
pub struct Typed { pub type_id: Type }
```

### Scope Management

The compiler uses hierarchical scopes for symbol resolution:

```rust
pub struct Scope<T> {
    variables: HashMap<String, T>,
    functions: HashMap<String, FunctionValue>,
    constants: HashMap<String, T>,
}

impl CodegenContext {
    pub fn enter_scope(&self) { /* ... */ }
    pub fn exit_scope(&self) { /* ... */ }
    pub fn find_variable(&self, name: &str) -> T { /* ... */ }
}
```

## Closure Integration Points

Closures touch every stage of the compiler:

### Parser Stage
- **Lambda Syntax**: `\(params) => body`
- **Function Types**: `(param_types) -> return_type`
- **Position Tracking**: Essential for capture analysis

### Typechecker Stage
- **Capture Analysis**: Identify free variables in lambda bodies
- **Type Inference**: Infer closure types from usage
- **Scope Validation**: Ensure captured variables are accessible

### Codegen Stage
- **Environment Creation**: Generate heap-allocated environment structs
- **Function Generation**: Create both standard and closure implementations
- **Call Site Handling**: Route direct vs indirect calls appropriately

## LLVM Integration

### IR Generation Strategy

The compiler generates LLVM IR with these principles:

1. **Direct Mapping**: Y constructs map naturally to LLVM constructs
2. **Optimization Friendly**: Generate IR that LLVM can optimize effectively
3. **Type Safety**: Maintain type information through to IR generation
4. **ABI Consistency**: Ensure consistent calling conventions

### Closure-Specific IR Patterns

#### Closure Struct Type
```llvm
%closure_type = type { ptr, ptr }  ; {function_ptr, env_ptr}
```

#### Environment Allocation
```llvm
%env_size = ptrtoint ptr getelementptr (%env_struct, ptr null, i32 1) to i64
%env_ptr = call ptr @malloc(i64 %env_size)
```

#### Closure Call
```llvm
%fn_ptr = extractvalue %closure_type %closure, 0
%env_ptr = extractvalue %closure_type %closure, 1
%result = call i64 %fn_ptr(ptr %env_ptr, i64 %arg)
```

## Build System Integration

### Just Commands

The project uses [Just](https://github.com/casey/just) for build automation:

- `just build` / `just b`: Build the compiler
- `just test` / `just t`: Run all tests
- `just watch`: Watch for changes and rebuild

### Cargo Workspace

The multi-crate structure enables:
- **Incremental Compilation**: Only changed crates rebuild
- **Modular Testing**: Each crate can have its own tests
- **Dependency Management**: Clear separation of concerns

## Testing Strategy

### Unit Tests
- **Parser Tests**: Verify AST generation from source
- **Typechecker Tests**: Validate type inference and error detection
- **Codegen Tests**: Check LLVM IR generation correctness

### Integration Tests
- **End-to-End**: Compile and execute Y programs
- **Example Programs**: Validate language features work together
- **Regression Tests**: Prevent breaking changes

### Closure-Specific Tests
- **Capture Analysis**: Verify correct variable detection
- **Environment Generation**: Check heap allocation and population
- **Call Conventions**: Ensure proper function calling

## Performance Considerations

### Compilation Speed
- **Incremental Parsing**: Tree-sitter enables fast re-parsing
- **Efficient Type Checking**: Scope-based symbol resolution
- **LLVM Optimization**: Leverage LLVM's proven optimization passes

### Runtime Performance
- **Direct Calls**: Named functions bypass closure overhead
- **Minimal Indirection**: Closure calls add only necessary overhead
- **Memory Efficiency**: Environment structs sized exactly for captures

### Memory Management
- **Heap Allocation**: Environments allocated as needed
- **No GC Overhead**: Simple allocation strategy
- **Future Work**: Reference counting for cleanup

## Language Server Integration

**File**: `src/bin/yls.rs`

The Y language server provides IDE support:
- **Syntax Highlighting**: Based on Tree-sitter grammar
- **Error Reporting**: Real-time compilation errors
- **Type Information**: Hover and completion support

The modular architecture enables the language server to reuse:
- **Parser**: For syntax analysis
- **Typechecker**: For semantic analysis and diagnostics
- **Position Tracking**: For accurate error locations

## Future Architecture Improvements

### Planned Enhancements
1. **Memory Management**: Add automatic cleanup for environments
2. **Optimization Passes**: Custom LLVM passes for Y-specific patterns
3. **Incremental Compilation**: Cache compilation artifacts
4. **Module System**: Support for multi-file programs

### Extensibility Points
- **AST Visitors**: Easy to add new analysis passes
- **Type System**: Designed for extension (generics, traits)
- **Codegen Backends**: LLVM abstraction allows alternative targets
- **Language Features**: Modular design supports feature additions

This architecture provides a solid foundation for a modern systems programming language while maintaining the flexibility to evolve and add new features.