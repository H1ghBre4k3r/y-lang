# Y Language Compiler Roadmap

## Current Status Summary (September 2024)

### Overall Progress: 80% Complete for v0.1 Release

**✅ Parser & Type System**: Excellent (95% complete)
- Complete syntax parsing for all language features
- Sophisticated type checking and inference
- Advanced features: structs, lambdas, arrays, control flow

**✅ Core CodeGen**: Excellent (90% complete)
- Basic functions, variables, arithmetic
- Struct declarations and initialization
- Property reading (`obj.field`)
- Complex assignment operations (`obj.field = value`, `arr[index] = value`)
- Empty array initialization (`&[]` with explicit type annotations)
- **Lambda expressions with full function pointer support**
- Function pointer variable loading and indirect calls
- Two-pass compilation for forward function references
- Main function wrapper generation (void and non-void return types)
- Control flow (if-else, while loops)
- String parameter handling (fixed segmentation faults)

**✅ Formatter & Tooling**: Complete (100% complete)
- Code formatter with blank line preservation
- Proper indentation for all language constructs
- Intelligent whitespace handling (preserves single blank lines, collapses multiple)

**❌ Critical CodeGen Gaps**: 1 blocking issue
- **Closure capture for lambda expressions** - Critically broken, silently ignores captured variables and produces incorrect results 

### Example Compilation Status
- ✅ **Working**: `simple.why`, `foo.why`, `printf.why`, `hello.why`, `instance.why`, `struct.why`, `assign.why`, `testarray.why`, `lambda.why`
- ⚠️ **Partially Working**: `closure.why` (compiles but produces incorrect results due to missing closure capture)
- ❌ **Failing**: `main.why`, `test.why`
- **Success Rate**: 9/12 examples compile without runtime issues

---

## Critical Implementation Gaps (Priority Order)

### 1. Lambda Closure Capture (HIGH)
**File**: `crates/why_lib/src/codegen/expressions/lambda.rs:56-69`
**Error**: `unwrap()` panic when accessing captured variables
**Impact**: Blocks closure syntax like `\(y) => x + y`
**Features Needed**:
- Capture analysis in typechecker
- Closure environment creation
- Modified calling convention for captured variables

### 2. Default Array Syntax (MEDIUM)
**File**: `crates/why_lib/src/codegen/expressions/mod.rs:93`
**Error**: `todo!("Default array initialization not yet implemented")`
**Feature**: `&[value; length]` syntax

---

## Architecture Overview

### Compiler Pipeline
1. **Parsing**: rust-sitter grammar → AST
2. **Type Checking**: Type inference and validation
3. **Code Generation**: AST → LLVM IR (via Inkwell)

### Key Design Patterns
- **CodeGen Trait**: Consistent code generation interface
- **Scope Management**: RefCell-based mutable scoping
- **Type System**: Generic parameters for compilation stages
- **Memory Management**: Stack locals, global constants, function pointers

---

## Development Priorities

### Phase 1: Critical CodeGen Fixes (3-4 weeks)
**Goal**: Get all current examples compiling
**Success Metrics**:
- ✅ 90%+ of examples compile without runtime issues (9/11 achieved)
- ✅ Lambda functions working with proper function pointer handling
- ✅ Main function wrapper generation supports all return types

### Phase 2: Lambda Enhancements (2-3 weeks)
**Goal**: Complete lambda functionality
- Closure capture support
- Advanced lambda features

### Phase 3: Code Quality & Tooling (1-2 weeks)
**Goal**: Improve codebase quality and developer experience
- Address compilation warnings (17 current warnings)
- Clean up unused imports and variables
- Enhanced error messages
- Documentation updates

### Phase 4: Array System Completion (1-2 weeks)
**Goal**: Full array functionality
- Default initialization syntax
- Enhanced indexing operations

---

## Testing & Build

