# Type Checking

This chapter provides comprehensive technical documentation for the type checking implementation in the Y language compiler. The type checker ensures program safety by verifying type correctness and inferring types where possible, transforming untyped ASTs into fully type-annotated representations.

## Table of Contents

- [Architecture Overview](#architecture-overview)
- [Type System Design](#type-system-design)
- [Type Inference Algorithm](#type-inference-algorithm)
- [Scope and Context Management](#scope-and-context-management)
- [Error Handling and Recovery](#error-handling-and-recovery)
- [Implementation Details](#implementation-details)
- [Integration Points](#integration-points)
- [Performance Considerations](#performance-considerations)
- [Developer Guide](#developer-guide)

## Architecture Overview

The Y language type checker follows a multi-stage approach that gradually builds complete type information:

```text
Untyped AST → Type Inference → Type Validation → Validated AST → Code Generation
    (())         (TypeInfo)      (Validation)     (ValidatedInfo)     (LLVM)
```

### Core Components

1. **Type System** (`crates/why_lib/src/typechecker/types.rs`)
   - Defines the complete type universe for Y
   - Handles primitive, composite, and function types
   - Implements type equality and compatibility checking

2. **Type Checker** (`crates/why_lib/src/typechecker/mod.rs`)
   - Main type checking orchestration
   - Implements bidirectional type inference
   - Manages the type checking pipeline

3. **Context & Scope** (`crates/why_lib/src/typechecker/context.rs`, `scope.rs`)
   - Lexical scope management for variables and functions
   - Context passing for type checking phases
   - Symbol table management

4. **Typed AST** (`crates/why_lib/src/typechecker/typed_ast/`)
   - Type-aware AST nodes with type checking logic
   - Mirrors parser AST structure with type information
   - Implements type checking for each language construct

## Type System Design

### Type Universe

The Y type system supports a rich hierarchy of types:

```rust
#[derive(Clone, PartialEq, Eq, Hash)]
pub enum Type {
    // Primitive types
    Integer,                    // i64
    FloatingPoint,              // f64
    Boolean,                    // bool
    Character,                  // char
    String,                     // str
    Void,                       // void/unit type

    // Composite types
    Array(Box<Type>),           // [T]
    Tuple(Vec<Type>),           // (T1, T2, ...)
    Reference(Box<Type>),       // &T
    Struct(String, Vec<(String, Type)>), // struct Name { field: Type }

    // Function types
    Function {
        params: Vec<Type>,
        return_value: Box<Type>,
    },

    // Special types
    Unknown,                    // During inference
}
```

#### Type Properties

**Primitive Types**
- **Integer**: 64-bit signed integers with overflow checking
- **FloatingPoint**: IEEE 754 double precision floating point
- **Boolean**: True/false values with logical operations
- **Character**: UTF-8 characters with proper encoding
- **String**: Immutable string values with standard operations
- **Void**: Unit type for procedures without return values

**Composite Types**
- **Array**: Homogeneous collections with compile-time element type
- **Tuple**: Heterogeneous fixed-size collections
- **Reference**: Borrowed references to other types (future feature)
- **Struct**: Named product types with field access

**Function Types**
- **First-class**: Functions are values that can be passed and returned
- **Higher-order**: Functions can accept other functions as parameters
- **Closure support**: Lambda expressions with environment capture

### Type Compatibility

The type system implements sophisticated type checking rules:

```rust
impl Type {
    pub fn does_eq(&self, other: &Self) -> bool {
        match (self, other) {
            // Reference types are compatible with their referents
            (Self::Reference(l0), r0) => l0.as_ref() == r0,
            (l0, Self::Reference(r0)) => l0 == r0.as_ref(),

            // Structural equality for composite types
            (Self::Struct(l_name, l_fields), Self::Struct(r_name, r_fields)) => {
                l_name == r_name && l_fields == r_fields
            }

            // Function type compatibility
            (Self::Function { params: l_params, return_value: l_ret },
             Self::Function { params: r_params, return_value: r_ret }) => {
                l_params == r_params && l_ret == r_ret
            }

            // Nominal equality for primitive types
            _ => core::mem::discriminant(self) == core::mem::discriminant(other),
        }
    }
}
```

## Type Inference Algorithm

### Bidirectional Type Checking

The Y type checker implements bidirectional type checking, combining inference and checking modes:

#### Inference Mode (Bottom-up)
Infers types from expressions without expected types:
```rust
// Infer from literal
let x = 42;        // x: Integer (inferred)

// Infer from operations
let y = x + 10;    // y: Integer (inferred from operands)

// Infer from function calls
let z = some_func(x, y);  // z: ReturnType (inferred from function signature)
```

#### Checking Mode (Top-down)
Checks expressions against expected types:
```rust
// Check against expected parameter type
fn takes_int(param: Integer) { ... }
takes_int(42);     // 42 checked against Integer

// Check against expected return type
fn returns_string(): String {
    "hello"        // "hello" checked against String
}
```

### Type Variable System

The type checker uses type variables for gradual type resolution:

```rust
pub struct TypeInformation {
    pub type_id: Rc<RefCell<Option<Type>>>,  // Mutable type slot
    pub context: Context,                    // Scope information
}
```

#### Type Variable Lifecycle

1. **Creation**: Fresh type variables assigned to unknown types
2. **Constraint Generation**: Generate equality constraints from expressions
3. **Unification**: Solve constraints to determine concrete types
4. **Substitution**: Replace type variables with resolved types

### Constraint-Based Inference

The type checker generates and solves type constraints:

```rust
// Example: Binary expression type checking
fn check_binary_expr(
    left: &Expression<TypeInformation>,
    op: BinaryOperator,
    right: &Expression<TypeInformation>,
    ctx: &mut Context
) -> TypeResult<Type> {
    let left_type = left.infer_type(ctx)?;
    let right_type = right.infer_type(ctx)?;

    // Generate constraint: left_type == right_type
    if !left_type.does_eq(&right_type) {
        return Err(TypeCheckError::TypeMismatch {
            expected: left_type,
            found: right_type,
            position: /* ... */,
        });
    }

    // Return result type based on operator
    match op {
        BinaryOperator::Add | BinaryOperator::Sub => Ok(left_type),
        BinaryOperator::Eq | BinaryOperator::Ne => Ok(Type::Boolean),
        // ... other operators
    }
}
```

## Scope and Context Management

### Lexical Scoping

The type checker maintains precise lexical scope information:

```rust
#[derive(Debug, Clone)]
pub struct Context {
    pub scope: Scope,  // Current lexical scope
}

pub struct Scope {
    variables: HashMap<String, Rc<RefCell<Option<Type>>>>,
    functions: HashMap<String, Type>,
    parent: Option<Box<Scope>>,
}
```

#### Scope Operations

**Variable Resolution**
```rust
impl Scope {
    pub fn resolve_name(&self, name: &str) -> Option<Rc<RefCell<Option<Type>>>> {
        // Check current scope first
        if let Some(var_type) = self.variables.get(name) {
            return Some(var_type.clone());
        }

        // Recursively check parent scopes
        if let Some(parent) = &self.parent {
            return parent.resolve_name(name);
        }

        None
    }
}
```

**Scope Management**
```rust
impl Context {
    pub fn enter_scope(&mut self) {
        let new_scope = Scope::new_with_parent(self.scope.clone());
        self.scope = new_scope;
    }

    pub fn exit_scope(&mut self) {
        if let Some(parent) = self.scope.parent.take() {
            self.scope = *parent;
        }
    }
}
```

### Context Threading

Context is threaded through the type checking process:

```rust
trait TypeCheckable {
    type Typed;

    fn check(self, ctx: &mut Context) -> TypeResult<Self::Typed>;
}

// Example implementation for expressions
impl TypeCheckable for Expression<()> {
    type Typed = Expression<TypeInformation>;

    fn check(self, ctx: &mut Context) -> TypeResult<Self::Typed> {
        match self {
            Expression::Id(id) => {
                let typed_id = id.check(ctx)?;
                Ok(Expression::Id(typed_id))
            }
            Expression::Binary(binary) => {
                let typed_binary = binary.check(ctx)?;
                Ok(Expression::Binary(typed_binary))
            }
            // ... other expression types
        }
    }
}
```

## Error Handling and Recovery

### Error Types

The type checker provides comprehensive error reporting:

```rust
#[derive(Debug, Clone)]
pub enum TypeCheckError {
    TypeMismatch {
        expected: Type,
        found: Type,
        position: Span,
    },
    UndefinedVariable {
        name: String,
        position: Span,
    },
    UndefinedFunction {
        name: String,
        position: Span,
    },
    InvalidMainSignature {
        found_signature: Type,
        position: Span,
    },
    MissingMainFunction,
    // ... other error types
}
```

### Error Recovery Strategy

The type checker implements robust error recovery:

#### Continue on Error
- Don't stop type checking at first error
- Collect multiple errors for better developer experience
- Provide partial type information even with errors

#### Error Propagation
- Limit error cascading effects
- Use `Unknown` type for failed type inference
- Maintain type checking progress despite local failures

#### Context Preservation
- Maintain scope information through errors
- Preserve position information for accurate reporting
- Enable IDE features even with type errors

### Error Display

Type errors include rich context and suggestions:

```text
Error: Type mismatch at line 5, column 10
Expected: Integer
Found:    String

  |
5 | let x: Integer = "hello";
  |                  ^^^^^^^ Expected Integer, found String
  |

Suggestion: Remove the type annotation or change the value to match the expected type.
```

## Implementation Details

### Two-Pass Type Checking

The type checker uses a two-pass approach for forward references:

#### Pass 1: Shallow Check
```rust
impl TypeChecker {
    fn shallow_check(&mut self) -> TypeResult<()> {
        // Register struct types first
        for stmt in &self.statements {
            if let TopLevelStatement::StructDeclaration(struct_decl) = stmt {
                struct_decl.register_type(&mut self.context)?;
            }
        }

        // Register function signatures
        for stmt in &self.statements {
            if let TopLevelStatement::Function(func) = stmt {
                func.register_signature(&mut self.context)?;
            }
        }

        Ok(())
    }
}
```

#### Pass 2: Deep Check
```rust
impl TypeChecker {
    fn deep_check(&mut self) -> TypeResult<Vec<TopLevelStatement<TypeInformation>>> {
        let mut checked = vec![];

        for stmt in &self.statements {
            let typed_stmt = stmt.clone().check(&mut self.context)?;
            checked.push(typed_stmt);
        }

        Ok(checked)
    }
}
```

### Main Function Validation

Special validation for the program entry point:

```rust
impl TypeChecker {
    fn check_main_function(&mut self) -> Result<(), TypeCheckError> {
        let main = self.context.scope.resolve_name("main")
            .ok_or(TypeCheckError::MissingMainFunction)?;

        let main_type = main.borrow().clone()
            .ok_or(TypeCheckError::MissingMainFunction)?;

        match main_type {
            Type::Function { params, return_value } => {
                // Main function must have no parameters
                if !params.is_empty() {
                    return Err(TypeCheckError::InvalidMainSignature { /* ... */ });
                }

                // Return type must be void or integer
                match *return_value {
                    Type::Void | Type::Integer => Ok(()),
                    _ => Err(TypeCheckError::InvalidMainSignature { /* ... */ }),
                }
            }
            _ => Err(TypeCheckError::MissingMainFunction),
        }
    }
}
```

### Type Validation

Final validation ensures all types are resolved:

```rust
impl TypeInformation {
    fn validate(self, position: &Span) -> Result<ValidatedTypeInformation, TypeValidationError> {
        let TypeInformation { type_id, context } = self;

        if let Some(concrete_type) = type_id.borrow().clone() {
            Ok(ValidatedTypeInformation {
                type_id: concrete_type,
                context,
            })
        } else {
            Err(TypeValidationError::UnresolvedType(position.clone()))
        }
    }
}
```

## Integration Points

### Parser Integration

The type checker consumes untyped ASTs from the parser:

```rust
pub fn type_check_program(
    statements: Vec<TopLevelStatement<()>>
) -> TypeResult<Vec<TopLevelStatement<ValidatedTypeInformation>>> {
    // Create type checker
    let mut checker = TypeChecker::new(statements);

    // Perform type checking
    let typed_statements = checker.check()?;

    // Validate all types are resolved
    let validated_statements = TypeChecker::validate(typed_statements)?;

    Ok(validated_statements)
}
```

### Code Generation Integration

The type checker provides complete type information for code generation:

```rust
impl CodeGen for Expression<ValidatedTypeInformation> {
    fn codegen(&self, ctx: &CodegenContext) -> BasicValueEnum {
        match self {
            Expression::Id(id) => {
                // Type information available for optimization
                let var_type = &id.info.type_id;
                match var_type {
                    Type::Integer => /* generate i64 load */,
                    Type::FloatingPoint => /* generate f64 load */,
                    // ... other types
                }
            }
            // ... other expressions
        }
    }
}
```

### Language Server Integration

Type information enables advanced IDE features:

```rust
// Example: Hover information
pub fn get_type_at_position(
    ast: &[TopLevelStatement<ValidatedTypeInformation>],
    position: &Position
) -> Option<Type> {
    // Find AST node at position
    let node = find_node_at_position(ast, position)?;

    // Return type information
    match node {
        AstNode::Expression(expr) => Some(expr.info.type_id.clone()),
        AstNode::Variable(var) => Some(var.info.type_id.clone()),
        // ... other node types
    }
}
```

## Performance Considerations

### Time Complexity

- **Type Inference**: O(n × α(n)) where α is inverse Ackermann (nearly linear)
- **Scope Resolution**: O(d) where d is scope depth (typically small)
- **Constraint Solving**: O(c) where c is constraint count
- **Overall**: O(n) linear in AST size for most programs

### Memory Usage

- **Type Information**: ~32 bytes per AST node
- **Type Variables**: Shared via `Rc<RefCell<>>` for efficiency
- **Scope Stack**: Proportional to nesting depth
- **Context**: Single instance threaded through checking

### Optimization Strategies

#### Type Variable Sharing
```rust
// Share type variables between related expressions
let shared_type = Rc::new(RefCell::new(None));
expr1.info.type_id = shared_type.clone();
expr2.info.type_id = shared_type.clone();
```

#### Scope Optimization
```rust
// Efficient scope chain traversal
impl Scope {
    fn resolve_name_optimized(&self, name: &str) -> Option<Type> {
        let mut current = self;
        loop {
            if let Some(var_type) = current.variables.get(name) {
                return Some(var_type.clone());
            }
            current = current.parent.as_ref()?;
        }
    }
}
```

#### Caching
- Cache resolved types for reuse
- Memoize expensive type operations
- Reuse context instances where possible

## Developer Guide

### Adding New Types

#### 1. Define Type Variant
Add new variant to the `Type` enum:

```rust
pub enum Type {
    // ... existing types
    NewType {
        field1: Type,
        field2: String,
    },
}
```

#### 2. Implement Type Operations
Update type checking operations:

```rust
impl Type {
    pub fn does_eq(&self, other: &Self) -> bool {
        match (self, other) {
            // ... existing cases
            (Self::NewType { field1: l1, field2: l2 },
             Self::NewType { field1: r1, field2: r2 }) => {
                l1 == r1 && l2 == r2
            }
            // ... other cases
        }
    }
}
```

#### 3. Add Type Checking Logic
Implement type checking for expressions using the new type:

```rust
impl TypeCheckable for NewExpression<()> {
    type Typed = NewExpression<TypeInformation>;

    fn check(self, ctx: &mut Context) -> TypeResult<Self::Typed> {
        // Implement type checking logic
        let field_type = self.field.check(ctx)?;

        // Create type information
        let type_info = TypeInformation {
            type_id: Rc::new(RefCell::new(Some(Type::NewType {
                field1: field_type.info.type_id.borrow().clone().unwrap(),
                field2: self.metadata.clone(),
            }))),
            context: ctx.clone(),
        };

        Ok(NewExpression {
            field: field_type,
            info: type_info,
            // ... other fields
        })
    }
}
```

### Debugging Type Checking

#### Common Issues

**Unresolved Type Variables**
```rust
// Debug unresolved types
if let Some(Type::Unknown) = type_var.borrow().as_ref() {
    eprintln!("Warning: Unresolved type variable at {:?}", position);
}
```

**Scope Issues**
```rust
// Debug scope resolution
fn debug_scope_lookup(scope: &Scope, name: &str) {
    eprintln!("Looking up '{}' in scope:", name);
    let mut current = Some(scope);
    let mut depth = 0;

    while let Some(scope) = current {
        eprintln!("  Depth {}: {:?}", depth, scope.variables.keys());
        current = scope.parent.as_ref().map(|p| p.as_ref());
        depth += 1;
    }
}
```

**Type Constraint Debugging**
```rust
// Log type unification
fn debug_unify(t1: &Type, t2: &Type) -> bool {
    let result = t1.does_eq(t2);
    if !result {
        eprintln!("Type unification failed: {:?} ≠ {:?}", t1, t2);
    }
    result
}
```

### Testing Type Checking

#### Test Structure
```rust
#[test]
fn test_function_type_inference() {
    let source = r#"
        fn add(x: i64, y: i64): i64 {
            return x + y;
        }

        fn main(): i64 {
            return add(1, 2);
        }
    "#;

    let ast = parse_program_string(source).unwrap();
    let typed_ast = type_check_program(ast).unwrap();

    // Verify function type
    let main_func = find_function(&typed_ast, "main").unwrap();
    assert_eq!(main_func.return_type, Some(Type::Integer));

    // Verify call type
    let call_expr = find_call_expression(&main_func.body).unwrap();
    assert_eq!(call_expr.info.type_id, Type::Integer);
}
```

#### Property Testing
```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn type_checking_preserves_ast_structure(ast in any_valid_ast()) {
        let typed_ast = type_check_program(ast.clone()).unwrap();

        // Verify structure preservation
        assert_eq!(count_nodes(&ast), count_nodes(&typed_ast));
        assert_eq!(get_function_names(&ast), get_function_names(&typed_ast));
    }
}
```

This type checking implementation provides strong safety guarantees while maintaining excellent performance and developer experience. The bidirectional inference algorithm ensures that type annotations are minimal while still providing complete type information for optimization and error detection.
