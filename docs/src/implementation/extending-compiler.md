# Extending the Compiler

This guide provides comprehensive instructions for extending the Y language compiler with new features. Whether you're adding language constructs, improving existing functionality, or implementing optimization passes, this document will walk you through the process step-by-step.

## Table of Contents

- [Development Setup](#development-setup)
- [Adding Language Features](#adding-language-features)
- [Compiler Architecture](#compiler-architecture)
- [Step-by-Step Guides](#step-by-step-guides)
- [Testing Your Changes](#testing-your-changes)
- [Performance Considerations](#performance-considerations)
- [Common Patterns](#common-patterns)
- [Debugging Tips](#debugging-tips)

## Development Setup

### Prerequisites

Before you begin extending the compiler, ensure you have the proper development environment:

```bash
# Install Rust toolchain (latest stable)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Install development dependencies
cargo install just
cargo install mdbook  # For documentation
cargo install cargo-watch  # For development workflow

# Clone and build the project
git clone https://github.com/H1ghBre4k3r/y-lang.git
cd y-lang
just build
```

### Development Workflow

The Y compiler uses `just` as a command runner:

```bash
# Build the compiler
just build           # Debug build
just build-release   # Release build

# Run tests
just test           # All tests
just test parser    # Specific module tests

# Watch for changes during development
just watch          # Rebuild on file changes

# Run examples
cargo run --bin yc examples/simple.why -o out/simple
./out/simple
```

### Project Structure

Understanding the codebase organization:

```text
y-lang/
├── crates/
│   ├── why_lib/           # Core compiler library
│   │   ├── src/
│   │   │   ├── lexer/     # Tokenization
│   │   │   ├── parser/    # AST generation
│   │   │   ├── typechecker/ # Type checking
│   │   │   ├── codegen/   # LLVM code generation
│   │   │   ├── formatter/ # Code formatting
│   │   │   └── grammar.rs # Language grammar
│   │   └── Cargo.toml
│   └── lex_derive/        # Procedural macros for lexer
├── src/
│   ├── bin/
│   │   └── yc.rs         # Compiler binary
│   └── lib.rs            # CLI interface
├── examples/             # Y language examples
├── docs/                # Documentation
└── justfile             # Build commands
```

## Adding Language Features

### Overview of the Process

Adding a new language feature involves changes across multiple compiler stages:

1. **Grammar Definition** - Define syntax rules
2. **Lexer Updates** - Add new tokens (if needed)
3. **Parser Changes** - Handle new syntax in AST
4. **Type Checking** - Add type rules for new feature
5. **Code Generation** - Emit LLVM IR for new construct
6. **Formatter Support** - Pretty-print new syntax
7. **Testing** - Comprehensive test coverage

### Example: Adding a `repeat` Loop

Let's walk through adding a `repeat N times { ... }` construct to the language.

#### Step 1: Grammar Definition

First, update the grammar in `crates/why_lib/src/grammar.rs`:

```rust
// Add to Statement enum
pub enum Statement {
    // ... existing variants
    RepeatLoop(Spanned<RepeatLoop>),
}

// Define the new construct
#[derive(Debug, Clone)]
pub struct RepeatLoop {
    #[rust_sitter::leaf(text = "repeat")]
    pub _repeat: (),
    pub count: Spanned<Expression>,
    #[rust_sitter::leaf(text = "times")]
    pub _times: (),
    pub body: Spanned<Block>,
}
```

#### Step 2: Lexer Updates

If new keywords are needed, they're automatically handled by the grammar. For `repeat` and `times`, add them to the lexer token definitions if not already present.

#### Step 3: Parser Changes

Add AST representation in `crates/why_lib/src/parser/ast/statement/`:

```rust
// In statement/mod.rs, add new file
mod repeat_loop;
pub use repeat_loop::*;

// Create statement/repeat_loop.rs
use crate::{grammar, lexer::Span, parser::ast::*};

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct RepeatLoop<T> {
    pub count: Expression<T>,
    pub body: Block<T>,
    pub position: Span,
    pub info: T,
}

impl<T: Default> FromGrammar<grammar::RepeatLoop> for RepeatLoop<T> {
    fn transform(repeat_loop: grammar::RepeatLoop, source: &str) -> Self {
        RepeatLoop {
            count: Expression::transform(repeat_loop.count.inner, source),
            body: Block::transform(repeat_loop.body.inner, source),
            position: Span::new(repeat_loop.span(), source),
            info: T::default(),
        }
    }
}
```

Update the main Statement enum:

```rust
// In statement/mod.rs
pub enum Statement<T> {
    // ... existing variants
    RepeatLoop(RepeatLoop<T>),
}

// Add transformation case
impl<T: Default> FromGrammar<grammar::Statement> for Statement<T> {
    fn transform(statement: grammar::Statement, source: &str) -> Self {
        match statement {
            // ... existing cases
            grammar::Statement::RepeatLoop(repeat) => {
                Statement::RepeatLoop(RepeatLoop::transform(repeat.inner, source))
            }
        }
    }
}
```

#### Step 4: Type Checking

Add type checking logic in `crates/why_lib/src/typechecker/typed_ast/statement/`:

```rust
// In the appropriate typed_ast file
impl TypeCheckable for RepeatLoop<()> {
    type Typed = RepeatLoop<TypeInformation>;

    fn check(self, ctx: &mut Context) -> TypeResult<Self::Typed> {
        // Check that count expression is an integer
        let typed_count = self.count.check(ctx)?;

        // Verify count is integer type
        if let Some(count_type) = typed_count.info.type_id.borrow().as_ref() {
            if !count_type.does_eq(&Type::Integer) {
                return Err(TypeCheckError::TypeMismatch {
                    expected: Type::Integer,
                    found: count_type.clone(),
                    position: typed_count.position(),
                });
            }
        }

        // Type check body in new scope
        ctx.enter_scope();
        let typed_body = self.body.check(ctx)?;
        ctx.exit_scope();

        Ok(RepeatLoop {
            count: typed_count,
            body: typed_body,
            position: self.position,
            info: TypeInformation {
                type_id: Rc::new(RefCell::new(Some(Type::Void))),
                context: ctx.clone(),
            },
        })
    }
}
```

#### Step 5: Code Generation

Add LLVM code generation in `crates/why_lib/src/codegen/statements/`:

```rust
// Add to statement codegen
impl CodeGen for RepeatLoop<ValidatedTypeInformation> {
    fn codegen(&self, ctx: &mut CodegenContext) -> Result<(), CodegenError> {
        let count_value = self.count.codegen(ctx)?;

        // Create basic blocks
        let loop_cond = ctx.context.append_basic_block(ctx.current_fn(), "repeat_cond");
        let loop_body = ctx.context.append_basic_block(ctx.current_fn(), "repeat_body");
        let loop_end = ctx.context.append_basic_block(ctx.current_fn(), "repeat_end");

        // Initialize counter variable
        let counter_type = ctx.context.i64_type();
        let counter = ctx.builder.build_alloca(counter_type, "repeat_counter")?;
        let zero = counter_type.const_int(0, false);
        ctx.builder.build_store(counter, zero)?;

        // Jump to condition check
        ctx.builder.build_unconditional_branch(loop_cond)?;

        // Loop condition block
        ctx.builder.position_at_end(loop_cond);
        let current_count = ctx.builder.build_load(counter_type, counter, "current_count")?;
        let condition = ctx.builder.build_int_compare(
            IntPredicate::ULT,
            current_count.into_int_value(),
            count_value.into_int_value(),
            "repeat_cond"
        )?;
        ctx.builder.build_conditional_branch(condition, loop_body, loop_end)?;

        // Loop body block
        ctx.builder.position_at_end(loop_body);
        self.body.codegen(ctx)?;

        // Increment counter
        let current = ctx.builder.build_load(counter_type, counter, "current")?;
        let one = counter_type.const_int(1, false);
        let incremented = ctx.builder.build_int_add(
            current.into_int_value(),
            one,
            "incremented"
        )?;
        ctx.builder.build_store(counter, incremented)?;

        // Jump back to condition
        ctx.builder.build_unconditional_branch(loop_cond)?;

        // Position at end block for subsequent code
        ctx.builder.position_at_end(loop_end);

        Ok(())
    }
}
```

#### Step 6: Formatter Support

Add formatting in `crates/why_lib/src/formatter/statement/`:

```rust
impl<T> Format for RepeatLoop<T> {
    fn format(&self, ctx: &mut FormatterContext) -> Result<(), std::fmt::Error> {
        ctx.write("repeat ")?;
        self.count.format(ctx)?;
        ctx.write(" times ")?;
        self.body.format(ctx)?;
        Ok(())
    }
}
```

#### Step 7: Testing

Create comprehensive tests:

```rust
// Unit test
#[test]
fn test_repeat_loop_parsing() {
    let source = "repeat 3 times { print(\"hello\"); }";
    let ast = parse_statement(source).unwrap();

    match ast {
        Statement::RepeatLoop(repeat) => {
            // Verify structure
            assert!(matches!(repeat.count, Expression::Num(_)));
            assert_eq!(repeat.body.statements.len(), 1);
        }
        _ => panic!("Expected repeat loop"),
    }
}

// Integration test
#[test]
fn test_repeat_loop_codegen() {
    let source = r#"
        fn main(): void {
            repeat 2 times {
                print("iteration");
            }
        }
    "#;

    let result = compile_and_run(source);
    assert_eq!(result.output, "iteration\niteration\n");
}
```

## Compiler Architecture

### Compilation Pipeline

Understanding the flow of data through the compiler:

```text
Source Code
    ↓
Lexer (Tokenization)
    ↓
Grammar Parser (rust-sitter)
    ↓
AST Transformation
    ↓
Type Checking
    ↓
Type Validation
    ↓
Code Generation (LLVM)
    ↓
Machine Code
```

### Module Responsibilities

**Lexer** (`src/lexer/`)
- Converts source text to tokens
- Tracks source positions
- Handles whitespace and comments

**Parser** (`src/parser/`)
- Converts tokens to AST
- Implements grammar rules
- Preserves source positions

**Type Checker** (`src/typechecker/`)
- Infers and validates types
- Manages scopes and contexts
- Reports type errors

**Code Generator** (`src/codegen/`)
- Emits LLVM IR
- Handles optimization
- Manages function calls and closures

**Formatter** (`src/formatter/`)
- Pretty-prints source code
- Preserves programmer intent
- Maintains consistent style

### Data Flow Patterns

**AST Type Parameters**
- `()` - Untyped AST from parser
- `TypeInformation` - Partially typed AST
- `ValidatedTypeInformation` - Fully typed AST

**Error Handling**
- Each stage has specific error types
- Errors include source position information
- Recovery strategies maintain partial progress

**Context Threading**
- Type checking context threaded through recursion
- Code generation context manages LLVM state
- Formatter context tracks indentation and output

## Step-by-Step Guides

### Adding a New Expression Type

Example: Adding conditional expressions `condition ? true_expr : false_expr`

#### 1. Grammar Definition
```rust
pub enum Expression {
    // ... existing variants
    Conditional(Spanned<ConditionalExpression>),
}

pub struct ConditionalExpression {
    pub condition: Spanned<Expression>,
    #[rust_sitter::leaf(text = "?")]
    pub _question: (),
    pub true_expr: Spanned<Expression>,
    #[rust_sitter::leaf(text = ":")]
    pub _colon: (),
    pub false_expr: Spanned<Expression>,
}
```

#### 2. AST Implementation
```rust
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Conditional<T> {
    pub condition: Box<Expression<T>>,
    pub true_expr: Box<Expression<T>>,
    pub false_expr: Box<Expression<T>>,
    pub position: Span,
    pub info: T,
}
```

#### 3. Type Checking
```rust
impl TypeCheckable for Conditional<()> {
    type Typed = Conditional<TypeInformation>;

    fn check(self, ctx: &mut Context) -> TypeResult<Self::Typed> {
        let condition = self.condition.check(ctx)?;
        let true_expr = self.true_expr.check(ctx)?;
        let false_expr = self.false_expr.check(ctx)?;

        // Condition must be boolean
        if !condition.info.type_id.borrow().as_ref().unwrap().does_eq(&Type::Boolean) {
            return Err(TypeCheckError::TypeMismatch { /* ... */ });
        }

        // Both branches must have same type
        let true_type = true_expr.info.type_id.borrow().clone().unwrap();
        let false_type = false_expr.info.type_id.borrow().clone().unwrap();

        if !true_type.does_eq(&false_type) {
            return Err(TypeCheckError::TypeMismatch { /* ... */ });
        }

        Ok(Conditional {
            condition,
            true_expr,
            false_expr,
            position: self.position,
            info: TypeInformation {
                type_id: Rc::new(RefCell::new(Some(true_type))),
                context: ctx.clone(),
            },
        })
    }
}
```

#### 4. Code Generation
```rust
impl CodeGen for Conditional<ValidatedTypeInformation> {
    fn codegen(&self, ctx: &mut CodegenContext) -> Result<BasicValueEnum, CodegenError> {
        let condition_value = self.condition.codegen(ctx)?;

        let true_block = ctx.context.append_basic_block(ctx.current_fn(), "cond_true");
        let false_block = ctx.context.append_basic_block(ctx.current_fn(), "cond_false");
        let merge_block = ctx.context.append_basic_block(ctx.current_fn(), "cond_merge");

        // Branch on condition
        ctx.builder.build_conditional_branch(
            condition_value.into_int_value(),
            true_block,
            false_block
        )?;

        // True branch
        ctx.builder.position_at_end(true_block);
        let true_value = self.true_expr.codegen(ctx)?;
        ctx.builder.build_unconditional_branch(merge_block)?;
        let true_block_end = ctx.builder.get_insert_block().unwrap();

        // False branch
        ctx.builder.position_at_end(false_block);
        let false_value = self.false_expr.codegen(ctx)?;
        ctx.builder.build_unconditional_branch(merge_block)?;
        let false_block_end = ctx.builder.get_insert_block().unwrap();

        // Merge block with phi node
        ctx.builder.position_at_end(merge_block);
        let phi = ctx.builder.build_phi(true_value.get_type(), "cond_result")?;
        phi.add_incoming(&[(&true_value, true_block_end), (&false_value, false_block_end)]);

        Ok(phi.as_basic_value())
    }
}
```

### Adding Built-in Functions

Example: Adding a `len()` function for arrays

#### 1. Built-in Registration
```rust
// In type checker initialization
impl Context {
    pub fn new() -> Self {
        let mut ctx = Context::default();
        ctx.register_builtin_functions();
        ctx
    }

    fn register_builtin_functions(&mut self) {
        // Register len function
        let len_type = Type::Function {
            params: vec![Type::Array(Box::new(Type::Unknown))], // Generic array
            return_value: Box::new(Type::Integer),
        };
        self.scope.register_function("len", len_type);
    }
}
```

#### 2. Code Generation
```rust
// In function call codegen
impl CodeGen for FunctionCall<ValidatedTypeInformation> {
    fn codegen(&self, ctx: &mut CodegenContext) -> Result<BasicValueEnum, CodegenError> {
        match self.name.as_str() {
            "len" => {
                // Built-in len function
                let array_arg = self.arguments[0].codegen(ctx)?;
                let array_ptr = array_arg.into_pointer_value();

                // Arrays have length as first field
                let len_ptr = ctx.builder.build_struct_gep(
                    ctx.get_array_type(),
                    array_ptr,
                    0, // Length field
                    "array_len_ptr"
                )?;

                let length = ctx.builder.build_load(
                    ctx.context.i64_type(),
                    len_ptr,
                    "array_length"
                )?;

                Ok(length)
            }
            _ => {
                // Regular function call
                self.codegen_regular_call(ctx)
            }
        }
    }
}
```

## Testing Your Changes

### Test Categories

**Unit Tests**
- Test individual components in isolation
- Fast feedback during development
- Located next to source code

**Integration Tests**
- Test full compilation pipeline
- Verify interaction between components
- Located in `tests/` directory

**Example Tests**
- Test real Y language programs
- Ensure examples continue working
- Run with `just test-examples`

### Writing Good Tests

#### Test Structure
```rust
#[test]
fn test_feature_name_scenario() {
    // Arrange
    let input = "test input";
    let expected = /* expected result */;

    // Act
    let result = function_under_test(input);

    // Assert
    assert_eq!(result, expected);
}
```

#### Property-Based Testing
```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn parsing_preserves_semantics(source in any_valid_program()) {
        let ast = parse_program(source.clone()).unwrap();
        let formatted = format_program(&ast).unwrap();
        let reparsed = parse_program(formatted).unwrap();

        assert_eq!(ast, reparsed);
    }
}
```

#### Error Testing
```rust
#[test]
fn test_type_error_reporting() {
    let source = r#"
        fn main(): i64 {
            return "string"; // Type error
        }
    "#;

    let result = compile_program(source);
    assert!(result.is_err());

    let error = result.unwrap_err();
    assert!(matches!(error, CompileError::TypeError(_)));
    assert!(error.to_string().contains("Expected i64, found String"));
}
```

## Performance Considerations

### Compilation Speed

**Hot Paths**
- Type checking inner loops
- AST traversal patterns
- String allocations during parsing

**Optimization Strategies**
- Use `Rc<RefCell<>>` for shared data
- Minimize string allocations
- Cache expensive computations
- Use iterators instead of collecting vectors

### Memory Usage

**AST Size**
- Each node carries type information
- Position information adds overhead
- Consider using interned strings for identifiers

**LLVM Integration**
- LLVM contexts are expensive to create
- Reuse contexts when possible
- Clean up temporary values

### Generated Code Quality

**Optimization Passes**
- LLVM provides most optimizations
- Focus on generating clean IR
- Let LLVM handle low-level optimizations

**Debug Information**
- Include source positions in LLVM metadata
- Support for debugger integration
- Maintain mapping from IR to source

## Common Patterns

### Visitor Pattern
```rust
trait Visitor<T> {
    fn visit_expression(&mut self, expr: &Expression<T>) -> VisitorResult;
    fn visit_statement(&mut self, stmt: &Statement<T>) -> VisitorResult;
}

struct TypeCollector {
    types: Vec<Type>,
}

impl<T> Visitor<T> for TypeCollector {
    fn visit_expression(&mut self, expr: &Expression<T>) -> VisitorResult {
        // Collect types from expression
        match expr {
            Expression::Id(id) => {
                if let Some(typ) = &id.info.type_id {
                    self.types.push(typ.clone());
                }
            }
            // ... other cases
        }
        Ok(())
    }
}
```

### Error Recovery
```rust
impl Parser {
    fn parse_with_recovery(&mut self) -> Result<Ast, Vec<ParseError>> {
        let mut ast = Vec::new();
        let mut errors = Vec::new();

        while !self.at_end() {
            match self.parse_statement() {
                Ok(stmt) => ast.push(stmt),
                Err(e) => {
                    errors.push(e);
                    self.recover_to_statement_boundary();
                }
            }
        }

        if errors.is_empty() {
            Ok(ast)
        } else {
            Err(errors)
        }
    }
}
```

### Builder Pattern
```rust
pub struct FunctionBuilder<T> {
    name: String,
    parameters: Vec<Parameter<T>>,
    return_type: Option<Type>,
    body: Option<Block<T>>,
}

impl<T> FunctionBuilder<T> {
    pub fn new(name: impl Into<String>) -> Self {
        FunctionBuilder {
            name: name.into(),
            parameters: Vec::new(),
            return_type: None,
            body: None,
        }
    }

    pub fn parameter(mut self, name: impl Into<String>, typ: Type) -> Self {
        self.parameters.push(Parameter::new(name, typ));
        self
    }

    pub fn returns(mut self, typ: Type) -> Self {
        self.return_type = Some(typ);
        self
    }

    pub fn body(mut self, body: Block<T>) -> Self {
        self.body = Some(body);
        self
    }

    pub fn build(self) -> Function<T> {
        Function {
            name: self.name,
            parameters: self.parameters,
            return_type: self.return_type,
            body: self.body.expect("Function must have a body"),
            // ... other fields
        }
    }
}
```

## Debugging Tips

### Compiler Debugging

**AST Inspection**
```rust
// Print AST structure
println!("{:#?}", ast);

// Serialize to JSON for inspection
let json = serde_json::to_string_pretty(&ast)?;
println!("{}", json);
```

**Type Checking Debug**
```rust
// Enable type checking debug output
RUST_LOG=y_lang::typechecker=debug cargo run -- input.why

// Add debug prints in type checker
eprintln!("Checking expression: {:?}", expr);
eprintln!("Inferred type: {:?}", inferred_type);
```

**LLVM IR Inspection**
```rust
// Emit LLVM IR to file
cargo run -- input.why --emit-llvm -o output.ll

// View generated IR
cat output.ll

// Use LLVM tools for analysis
opt -analyze -print-cfg output.ll
```

### Common Issues

**Missing Pattern Matches**
- Rust compiler will catch incomplete pattern matches
- Use `#[warn(unreachable_patterns)]` to catch redundant cases
- Consider using `#[non_exhaustive]` for extensible enums

**Memory Leaks in LLVM**
- LLVM values are managed by context
- Don't hold references to LLVM values across context destruction
- Use LLVM's built-in memory management

**Type Inference Loops**
- Avoid creating circular type dependencies
- Use occurs check in unification algorithm
- Implement cycle detection in complex type operations

### IDE Integration

**rust-analyzer Setup**
```json
// .vscode/settings.json
{
    "rust-analyzer.cargo.features": ["dev"],
    "rust-analyzer.checkOnSave.command": "clippy",
    "rust-analyzer.procMacro.enable": true
}
```

**Debugging Configuration**
```json
// .vscode/launch.json
{
    "type": "lldb",
    "request": "launch",
    "name": "Debug Compiler",
    "cargo": {
        "args": ["build", "--bin", "yc"],
        "filter": {
            "name": "yc",
            "kind": "bin"
        }
    },
    "args": ["examples/simple.why", "-o", "out/simple"],
    "cwd": "${workspaceFolder}"
}
```

This guide provides a comprehensive foundation for extending the Y language compiler. The modular architecture makes it straightforward to add new features while maintaining code quality and performance.