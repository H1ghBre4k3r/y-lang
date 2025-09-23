# Y Lang Closure Implementation Roadmap

This document provides a comprehensive roadmap for implementing lambda expressions and closures in Y Lang, from the current state (panic on lambda expressions) to full closure support with capture semantics.

## Current State Analysis

### ✅ What's Working
- Complete lambda type checking system in `crates/why_lib/src/typechecker/typed_ast/expression/lambda.rs`
- Lambda parameter type inference and validation
- Function type representation in type system
- Lambda AST parsing and grammar support
- Type checking context management for lambda scopes

### ❌ What's Broken
- **CRITICAL**: `examples/main.why` panics on lambda expressions due to missing codegen
- Lambda codegen has `todo!()` in `crates/why_lib/src/codegen/expressions/mod.rs:28`
- No LLVM function generation for lambdas
- Missing closure capture implementation
- No runtime support for function pointers

### 🧪 Test Cases
- ❌ `examples/main.why` - Panics on `let x: (i64) -> i64 = \(x) => x;`
- ❌ Lambda assignment and function calls
- ❌ Closure capture scenarios
- ❌ Higher-order function usage

## Phase 1: Basic Lambda Support (Immediate Priority)

**Goal**: Fix the panic in `examples/main.why` and enable basic lambda compilation

### 1.1 Minimal Lambda Codegen
- [ ] Create `crates/why_lib/src/codegen/expressions/lambda.rs`
- [ ] Implement basic LLVM function generation for lambdas
- [ ] Handle simple lambda expressions without captures
- [ ] Generate function pointers for lambda values
- [ ] Test with identity lambda: `\(x) => x`

**Implementation Notes:**
- Start with lambdas that don't capture external variables
- Use LLVM function types and function pointers
- Focus on making `examples/main.why` compile first

### 1.2 Function Type LLVM Integration
- [ ] Map Y Lang function types to LLVM function types
- [ ] Implement function pointer creation and storage
- [ ] Handle function pointer calling conventions
- [ ] Add function type size calculations for memory layout

### 1.3 Basic Lambda Testing
- [ ] Create `examples/lambda_basic.why` with simple lambdas
- [ ] Test lambda assignment: `let f = \(x) => x + 1;`
- [ ] Test lambda calls: `f(42)`
- [ ] Test lambda parameters in functions: `fn takes_func(f: (i64) -> i64)`

**Success Criteria:**
- `examples/main.why` compiles without panicking
- Basic lambda expressions work in simple cases
- Function pointers can be stored and called

## Phase 2: Function Calls and Higher-Order Functions

**Goal**: Enable lambdas to be passed around and called as first-class values

### 2.1 Lambda Calling Convention
- [ ] Implement lambda invocation in postfix expressions
- [ ] Handle function pointer dereferencing in LLVM
- [ ] Support lambda calls: `lambda_var(args)`
- [ ] Add proper calling convention for lambda functions

### 2.2 Higher-Order Function Support
- [ ] Enable passing lambdas to functions
- [ ] Support returning lambdas from functions
- [ ] Test function composition scenarios
- [ ] Implement function type checking in calls

### 2.3 Advanced Lambda Testing
- [ ] Create `examples/higher_order.why`
- [ ] Test: `fn map(arr: &[i64], f: (i64) -> i64) -> &[i64]`
- [ ] Test lambda returns: `fn make_adder(x: i64) -> (i64) -> i64`
- [ ] Test nested lambda calls

**Success Criteria:**
- Lambdas can be passed as arguments to functions
- Functions can return lambdas
- Complex function compositions work

## Phase 3: Closure Capture Implementation

**Goal**: Implement proper closure semantics with variable capture

### 3.1 Capture Analysis
- [ ] Implement closure capture detection in type checker
- [ ] Identify free variables in lambda expressions
- [ ] Determine capture-by-value vs capture-by-reference
- [ ] Add capture validation and error reporting

### 3.2 Closure Data Structure
- [ ] Design closure representation in LLVM
- [ ] Implement closure environment/context structure
- [ ] Handle captured variable storage and access
- [ ] Add memory management for closure environments

### 3.3 Closure Codegen
- [ ] Generate closure allocation code
- [ ] Implement captured variable copying/referencing
- [ ] Create closure invocation trampolines
- [ ] Handle closure lifetime management

### 3.4 Closure Testing
- [ ] Create `examples/closure_basic.why`
- [ ] Test simple capture: `let x = 42; let f = \() => x;`
- [ ] Test mutable capture scenarios
- [ ] Test nested closures and complex captures

**Success Criteria:**
- Closures can capture variables from outer scopes
- Captured variables maintain correct values
- Memory safety is preserved for closure lifetimes

## Phase 4: Advanced Closure Features

**Goal**: Complete closure implementation with full language integration

### 4.1 Mutable Captures
- [ ] Implement capture-by-reference for mutable variables
- [ ] Handle mutable closure invocation
- [ ] Add capture mutability checking
- [ ] Test mutable capture scenarios

### 4.2 Complex Closure Scenarios
- [ ] Support closures in struct fields
- [ ] Enable closure arrays and collections
- [ ] Implement recursive closures
- [ ] Handle closure parameter inference

