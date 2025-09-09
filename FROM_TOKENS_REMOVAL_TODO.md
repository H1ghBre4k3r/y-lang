# FromTokens Removal - Remaining Tasks

## Status: 95% Complete ✅

The major work of removing FromTokens trait and implementations is **DONE**. Only test cleanup remains.

## What's Been Completed ✅

1. **✅ Test Infrastructure** - `test_helpers.rs` with comprehensive helper functions
2. **✅ Integration Tests** - `tests/parser_integration.rs` with good coverage  
3. **✅ Key Test Migrations** - Major expression and statement tests migrated to use new helpers
4. **✅ FromTokens Implementations Removed** - All 25+ `impl FromTokens<Token>` blocks removed across 21 files
5. **✅ Core Infrastructure Cleanup** - Removed deprecated `FromTokens` trait, `parse()` function, `combinators.rs`, `parse_state.rs`
6. **✅ Binary Updates** - Updated `yc`, `yfmt`, `yls` binaries to use new grammar-based parsing
7. **✅ Build Success** - Project compiles successfully

## Remaining Tasks 🔧

### Test Cleanup (Only Remaining Work)

The following test files still have remnants of the old token-based testing approach and need cleanup:

#### 1. Fix Test Files with Old Parsing Code
These files have tests that still try to call `Expression::parse()` or use `Token` directly:

- `crates/why_lib/src/parser/ast/expression/mod.rs` - Lines 166, 183, 172, 189, 205, etc.
- `crates/why_lib/src/parser/ast/statement/method_declaration.rs` - Lines 58, 83
- `crates/why_lib/src/parser/ast/statement/struct_declaration.rs` - Lines 88, 117, 156

**Fix**: Replace with calls to test helper functions from `test_helpers.rs`

#### 2. Fix TypeName Test Issues  
- `crates/why_lib/src/parser/ast/type_name.rs` - Lines 146, 156, 166, 185, 199, 212
- **Issue**: Using `**return_type` and `**inner` syntax incorrectly
- **Fix**: Update pattern matching to use correct dereferencing syntax

#### 3. Clean Up Unused Imports
Several test files have unused imports from the old system:
- `expression/array.rs:72` - unused `ast::Id`
- `expression/character.rs:33` - unused `super::*`
- `expression/id.rs:36` - unused `super::*` 
- `expression/lambda.rs:76` - unused `super::*`
- `expression/string.rs:32` - unused `super::*`

#### 4. Missing Test Migrations
Some files may still need their tests fully migrated to the new approach:
- `expression/mod.rs` - Large expression test suite
- `statement/mod.rs` - Statement parsing tests
- Any other files with remaining `.lex()` calls or `FromTokens::parse()` usage

## Quick Fix Strategy 🚀

1. **Search and Replace**: Find all remaining `SomeType::parse(&mut tokens)` calls and replace with appropriate test helper calls
2. **Import Cleanup**: Remove unused imports and add `use crate::parser::test_helpers::*;` where needed
3. **Pattern Matching**: Fix the `**` dereferencing issues in type_name.rs tests
4. **Validation**: Run `just test` after each fix to ensure progress

## Expected Outcome ✨

After completing these test cleanups:
- ✅ Zero `FromTokens` dependencies anywhere in codebase
- ✅ All tests passing with equivalent coverage using rust-sitter parsing
- ✅ Clean codebase with no deprecated warnings
- ✅ All CLI tools (yc, yfmt, yls) working correctly with new parser

## Time Estimate: 30-60 minutes 

The remaining work is straightforward test cleanup - no complex parser logic changes needed!