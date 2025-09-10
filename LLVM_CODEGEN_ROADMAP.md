# LLVM Code Generation Roadmap with Inkwell

This roadmap outlines the implementation plan for adding LLVM code generation to the Y-lang compiler using the Rust `inkwell` crate.

## Current State Analysis

The Y-lang compiler currently has:
- ✅ **Lexer** - Tokenizes source code
- ✅ **Parser** - Builds AST from tokens
- ✅ **Type Checker** - Type checking with context and scope management
- ✅ **Validator** - Validates typed AST
- ✅ **Formatter** - Code formatting
- ❌ **Code Generator** - Missing (this roadmap's target)

The compiler pipeline flows: `Source → Lexer → Parser → TypeChecker → Validator → ?CodeGen?`

## Implementation Phases

### Phase 1: Infrastructure Setup

**Goal**: Set up the foundation for LLVM code generation

#### Tasks:
1. **Add inkwell dependency**
   - Add `inkwell = { version = "0.4", features = ["llvm17-0"] }` to `crates/why_lib/Cargo.toml`
   - Consider feature flags for different LLVM versions

2. **Create codegen module structure**
   ```
   crates/why_lib/src/codegen/
   ├── mod.rs              # Main codegen interface and exports
   ├── context.rs          # LLVM context and module management
   ├── values.rs           # Value mapping and stack management
   ├── types.rs            # Y-lang type → LLVM type conversion
   ├── expression.rs       # Expression code generation
   ├── statement.rs        # Statement code generation
   └── error.rs            # Code generation error types
   ```

3. **Add CLI emission flags** to `VCArgs` in `src/lib.rs`:
   - `--emit-llvm` - Emit LLVM IR (.ll files)
   - `--emit-bitcode` - Emit LLVM bitcode (.bc files) 
   - `--emit-assembly` - Emit native assembly (.s files)
   - `--emit-object` - Emit object files (.o files)
   - an executable is always emitted

4. **Update main compilation pipeline**
   - Integrate codegen after validation step
   - Handle emission options

### Phase 2: Core Code Generation Framework

**Goal**: Build the foundational code generation infrastructure

#### Tasks:
5. **Implement `CodeGenContext`**
   ```rust
   pub struct CodeGenContext<'ctx> {
       context: &'ctx Context,
       module: Module<'ctx>,
       builder: Builder<'ctx>,
       functions: HashMap<String, FunctionValue<'ctx>>,
       variables: HashMap<String, PointerValue<'ctx>>,
       // ... other state
   }
   ```

6. **Implement Y-lang → LLVM type conversion**
   - Map `Type::Integer` → `IntType<64>`
   - Map `Type::FloatingPoint` → `FloatType<64>`
   - Map `Type::Boolean` → `IntType<1>`
   - Map `Type::Character` → `IntType<8>`
   - Map `Type::String` → pointer to `IntType<8>`
   - Map `Type::Function` → `FunctionType`
   - Map `Type::Struct` → `StructType`
   - Map `Type::Array` → `ArrayType`
   - Map `Type::Tuple` → `StructType`
   - Map `Type::Reference` → `PointerType`

7. **Create `CodeGenerable` trait**
   ```rust
   pub trait CodeGenerable<'ctx> {
       type Output;
       fn codegen(&self, ctx: &mut CodeGenContext<'ctx>) -> Result<Self::Output, CodeGenError>;
   }
   ```

8. **Implement value and variable management**
   - Stack allocation for local variables
   - Load/store operations for variables
   - Scope management for nested blocks

### Phase 3: Expression Code Generation

**Goal**: Generate LLVM IR for all Y-lang expressions

#### Tasks:
9. **Literal expressions**
   - `Expression::Num` → `IntValue` or `FloatValue`
   - `Expression::Character` → `IntValue<8>`
   - `Expression::AstString` → global string constant + pointer
   - Boolean literals → `IntValue<1>`

10. **Variable expressions**
    - `Expression::Id` → load from stack-allocated variable
    - Handle scope resolution and variable lookup

11. **Binary operations**
    - Arithmetic: `+`, `-`, `*`, `/`, `%`
    - Comparison: `==`, `!=`, `<`, `>`, `<=`, `>=`
    - Logical: `&&`, `||`
    - Handle type-specific operations (int vs float)

12. **Prefix operations**
    - Unary minus: `-expr`
    - Logical not: `!expr`

13. **Function calls and postfix operations**
    - Function call: `func(args...)`
    - Array access: `arr[index]`
    - Struct field access: `struct.field`
    - Method calls: `obj.method(args...)`

14. **Lambda expressions**
    - Create anonymous functions
    - Handle closure capture (if needed)
    - Function pointer generation

15. **Control flow expressions**
    - `Expression::If` → conditional branches with phi nodes
    - `Expression::Block` → sequential instruction generation

### Phase 4: Statement & Control Flow Code Generation

**Goal**: Handle statements and control flow constructs

#### Tasks:
16. **Variable statements**
    - `Statement::Initialization` → alloca + store
    - `Statement::Assignment` → load + store
    - `Statement::Constant` → global constants

17. **Function declarations**
    - `Statement::Function` → LLVM function definition
    - Parameter handling and local variable setup
    - Return value generation

18. **Control flow statements**
    - `Statement::If` → conditional branches
    - `Statement::WhileLoop` → loop with condition check
    - `Statement::Return` → return instruction
    - Basic block management for complex control flow

19. **Expression statements**
    - `Statement::Expression` → generate IR and discard result
    - `Statement::YieldingExpression` → generate IR and use result

### Phase 5: Advanced Features

**Goal**: Support advanced language features

#### Tasks:
20. **Struct support**
    - `StructDeclaration` → LLVM struct type definition
    - `StructInitialisation` → struct construction
    - Field access and modification
    - Memory layout and alignment

21. **Instance methods**
    - Method dispatch for struct instances
    - `self` parameter handling
    - Method table generation (if needed)

22. **Array operations**
    - Dynamic array creation: `&[1, 2, 3]`
    - Array indexing with bounds checking (optional)
    - Array literals and initialization

23. **Memory management**
    - Stack allocation for local variables
    - Heap allocation for dynamic structures (basic malloc/free)
    - Consider integration with Rust's allocator or custom runtime

24. **Function pointers and first-class functions**
    - Function-as-value semantics
    - Function pointer calls
    - Lambda capture and closure generation

### Phase 6: Integration & Output Generation

**Goal**: Complete integration with the compiler pipeline

#### Tasks:
25. **Pipeline integration**
    - Call codegen after validation in `compile_file()`
    - Pass validated AST to code generator
    - Handle codegen errors gracefully

26. **Output emission**
    - LLVM IR output (`.ll` files)
    - Bitcode output (`.bc` files)
    - Assembly output (`.s` files) using LLVM's target machine
    - Object file output (`.o` files)
    - Executable generation (link with system linker)

27. **Error handling and diagnostics**
    - Comprehensive error types for codegen failures
    - Source location tracking in generated IR
    - Helpful error messages for common issues

28. **Testing and validation**
    - Unit tests for each codegen component
    - Integration tests with example programs
    - Comparison with expected LLVM IR output
    - Runtime testing of generated executables

## Technical Considerations

### LLVM Version Compatibility
- Target LLVM 17.x initially (latest stable)
- Use inkwell feature flags for version selection
- Consider backward compatibility needs

### Memory Model
- Stack allocation for local variables using `alloca`
- Global constants for string literals and constants
- Heap allocation strategy for dynamic arrays and structures
- Consider garbage collection vs manual memory management

### ABI Compatibility
- Follow platform calling conventions
- Ensure struct layout matches C ABI when needed
- Handle name mangling for exported functions

### Optimization
- Start with `-O0` (no optimization) for debugging
- Add optimization passes in later phases
- Consider LLVM's optimization pipeline integration

### Error Recovery
- Graceful handling of unsupported features
- Clear error messages with source location
- Fallback mechanisms for partial compilation

## Success Criteria

1. **Basic Programs**: Simple arithmetic and function calls compile and run
2. **Control Flow**: If statements and while loops work correctly
3. **Data Structures**: Structs and arrays can be created and accessed
4. **Functions**: First-class functions and lambdas work as expected
5. **Memory Safety**: No memory leaks in generated code (within reason)
6. **Performance**: Generated code has reasonable performance characteristics

## Future Enhancements

- **Optimization**: Advanced LLVM optimization passes
- **Debugging**: DWARF debug information generation
- **FFI**: Foreign function interface for C interop
- **Concurrency**: Support for concurrent programming constructs
- **JIT Compilation**: Runtime code generation and execution
- **Cross-compilation**: Target multiple platforms and architectures

## Estimated Timeline

- **Phase 1-2**: 1-2 weeks (Infrastructure and framework)
- **Phase 3**: 2-3 weeks (Expression generation)  
- **Phase 4**: 2-3 weeks (Statement and control flow)
- **Phase 5**: 3-4 weeks (Advanced features)
- **Phase 6**: 1-2 weeks (Integration and polish)

**Total**: ~10-14 weeks for full implementation

This roadmap provides a structured approach to implementing LLVM code generation while building on the solid foundation of your existing compiler frontend.
