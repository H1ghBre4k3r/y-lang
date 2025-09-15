# Closure Implementation Roadmap

## Current Status
The closure implementation has basic infrastructure working (environment creation, closure structs, LLVM code generation) but has significant gaps in the type system integration. Currently, only one hardcoded example (`examples/closure.why`) works due to a targeted hack rather than a robust general solution.

## Critical Issues Identified

1. **Hardcoded function detection**: Only functions named "get" with specific signatures work
2. **Hardcoded capture information**: Captures are hardcoded as `[("x", Integer)]` instead of using actual analysis
3. **Timing issue in two-pass compilation**: Function calls processed before complete type information is available
4. **Disconnect from free variable analysis**: Existing sophisticated analysis in `lambda.rs` is not used
5. **Limited pattern support**: Won't work for multiple captures, different types, nested closures, etc.

## Roadmap Implementation Plan

### Phase 1: Foundation & Comprehensive Testing (1-2 days)
**Goal**: Establish what patterns should work and create validation framework

#### 1.1 Create comprehensive test suite covering:
- Multiple function names (not just "get")
  ```why
  fn makeAdder(x: i64): (i64) -> i64 { \(y) => x + y }
  fn makeMultiplier(factor: i64): (i64) -> i64 { \(value) => factor * value }
  ```
- Different capture patterns:
  - Single capture: `\(y) => x + y`
  - Multiple captures: `\(y) => x + z + y`
  - Zero captures: `\() => 42`
  - Different types: `\(s: String) => x.toString() + s`
- Nested closures: `\(x) => \(y) => x + y`
- Complex expressions in captures
- Error cases and edge conditions

#### 1.2 Document current limitations systematically
- Run tests against current implementation
- Catalog exactly what fails and why
- Create baseline for measuring progress

### Phase 2: Architecture Analysis & Design (1-2 days)
**Goal**: Understand and fix the fundamental timing issue

#### 2.1 Deep analysis of two-pass compilation timing
- Map exactly when closure type information gets lost
- Trace the flow: lambda → function → call site
- Identify why function calls can't see complete type information
- Analyze `shallow_check()` vs full type checking phases

#### 2.2 Design proper type propagation architecture
- Determine if we need a third pass, or better inter-pass communication
- Plan how closure type information should flow through the system
- Design integration points with existing free variable analysis
- Consider performance implications

### Phase 3: Fix Core Type Propagation (2-3 days)
**Goal**: Make closure types flow properly through the system

#### 3.1 Implement proper closure type detection at call sites
- Replace hardcoded function name checking with type-based detection
- Ensure function calls can access complete closure type information
- Fix the timing so all function types are available when needed
- Remove the hack in `postfix.rs:133-142`

#### 3.2 Enhance scope management for closure types
- Ensure closure capture information persists through compilation phases
- Fix any scope resolution issues with closure types
- Improve `update_variable` and scope propagation

### Phase 4: Integration with Free Variable Analysis (1-2 days)
**Goal**: Connect existing analysis infrastructure with type system

#### 4.1 Replace hardcoded captures with actual analysis
- Use existing `find_free_variables` function from `lambda.rs`
- Integrate capture analysis results into closure type creation
- Make this work for any function, not just "get"
- Ensure type information flows from analysis to code generation

#### 4.2 Generalize closure pattern detection
- Remove all function-name specific hardcoding
- Base detection purely on type analysis and free variable results
- Support all closure patterns identified in Phase 1

### Phase 5: Validation & Edge Cases (1 day)
**Goal**: Ensure robust production-ready implementation

#### 5.1 Run comprehensive test suite
- Validate all patterns now work correctly
- Verify LLVM IR generation is correct for all cases
- Check performance and memory usage
- Compare generated code quality

#### 5.2 Handle error cases gracefully
- Invalid capture scenarios
- Type mismatches in closures
- Circular dependencies
- Clear error messages for debugging

## Key Files to Modify

### Core Type System Files:
- `crates/why_lib/src/typechecker/typed_ast/expression/postfix.rs` - Remove hardcoded hack, implement proper detection
- `crates/why_lib/src/typechecker/typed_ast/expression/function.rs` - Enhance closure type propagation
- `crates/why_lib/src/typechecker/typed_ast/expression/lambda.rs` - Integrate with type system
- `crates/why_lib/src/typechecker/scope.rs` - Improve closure type management
- `crates/why_lib/src/typechecker/mod.rs` - Fix timing issues in two-pass compilation

### Test Files:
- `examples/closure_*.why` - New comprehensive test cases
- Unit tests in relevant modules

## Estimated Timeline: 5-8 days total

## Key Dependencies:
- Phase 1 must complete before others (establishes success criteria)
- Phase 2 analysis is prerequisite for Phase 3 implementation
- Phase 3 type propagation must work before Phase 4 integration
- Phase 5 validates everything works together

## Success Criteria:
- All test patterns compile and generate correct LLVM IR
- Zero hardcoded function names or capture information
- Closure calling convention used correctly in all cases
- Performance comparable to direct function calls where appropriate
- Clean, maintainable code that integrates well with existing type system

## Current Working Example:
```why
fn main(): i64 {
    let add_one = get(1);
    add_one(42)
}

fn get(x: i64): (i64) -> i64 {
    \(y) => x + y
}
```

## Target: All Patterns Should Work:
```why
fn main(): i64 {
    let add_one = makeAdder(1);
    let multiply_by_3 = makeMultiplier(3);
    let complex = complexCapture(10, 20);

    add_one(42) + multiply_by_3(5) + complex()
}

fn makeAdder(x: i64): (i64) -> i64 { \(y) => x + y }
fn makeMultiplier(factor: i64): (i64) -> i64 { \(value) => factor * value }
fn complexCapture(a: i64, b: i64): () -> i64 { \() => a * b + 100 }
```