### 4.3 Optimization
- [ ] Implement closure inlining optimizations
- [ ] Add escape analysis for stack allocation
- [ ] Optimize simple lambdas to function pointers
- [ ] Memory pool allocation for closure environments

### 4.4 Integration Testing
- [ ] Test closures with all language features
- [ ] Validate closure interaction with structs
- [ ] Test closure serialization/deserialization
- [ ] Performance benchmarking

**Success Criteria:**
- All closure use cases work correctly
- Performance is competitive with other languages
- Memory usage is optimized

## Implementation Architecture

### Core Components

#### 1. Type System Integration
**Location**: `crates/why_lib/src/typechecker/`
- ✅ Lambda type checking (`expression/lambda.rs`)
- [ ] Closure capture analysis
- [ ] Environment type generation
- [ ] Capture validation

#### 2. Code Generation
**Location**: `crates/why_lib/src/codegen/`
- ❌ Lambda codegen (`expressions/lambda.rs` - needs creation)
- [ ] Closure environment codegen
- [ ] Function pointer management
- [ ] Memory allocation strategies

#### 3. Runtime Support
**Location**: `crates/why_lib/src/runtime/` (may need creation)
- [ ] Closure allocation functions
- [ ] Function pointer calling conventions
- [ ] Memory management utilities
- [ ] Debug support for closures

### Memory Management Strategy

#### Stack Allocation (Phase 1-2)
- Simple lambdas without captures
- Function pointers only
- No dynamic allocation needed

#### Heap Allocation (Phase 3-4)
- Closures with captured environments
- Reference counting or GC integration
- Escape analysis optimization

### LLVM Integration Details

#### Function Types
```rust
// Y Lang: (i64, i64) -> i64
// LLVM: i64 (i64, i64)*
```

#### Closure Representation
```rust
// Y Lang closure with captures
struct Closure {
    function_ptr: fn_ptr,
    environment: *mut CaptureEnv,
}
```

## Test Strategy

### Example Files Progression

1. **`examples/lambda_minimal.why`** - Basic lambda syntax
2. **`examples/lambda_calls.why`** - Function calls with lambdas
3. **`examples/closure_simple.why`** - Basic variable capture
4. **`examples/closure_complex.why`** - Advanced capture scenarios
5. **`examples/closure_performance.why`** - Optimization validation

### Unit Test Structure

```rust
// crates/why_lib/src/codegen/expressions/lambda.rs
#[cfg(test)]
mod tests {
    // Test lambda codegen without captures
    // Test function pointer generation
    // Test calling convention
}
```

### Integration Tests

```rust
// tests/integration/closure_tests.rs
// Full compilation and execution tests
// Memory safety validation
// Performance benchmarks
```

## Potential Pitfalls & Solutions

### 1. Memory Management Complexity
**Problem**: Closure environments need careful lifetime management
**Solution**: Start with simple reference counting, optimize later

### 2. LLVM Function Pointer Complexity
**Problem**: LLVM function pointers can be tricky with captures
**Solution**: Use trampolines for closures, direct calls for simple lambdas

### 3. Type Inference Interactions
**Problem**: Lambda parameter types might conflict with inference
**Solution**: Maintain existing type checking approach, add capture typing

### 4. Capture Semantics Decisions
**Problem**: Move vs copy vs reference semantics for captures
**Solution**: Start with copy semantics, add move/reference later

### 5. Performance Overhead
**Problem**: Closures might introduce significant overhead
**Solution**: Implement optimization passes and escape analysis

## Success Metrics

### Functionality Metrics
- [ ] All `examples/*.why` files compile successfully
- [ ] Zero `todo!()` panics in lambda-related code
- [ ] 100% test coverage for closure features
- [ ] Memory safety validation passes

### Performance Metrics
- [ ] Simple lambdas have zero overhead vs function pointers
- [ ] Closure allocation overhead < 10% vs equivalent manual code
- [ ] Function call overhead comparable to C function pointers

### Developer Experience Metrics
- [ ] Clear error messages for closure-related issues
- [ ] Comprehensive documentation with examples
- [ ] IDE support for lambda type inference
- [ ] Debugging support for closure inspection

## Timeline Estimate

- **Phase 1**: 1-2 weeks (Critical path to fix main.why)
- **Phase 2**: 2-3 weeks (Higher-order functions)
- **Phase 3**: 3-4 weeks (Closure capture implementation)
- **Phase 4**: 2-3 weeks (Advanced features and optimization)

**Total**: 8-12 weeks for complete closure implementation

## Dependencies & Prerequisites

### Existing Systems
- ✅ Type checking system
- ✅ LLVM codegen infrastructure
- ✅ Function declaration system
- ❌ Block expressions (may be needed for closure bodies)
- ❌ Assignment statements (needed for mutable captures)

### External Dependencies
- LLVM function type support
- Memory allocation runtime
- Debug info generation (for debugging support)

---

## Quick Start Checklist

To begin implementation immediately:

1. [ ] Create `crates/why_lib/src/codegen/expressions/lambda.rs`
2. [ ] Replace `todo!()` in `expressions/mod.rs:28` with lambda codegen call
3. [ ] Implement minimal LLVM function generation
4. [ ] Test with simple identity lambda
5. [ ] Verify `examples/main.why` compiles

This roadmap provides a clear path from the current broken state to full closure support, with incremental milestones and comprehensive testing strategy.