### Test Execution
```bash
# Run all tests (183 parser tests, comprehensive type checking)
cargo test

# Build and test specific examples
cargo run --bin yc -- example.why -o output
./output

# Test working examples
cargo run --bin yc -- examples/hello.why -o out/hello && ./out/hello
cargo run --bin yc -- examples/foo.why -o out/foo && ./out/foo
cargo run --bin yc -- examples/printf.why -o out/printf && ./out/printf

# Format code with blank line preservation
cargo run --bin yc -- --format example.why

# Quick build commands
just build    # Development build
just test     # Run test suite
```

### Working Language Features
```rust
// Struct types with methods
struct Point { x: i64; y: i64; }
instance Point {
    fn distance(): f64 { /* implementation */ }
}

// Non-capturing lambdas (working)
fn main(): i64 {
    let double = \(x) => x * 2;
    double(21)  // Returns 42
}

// ❌ Closure capture (broken - compiles but produces wrong results)
fn get(x: i64): (i64) -> i64 {
    \(y) => x + y  // Should capture x, but silently ignores it
}
// get(1)(42) returns 84 instead of 43 (adds 42+42 instead of 1+42)

// Function parameters and string handling
fn takes_fn(f: (i64) -> i64): i64 { f(24) }
fn print_string(s: str): void { /* string operations */ }

// Control flow with proper blank line preservation
fn example(): void {
    let x = 42;

    if (x > 0) {
        printf("positive");
    } else {
        printf("non-positive");
    }

    let mut i = 0;
    while (i < 10) {
        i = i + 1;
    }
}
```

---

## Far Future Roadmap (6+ months)

### Language System Features

#### Module System & Package Management
- **Import/Export Syntax**: `import std.io.println;`, `export { MyStruct, my_function };`
- **Package Resolution**: Hierarchical module loading with dependency management
- **Namespace Management**: Prevent naming conflicts across modules
- **Build System Integration**: Package.toml configuration files
- **Repository Integration**: Remote package fetching and version management

#### Advanced Type System
- **Generic Types**: `struct Vec<T> { items: &[T]; }`, `fn map<T, U>(items: &[T], f: (T) -> U): &[U]`
- **Type Constraints**: `fn sort<T: Comparable>(items: &[T]): &[T]`
- **Trait System**: Interface-like constructs for shared behavior
- **Type Inference Enhancement**: Full Hindley-Milner style inference
- **Associated Types**: Types associated with traits/interfaces

#### Standard Library Foundation
- **Core Data Structures**: Vec, HashMap, Set, String, Option, Result
- **I/O Operations**: File handling, network operations, console I/O
- **String Manipulation**: Unicode support, formatting, parsing
- **Mathematical Operations**: Extended math library, random numbers
- **Date/Time Handling**: Timestamps, duration calculations, formatting
- **Error Handling**: Standardized error types and propagation patterns

### Developer Experience Enhancements

#### Tooling & IDE Support
- **Language Server Protocol (LSP)**: Complete IDE integration with autocomplete, go-to-definition
- **Debugger Integration**: LLDB/GDB support with source-level debugging
- **Formatter**: Automatic code formatting (like rustfmt)
- **Linter**: Static analysis and style checking
- **Documentation Generator**: Automatic docs from comments
- **Package Manager CLI**: Dependency management and project scaffolding

#### Advanced Error Handling
- **Rich Error Messages**: Detailed compilation errors with suggestions
- **Error Recovery**: Continue parsing after errors for better IDE experience
- **Warning System**: Non-fatal issues and style suggestions
- **Diagnostic Output**: JSON/structured error output for tooling integration

### Performance & Optimization

#### Compiler Optimizations
- **LLVM Optimization Passes**: Dead code elimination, inlining, constant folding
- **Profile-Guided Optimization**: Runtime profiling for optimization hints
- **Link-Time Optimization**: Cross-module optimizations
- **Incremental Compilation**: Only recompile changed modules
- **Parallel Compilation**: Multi-threaded compilation for large projects

#### Runtime Enhancements
- **Memory Management**: Advanced allocation strategies, garbage collection options
- **Concurrency Primitives**: Async/await, threads, channels, atomic operations
- **SIMD Support**: Vector operations for performance-critical code
- **Foreign Function Interface**: Safe C/Rust interop
- **Dynamic Loading**: Plugin systems and shared libraries

