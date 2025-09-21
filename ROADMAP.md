# Y Lang Compiler Roadmap

This roadmap outlines the development plan for the Y Lang programming language compiler, from the current state (v0.1 with complete parser) to a production-ready language (v1.0).

## Current State Analysis (v0.1)

### ✅ What's Working
- Complete lexing and parsing pipeline using rust-sitter grammar
- Full type checking system with two-phase validation
- Basic LLVM code generation for simple programs
- Function declarations, basic expressions, binary operations
- String literals, integer literals, variable initialization
- Function calls and external declarations (like printf)
- Basic control flow (simple functions work)
- If/else expressions with proper conditional branching and LLVM codegen

### ❌ What's Missing (Many `todo!()` placeholders)
- Lambda expressions and closures (critical missing feature)
- Arrays and array indexing
- Struct declarations and initialization
- While loops
- Character literals
- Float literals
- Block expressions
- Method calls and instances
- Property access
- Assignment statements
- Constants
- Prefix operators
- Complex type handling (tuples, references)

### 🧪 Test Status
- ✅ `examples/simple.why` - Compiles and runs successfully
- ✅ `examples/if.why` - Compiles and runs successfully with conditional expressions
- ❌ `examples/main.why` - Panics on lambda expressions in type checker
- ❌ Most complex language features untested due to missing implementations

## V0.2 - Foundation Completion (Next 3-6 months)

**Goal**: Complete core language features to enable practical programming

### Priority 1: Core Expression System
- [ ] **Lambda Expressions & Closures** - Complete implementation in type checker and codegen
  - Fix panic in `examples/main.why`
  - Implement closure capture semantics
  - Add proper lifetime management
- ✅ **If/Else Expressions** - ✅ COMPLETED: Full conditional expression support with proper type checking and LLVM codegen
- [ ] **Block Expressions** - Scoped execution blocks with proper variable handling
- [ ] **Arrays** - Array literals, indexing, and memory management
  - Static arrays: `[42, 1337]`
  - Array indexing: `arr[0]`
  - Reference arrays: `&[42, 1337]`
- [ ] **Character Literals** - Basic char support and UTF-8 handling

### Priority 2: Control Flow & Statements
- [ ] **While Loops** - Complete loop implementation with proper scoping
- [ ] **Assignment Statements** - Variable mutation and complex assignments
- [ ] **Constants** - Global and local constant declarations
- [ ] **Expression Statements** - Proper handling of expression vs statement contexts

### Priority 3: Advanced Types & Structs
- [ ] **Struct Declaration & Initialization** - Complete struct system
  - Struct definitions: `struct Foo { x: i64; }`
  - Struct initialization: `Foo { x: 42 }`
  - Field access: `foo.x`
- [ ] **Method Calls & Instances** - Object-oriented features
  - Instance methods: `foo.method()`
  - Method declarations in instance blocks
- [ ] **Property Access** - Dot notation and field access
- [ ] **Tuples** - Multi-value types and destructuring

### Priority 4: Robustness & Optimization
- [ ] **Error Recovery** - Better error messages and compilation resilience
- [ ] **Memory Management** - Proper LLVM memory allocation strategies
- [ ] **Basic Optimizations** - Dead code elimination, constant folding
- [ ] **Float Support** - Complete floating-point arithmetic

### Success Criteria for V0.2
- All examples in `/examples/` directory compile and run correctly
- No `todo!()` panics in core language features
- Basic programs can be written and executed reliably

## V0.3 - Advanced Features (6-12 months)

**Goal**: Modern language features and enhanced developer experience

### Language Features
- [ ] **Pattern Matching** - Match expressions with destructuring
- [ ] **Generics/Templates** - Basic generic types and functions
- [ ] **Traits/Interfaces** - Behavioral contracts and implementation
- [ ] **Module System** - File imports, namespaces, and visibility
- [ ] **Error Handling** - Result types and exception mechanisms

### Tooling Enhancements
- [ ] **Enhanced LSP** - Auto-completion, go-to-definition, refactoring
- [ ] **Debugger Support** - DWARF debug info generation
- [ ] **Package Manager** - Dependency management and build system
- [ ] **Documentation Generator** - Automatic docs from source code

### Success Criteria for V0.3
- Self-hosting capability (parts of compiler written in Y Lang)
- Professional development experience with full LSP support
- Module system enables code organization and reuse

## V1.0 - Production Ready (1-2 years)

**Goal**: Stable, performant language suitable for real-world applications

### Advanced Language Features
- [ ] **Async/Await** - Asynchronous programming support
- [ ] **Memory Safety** - Borrow checker or GC implementation
- [ ] **FFI** - C interoperability and external library binding
- [ ] **Macros** - Compile-time code generation
- [ ] **Standard Library** - Comprehensive built-in functionality

### Ecosystem & Tooling
- [ ] **IDE Plugins** - VSCode, IntelliJ, Vim extensions
- [ ] **Build System** - Advanced build configuration and caching
- [ ] **Testing Framework** - Unit tests, integration tests, benchmarks
- [ ] **Performance Tools** - Profiler integration and optimization guides

### Infrastructure
- [ ] **Cross-Platform** - Windows, Linux, macOS, WebAssembly targets
- [ ] **CI/CD** - Automated testing and release pipeline
- [ ] **Documentation** - Complete language specification and tutorials
- [ ] **Community** - Package registry, examples, and learning resources

### Success Criteria for V1.0
- Real-world applications built in Y Lang
- Active community adoption and contributions
- Performance competitive with established languages

## Immediate Next Steps (Development Priority)

1. **Fix Lambda Type Checking** - Resolve the panic in `examples/main.why`
2. **Implement Block Expressions** - Required for most control flow
3. **Add Array Support** - Critical for practical programs
4. **Add While Loops** - Basic iteration support

## Architecture Considerations

### Code Generation Strategy
- Continue with LLVM-based backend for performance and portability
- Implement proper memory management patterns early
- Consider future WebAssembly target support

### Type System Evolution
- Maintain expression-centric design philosophy
- Plan for future generic type support
- Consider memory safety integration points

### Tooling Philosophy
- Language server first: all tools should share the same core
- Fast compilation: optimize for developer experience
- Comprehensive error messages: learning-friendly design

## Long-term Vision

Y Lang aims to be a modern, expression-oriented programming language that combines:
- **Performance**: Zero-cost abstractions and LLVM optimization
- **Safety**: Memory safety without sacrificing control
- **Expressiveness**: Functional programming concepts with practical syntax
- **Productivity**: Excellent tooling and developer experience

The roadmap balances immediate functionality needs with long-term language design goals, ensuring Y Lang evolves into a practical and powerful programming language suitable for both systems programming and application development.