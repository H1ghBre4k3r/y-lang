# Lexing and Parsing

This chapter provides detailed technical documentation for the lexing and parsing implementation in the Y language compiler. These components form the foundation of the compilation pipeline, transforming source code text into structured Abstract Syntax Trees (ASTs).

## Table of Contents

- [Architecture Overview](#architecture-overview)
- [Lexical Analysis](#lexical-analysis)
- [Parsing Implementation](#parsing-implementation)
- [AST Design](#ast-design)
- [Error Handling](#error-handling)
- [Integration Points](#integration-points)
- [Performance Considerations](#performance-considerations)
- [Developer Guide](#developer-guide)

## Architecture Overview

The Y language employs a two-stage front-end architecture that separates lexical analysis from syntactic analysis:

```text
Source Code → Lexer → Token Stream → Parser → AST → Type Checker
```

### Key Components

1. **Lexer** (`crates/why_lib/src/lexer/`)
   - Tokenizes source code using regex-based pattern matching
   - Tracks precise source positions for error reporting
   - Handles whitespace, comments, and Unicode correctly

2. **Grammar** (`crates/why_lib/src/grammar.rs`)
   - Defines Y language syntax using rust-sitter
   - Automatically generates parsing logic from grammar specification
   - Provides incremental parsing capabilities

3. **Parser** (`crates/why_lib/src/parser/`)
   - Transforms grammar-based parse trees to typed ASTs
   - Implements position-preserving transformation
   - Supports generic type parameters for compilation stages

4. **AST** (`crates/why_lib/src/parser/ast/`)
   - Strongly-typed representation of Y language constructs
   - Generic over type information stages
   - Supports serialization for tooling integration

## Lexical Analysis

### Token Recognition System

The lexer uses a sophisticated pattern-matching system built on procedural macros:

```rust
// Example token definitions (simplified)
#[derive(Token)]
pub enum Token {
    #[pattern(r"\d+")]
    Integer { value: u64, position: Span },

    #[pattern(r"[a-zA-Z_][a-zA-Z0-9_]*")]
    Id { value: String, position: Span },

    #[keyword("fn")]
    FnKeyword { position: Span },
}
```

#### Key Features

- **Longest Match**: Always selects the longest possible token match
- **Priority System**: Keywords take precedence over identifiers
- **Position Tracking**: Every token includes precise source location
- **Error Recovery**: Clear error messages for unrecognized input

### Position Tracking

Every token carries comprehensive position information:

```rust
pub struct Span {
    pub start: (usize, usize),  // (line, column)
    pub end: (usize, usize),    // (line, column)
    pub source: String,         // Original source text
}
```

Position tracking enables:
- Precise error reporting with source context
- IDE integration (go-to-definition, hover, etc.)
- Source map generation for debugging
- Accurate refactoring tool support

### Performance Characteristics

- **Time Complexity**: O(n) where n is input length
- **Space Complexity**: O(t) where t is token count
- **Optimization**: Compiled regex patterns cached for reuse
- **Memory**: ~32 bytes per token including position data

## Parsing Implementation

### rust-sitter Integration

Y uses rust-sitter for parsing, which provides several advantages:

```rust
#[rust_sitter::grammar("ylang_grammar")]
mod ylang_grammar {
    #[rust_sitter::language]
    pub struct Program {
        #[rust_sitter::repeat]
        pub statements: Vec<Spanned<ToplevelStatement>>,
    }
}
```

#### Benefits of rust-sitter

- **Incremental Parsing**: Efficient re-parsing for language servers
- **Error Recovery**: Continues parsing after syntax errors
- **Tree Surgery**: Support for editing operations in IDEs
- **Performance**: Fast parsing with minimal memory allocation

### Grammar Definition

The Y language grammar defines syntax rules declaratively:

#### Expression Grammar
```rust
pub enum Expression {
    Boolean(Spanned<BooleanLiteral>),
    Identifier(Spanned<Identifier>),
    Number(Spanned<Number>),
    BinaryExpression(Spanned<BinaryExpression>),
    Lambda(Spanned<Lambda>),
    // ... other expression types
}
```

#### Statement Grammar
```rust
pub enum Statement {
    FunctionDeclaration(Spanned<FunctionDeclaration>),
    VariableDeclaration(Spanned<VariableDeclaration>),
    Assignment(Spanned<Assignment>),
    WhileStatement(Spanned<WhileStatement>),
    // ... other statement types
}
```

### AST Transformation

The parser transforms grammar types to strongly-typed AST nodes:

```rust
impl FromGrammar<grammar::Expression> for Expression<()> {
    fn transform(expr: grammar::Expression, source: &str) -> Self {
        match expr {
            grammar::Expression::Number(num) => {
                Expression::Num(Num::transform(num.inner, source))
            }
            grammar::Expression::Identifier(id) => {
                Expression::Id(Id::transform(id.inner, source))
            }
            // ... other transformations
        }
    }
}
```

This transformation:
- Converts grammar-specific types to domain types
- Preserves position information through Span
- Adds type parameter slots for later compilation stages
- Validates structural constraints

## AST Design

### Type Parameterization

The AST uses generic type parameters to track compilation progress:

```rust
// Stage 1: No type information
let ast: Vec<TopLevelStatement<()>> = parse_program(source);

// Stage 2: Partial type information
let typed_ast: Vec<TopLevelStatement<TypeInformation>> = type_check(ast);

// Stage 3: Complete type information
let validated_ast: Vec<TopLevelStatement<ValidatedTypeInformation>> = validate(typed_ast);
```

#### Benefits

- **Type Safety**: Compile-time verification of AST stage
- **Progressive Enhancement**: Type information added gradually
- **Tool Integration**: Different tools work with appropriate stages
- **Memory Efficiency**: Zero-cost abstractions via generics

### Node Categories

#### Expression Nodes
Value-producing constructs:
- **Literals**: `Num`, `Bool`, `Character`, `AstString`
- **Identifiers**: `Id` for variables and functions
- **Operations**: `Binary`, `Prefix`, `Postfix`
- **Control Flow**: `If`, `Block`
- **Functions**: `Lambda`, `Function`
- **Composite**: `Array`, `StructInitialisation`

#### Statement Nodes
Action-performing constructs:
- **Declarations**: Variable and function declarations
- **Assignments**: Variable modification
- **Control Flow**: `WhileLoop`, return statements
- **Type Definitions**: Struct declarations and instance blocks

### Position Preservation

Every AST node includes position information:

```rust
pub struct Function<T> {
    pub name: String,
    pub parameters: Vec<FunctionParameter<T>>,
    pub return_type: Option<TypeName>,
    pub body: Block<T>,
    pub position: Span,  // Position preserved
    pub info: T,         // Type information slot
}
```

## Error Handling

### Lexical Errors

The lexer provides detailed error information:

```rust
pub struct LexError(String);

// Example error message:
// "Failed to lex 'let x = @' at position 8; remaining '@'"
```

#### Error Context
- Exact position of failure
- Character that caused the error
- Remaining unparsed input
- Suggestions for common mistakes

### Parse Errors

The parser leverages rust-sitter's error recovery:

```rust
pub struct ParseError {
    pub message: String,
    pub position: Option<Span>,
}
```

#### Error Recovery Strategy
- Continue parsing after errors when possible
- Generate partial AST for better tooling support
- Provide context-sensitive error messages
- Suggest likely fixes based on common patterns

### Error Display

Both lexer and parser errors include rich context:

```text
Error: Unexpected character '@' at line 1, column 9
  |
1 | let x = @
  |         ^ Expected expression
```

## Integration Points

### Type Checker Integration

The parser provides a clean interface to the type checker:

```rust
pub fn parse_program(program: Program, source: &str) -> Vec<TopLevelStatement<()>> {
    // Transform grammar types to AST nodes
    let mut statements = vec![];
    for statement in program.statements {
        statements.push(TopLevelStatement::transform(statement, source));
    }
    statements
}
```

#### Handoff Guarantees
- Syntactically valid AST structure
- Complete position information
- Type parameter slots ready for type information
- Structural invariants maintained

### Language Server Integration

The parsing system supports language server features:

#### Incremental Parsing
- rust-sitter enables efficient re-parsing of modified regions
- Position mapping for editor-to-AST coordinate translation
- Error streaming for real-time diagnostics

#### IDE Features
- Syntax highlighting based on token types
- Bracket matching using AST structure
- Code folding based on block structure
- Symbol outline from declaration nodes

### Formatter Integration

The formatter consumes the same AST:

```rust
pub trait Format {
    fn format(&self, ctx: &mut FormatterContext) -> Result<(), std::fmt::Error>;
}

// Every AST node implements Format
impl<T> Format for Expression<T> {
    fn format(&self, ctx: &mut FormatterContext) -> Result<(), std::fmt::Error> {
        match self {
            Expression::Num(num) => num.format(ctx),
            Expression::Id(id) => id.format(ctx),
            // ... other cases
        }
    }
}
```

## Performance Considerations

### Parsing Performance

#### Time Complexity
- **Lexing**: O(n) where n is input length
- **Parsing**: O(n) for most constructs, O(n log n) worst case
- **AST Transform**: O(n) where n is number of nodes

#### Memory Usage
- **Tokens**: ~32 bytes per token
- **AST Nodes**: ~64-128 bytes per node depending on type
- **Position Data**: ~16 bytes per position
- **Total**: Approximately 3-5x source size in memory

### Optimization Strategies

#### Lexer Optimizations
- Compiled regex patterns cached across lexer instances
- Direct byte-level iteration for position tracking
- Minimal string allocation during tokenization

#### Parser Optimizations
- rust-sitter's incremental parsing for large files
- Lazy AST node construction when possible
- Efficient position tracking without additional allocations

#### Memory Optimizations
- Intern common strings (keywords, operators)
- Use compact representations for common node types
- Pool allocation for temporary objects

## Developer Guide

### Adding New Language Features

#### 1. Update Grammar
Add new syntax to `crates/why_lib/src/grammar.rs`:

```rust
pub enum Expression {
    // ... existing variants
    NewConstruct(Spanned<NewConstruct>),
}

pub struct NewConstruct {
    pub field1: Spanned<Type1>,
    pub field2: Spanned<Type2>,
}
```

#### 2. Add AST Node
Create corresponding AST type in `crates/why_lib/src/parser/ast/`:

```rust
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct NewConstruct<T> {
    pub field1: Type1<T>,
    pub field2: Type2<T>,
    pub position: Span,
    pub info: T,
}
```

#### 3. Implement Transformation
Add `FromGrammar` implementation:

```rust
impl<T: Default> FromGrammar<grammar::NewConstruct> for NewConstruct<T> {
    fn transform(construct: grammar::NewConstruct, source: &str) -> Self {
        NewConstruct {
            field1: Type1::transform(construct.field1, source),
            field2: Type2::transform(construct.field2, source),
            position: Span::new(construct.span(), source),
            info: T::default(),
        }
    }
}
```

#### 4. Update Expression Enum
Add the new variant to the main expression enum:

```rust
pub enum Expression<T> {
    // ... existing variants
    NewConstruct(NewConstruct<T>),
}
```

#### 5. Add Pattern Matching
Update all pattern matches to handle the new variant:
- Type checker implementation
- Code generator implementation
- Formatter implementation
- Any visitor patterns

### Debugging Tips

#### Lexer Debugging
- Use `Lexer::lex()` directly to inspect token stream
- Check position information with manual calculation
- Test edge cases with unusual Unicode characters

#### Parser Debugging
- Examine rust-sitter parse tree before AST transformation
- Use `serde` serialization to inspect AST structure
- Compare position information between grammar and AST nodes

#### AST Debugging
- Implement `Debug` for custom formatting of large ASTs
- Use position information to trace back to source
- Validate AST invariants in test cases

### Testing Guidelines

#### Test Categories
1. **Unit Tests**: Individual component testing
2. **Integration Tests**: Full pipeline validation
3. **Regression Tests**: Prevent behavior changes
4. **Performance Tests**: Parsing speed benchmarks

#### Example Test Structure
```rust
#[test]
fn test_parse_function_declaration() {
    let source = "fn foo(x: i64): i64 { return x + 1; }";
    let result = parse_program_string(source);

    assert!(result.is_ok());
    let ast = result.unwrap();
    assert_eq!(ast.len(), 1);

    match &ast[0] {
        TopLevelStatement::Function(func) => {
            assert_eq!(func.name, "foo");
            assert_eq!(func.parameters.len(), 1);
            // ... more assertions
        }
        _ => panic!("Expected function declaration"),
    }
}
```

This implementation provides a solid foundation for the Y language front-end, with excellent error handling, performance characteristics, and extensibility for future language features.