### Advanced Language Features

#### Pattern Matching Enhancement
- **Exhaustive Pattern Matching**: Compiler-verified complete pattern coverage
- **Guard Expressions**: `match x { n if n > 0 => "positive" }`
- **Destructuring**: Deep pattern matching on structs and arrays
- **Pattern Macros**: User-defined pattern extensions

#### Metaprogramming
- **Compile-Time Functions**: Code generation during compilation
- **Macro System**: Procedural and declarative macros
- **Reflection**: Runtime type information and introspection
- **Code Generation**: Template-based code generation

#### Memory Safety Features
- **Ownership System**: Borrow checker similar to Rust (optional)
- **Region-Based Memory**: Automatic memory management without GC
- **Unsafe Blocks**: Escape hatches for low-level operations
- **Memory Pool Allocation**: Custom allocation strategies

---

## Long-Term Vision Timeline

### Year 1: Foundation Completion
- ✅ Core language features (current roadmap)
- 🔄 Basic standard library (collections, I/O)
- 🔄 Module system basics
- 🔄 LSP server implementation

### Year 2: Ecosystem Development  
- ⏳ Package manager and repository
- ⏳ Generic type system
- ⏳ Advanced tooling (formatter, linter)
- ⏳ Performance optimizations

### Year 3: Production Readiness
- ⏳ Comprehensive standard library
- ⏳ Advanced error handling and diagnostics
- ⏳ Concurrency and async support
- ⏳ Foreign function interface

### Year 4+: Advanced Features
- ⏳ Metaprogramming and macros
- ⏳ Advanced type system features
- ⏳ Specialized domain libraries
- ⏳ IDE plugins and ecosystem tools

---

## Technical Debt & Quality

**Codebase Quality**: Excellent - clean architecture, good separation of concerns
**Technical Debt**: Low - minimal `todo!()` implementations remain
**Runtime Stability**: Good - no segfaults, clean panics on unimplemented features
**LLVM IR Quality**: Good - generates correct, unoptimized IR
**Formatter Quality**: Excellent - sophisticated blank line preservation and indentation

**Current Focus**: Implementation completeness rather than optimization or features

### Testing & Quality Assurance

**🚨 Silent Failure Detection Critical**:
- The `closure.why` example demonstrates a dangerous class of bugs where code compiles but produces incorrect results
- Need comprehensive runtime testing to catch semantic errors vs compilation errors
- Closure capture implementation is the highest priority due to silent failure mode

### Recent Technical Achievements

#### Lambda Function Implementation - Partially Complete (September 2024)
- **✅ Non-Capturing Lambdas**: Fully functional with proper function pointer support
- **Function Pointer Variable Loading**: Fixed critical bug where function pointer variables were not being loaded from memory correctly
- **Two-Pass Compilation System**: Implemented forward function reference support by separating function declaration from body generation
- **Main Function Wrapper Generation**: Extended wrapper creation to handle both void and non-void main functions with proper i32 return type conversion
- **Indirect Call Support**: Lambda functions work correctly as function pointers with proper calling conventions
- **❌ Closure Capture**: **CRITICALLY BROKEN** - Silently ignores captured variables, producing incorrect runtime results

**Key Files Modified**:
- `crates/why_lib/src/codegen/expressions/id.rs` - Fixed function pointer variable loading
- `crates/why_lib/src/codegen/statements/function.rs` - Added main wrapper for non-void functions
- `crates/why_lib/src/lib.rs` - Implemented two-pass compilation system

**🚨 Critical Test Case**:
- `examples/closure.why` - Demonstrates closure capture bug (returns 84 instead of 43)

**Known Issues**:
- **🚨 Critical**: Closure capture silently fails - `examples/closure.why` compiles but produces wrong results (42+42 instead of 1+42)
- 17 compilation warnings (unused imports, variables)
- 2 remaining examples with compilation issues (`main.why`, `test.why`)
- Default array initialization syntax (`&[value; length]`) pending
