# Complete FromTokens Removal & Test Migration Plan

## Overview

This document outlines the comprehensive plan to transition from the legacy `FromTokens` trait-based parser to the new rust-sitter based approach, while maintaining full test coverage and code quality.

## Current State Analysis

- **21 files** contain `FromTokens` implementations across AST modules
- **20 files** contain unit tests that depend on `FromTokens`
- Core infrastructure includes `FromTokens` trait, combinators, and `ParseState` utilities
- All `FromGrammar` implementations are complete and functional
- rust-sitter parsing is working via `grammar::parse` and `parse_program`

---

## Phase 1: Create Test Infrastructure (Foundation)
**Objective**: Build helper utilities for clean test migration

### 1.1 Create Test Helper Module (`src/parser/test_helpers.rs`)
```rust
// Helper functions to simplify test migration:
pub fn parse_expression(code: &str) -> Result<Expression, ParseError>
pub fn parse_statement(code: &str) -> Result<Statement, ParseError>  
pub fn parse_type_name(code: &str) -> Result<TypeName, ParseError>
pub fn parse_function(code: &str) -> Result<Function, ParseError>
pub fn parse_program_single<T>(code: &str) -> Result<T, ParseError>
```

**Strategy**: Wrap minimal contexts around code snippets to enable component testing:
- Expressions: wrap in `fn main(): void { EXPR; }`
- Statements: wrap in `fn main(): void { STMT }`  
- TypeNames: wrap in `declare x: TYPE;`
- Functions: parse directly as top-level

### 1.2 Add Integration Test Module (`tests/parser_integration.rs`)
- End-to-end parsing tests using `grammar::parse` directly
- Verify full pipeline works correctly
- Test complex, realistic code samples

---

## Phase 2: Migrate All Unit Tests (Critical Path)
**Objective**: Update 20+ test files to use rust-sitter approach while maintaining identical test coverage

### 2.1 Test Migration Pattern
**For each test file** (21 total):
1. Replace `Lexer::new(code).lex()?.into()` with helper functions
2. Replace `SomeType::parse(&mut tokens)` with helper + pattern matching
3. Update imports to remove `FromTokens`, `ParseState`, `ParseError`
4. Keep same test coverage and assertion logic
5. Add new edge cases that rust-sitter might handle differently

### 2.2 Migration Examples

**Before (Token-based)**:
```rust
#[test]
fn test_parse() {
    let mut tokens = Lexer::new("42").lex()?.into();
    let result = Num::parse(&mut tokens)?;
    assert_eq!(result, AstNode::Num(Num::Integer(42, (), Span::default())));
}
```

**After (Helper-based)**:
```rust
#[test]
fn test_parse() {
    let result = parse_expression("42")?;
    if let Expression::Num(Num::Integer(42, (), _)) = result {
        // Test passes
    } else {
        panic!("Expected integer 42");
    }
}
```

### 2.3 Files Requiring Test Migration
1. `/parser/ast/expression/num.rs` - number parsing tests
2. `/parser/ast/expression/id.rs` - identifier tests  
3. `/parser/ast/expression/function.rs` - function definition tests
4. `/parser/ast/expression/if_expression.rs` - conditional tests
5. `/parser/ast/expression/struct_initialisation.rs` - struct literal tests
6. `/parser/ast/statement/instance.rs` - instance/implementation tests
7. `/parser/ast/statement/assignment.rs` - assignment statement tests
8. `/parser/ast/statement/initialisation.rs` - variable declaration tests
9. `/parser/ast/type_name.rs` - type annotation tests
10. ... (11 additional files with test suites)

---

## Phase 3: Remove FromTokens Implementations  
**Objective**: Clean up deprecated parsing code

### 3.1 Remove FromTokens Implementations (21 files)
- **Keep**: `FromGrammar` implementations (these stay!)
- **Remove**: All `impl FromTokens<Token> for T` blocks
- **Remove**: Unused imports (`FromTokens`, `ParseError`, `ParseState`)
- **Review**: `From<T> for AstNode` implementations (keep if used elsewhere)

### 3.2 Update Combinator System
**Analysis needed**: Determine if `combinators.rs` is used beyond FromTokens
- **Option A**: Complete removal if only used by FromTokens
- **Option B**: Refactor for any remaining use cases (formatters, etc.)
- **Decision point**: Check for dependencies in typechecker, formatter modules

---

## Phase 4: Remove Core Infrastructure
**Objective**: Remove deprecated traits and utilities

### 4.1 Remove FromTokens Trait
- Remove `pub trait FromTokens<T>` from `parser/mod.rs`
- Remove associated trait imports and exports

### 4.2 Clean Up ParseState  
- Remove `ParseState` utilities that are only used by FromTokens
- Keep utilities that might be used elsewhere (error handling, etc.)
- Update or remove `parse_state.rs` based on remaining usage

### 4.3 Update Module Exports
- Clean up `pub use` statements in `parser/mod.rs`
- Remove deprecated function exports
- Keep only rust-sitter based exports

---

## Phase 5: Final Cleanup & Validation
**Objective**: Ensure clean, working codebase

### 5.1 Code Quality
- **Remove deprecated warnings** - they should all be gone
- **Dead code elimination** - remove any unused utilities
- **Import cleanup** - remove unused dependencies

### 5.2 Testing & Validation  
- **Run full test suite** - ensure 100% pass rate
- **Performance testing** - ensure no regressions vs old parser
- **Integration testing** - verify CLI tools still work correctly

### 5.3 Documentation Updates
- Update any parser documentation to reflect rust-sitter approach
- Update README if it mentions parsing internals
- Add comments explaining the new architecture

---

## Risk Mitigation Strategy

### Testing-First Approach
**Migrate tests BEFORE removing implementations** to ensure we maintain coverage and can validate the transition works correctly.

### Incremental & Reversible  
Each phase is independently testable and reversible if issues arise.

### Validation Points
Full test suite must pass after each phase before proceeding to the next.

### Complexity Management
Helper functions isolate the complexity of rust-sitter integration from the test logic.

---

## Expected Outcomes

✅ **Clean Codebase**: Only rust-sitter based parsing remains  
✅ **Maintained Coverage**: All existing tests converted and passing  
✅ **Better Performance**: Single parse pass instead of multiple token-based passes  
✅ **Future Proof**: Foundation for grammar evolution without code changes  
✅ **Reduced Complexity**: Elimination of dual parser maintenance burden

---

## Success Criteria

1. **Zero compilation errors** after each phase
2. **100% test pass rate** maintained throughout
3. **No performance regressions** in parsing speed
4. **All CLI functionality preserved** (yc, yfmt, yls tools work)
5. **Clean codebase** with no deprecated warnings or dead code

---

*This plan provides a systematic approach to complete the parser modernization while maintaining stability and test coverage.*