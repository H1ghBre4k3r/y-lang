# Y Lang LSP Development Roadmap

## Overview

This roadmap outlines the development plan for enhancing Language Server Protocol (LSP) support in Y Lang, transforming it from a basic diagnostic server into a comprehensive IDE experience.

## Current State Analysis

### Existing Features ✅
- **Basic LSP Server**: Functional server with document synchronization using tower-lsp-server
- **Real-time Diagnostics**: Syntax and type error reporting with precise source locations
- **Document Formatting**: AST-based pretty-printing that preserves semantic structure
- **File Operations**: Watching and handling .why files with proper lifecycle management
- **Error Reporting**: Detailed error messages with span information for accurate positioning

### Architecture Strengths
- **Clean Pipeline**: Well-defined grammar → parser → typechecker → formatter flow
- **Rich Type System**: Support for functions, lambdas, structs, methods, and type inference
- **Comprehensive AST**: All nodes include position tracking via `Span` struct
- **Two-phase Type Checking**: Shallow declaration pass followed by full semantic analysis
- **Extensible Grammar**: rust-sitter based grammar with procedural macro support

### Current Limitations
- No navigation features (go-to-definition, find references)
- Missing code intelligence (auto-completion, hover information)
- No semantic highlighting or advanced editing features
- Limited to single-file analysis
- No workspace-wide symbol resolution

## Implementation Roadmap

### Phase 1: Core Navigation Features (2-3 weeks)
**Goal**: Provide essential IDE functionality for code exploration and understanding

#### 1.1 Go to Definition (`textDocument/definition`)
**Priority**: Critical
**Estimated Time**: 1 week

**Implementation Plan**:
- Extend `TypeChecker` to collect symbol definitions during analysis
- Create `SymbolDatabase` struct to store definition locations
- Add position-to-symbol lookup functionality
- Support: functions, variables, struct types, struct fields, method definitions

**Key Files to Modify**:
- `src/bin/yls.rs`: Add definition handler
- `crates/why_lib/src/typechecker/context.rs`: Extend with symbol collection
- New: `crates/why_lib/src/lsp/symbol_index.rs`

**Requirements**:
- Reuse existing scope resolution in typechecker
- Handle cross-reference between declaration sites and usage sites
- Support both local and global symbol definitions

#### 1.2 Find All References (`textDocument/references`)
**Priority**: High
**Estimated Time**: 4-5 days

**Implementation Plan**:
- Build symbol usage index during type checking
- Track all identifier references with their positions
- Implement efficient lookup by symbol definition

**Technical Approach**:
- Extend AST visitor pattern from formatter
- Store references in concurrent data structure (DashMap)
- Handle both definition sites and usage sites

#### 1.3 Document Symbols (`textDocument/documentSymbol`)
**Priority**: High
**Estimated Time**: 3-4 days

**Implementation Plan**:
- Create AST traversal to extract document outline
- Build hierarchical symbol tree (functions contain local variables, etc.)
- Support symbol kinds: Function, Variable, Struct, Field, Constant

**Output Format**:
```rust
DocumentSymbol {
    name: "main".to_string(),
    kind: SymbolKind::FUNCTION,
    range: function_span.into(),
    selection_range: function_name_span.into(),
    children: vec![/* local variables */],
}
```

### Phase 2: Advanced Code Intelligence (3-4 weeks)
**Goal**: Provide smart editing assistance and contextual information

#### 2.1 Hover Information (`textDocument/hover`)
**Priority**: High
**Estimated Time**: 1 week

**Implementation Plan**:
- Extract type information at cursor position
- Format type signatures, function prototypes
- Display variable types, return types, struct field types
- Future: Include documentation from comments (requires grammar extension)

**Example Output**:
```
fn add(a: i64, b: i64) -> i64
  Adds two integers and returns the result
```

#### 2.2 Auto-completion (`textDocument/completion`)
**Priority**: High
**Estimated Time**: 1.5 weeks

**Implementation Plan**:
- Context-aware completion based on scope
- Complete: local variables, function names, struct fields, built-in types
- Smart filtering based on expected type context
- Snippet completions for common patterns

**Completion Contexts**:
- Function calls: suggest available functions
- Struct initialization: suggest field names
- Variable usage: suggest in-scope variables
- Type annotations: suggest available types

#### 2.3 Semantic Highlighting (`textDocument/semanticTokens`)
**Priority**: Medium
**Estimated Time**: 4-5 days

**Implementation Plan**:
- Classify tokens by semantic meaning during type checking
- Support token types: variable, function, type, parameter, property
- Include token modifiers: definition, readonly, mutable
- Provide both full and range-based highlighting

### Phase 3: Code Quality & Refactoring (2-3 weeks)
**Goal**: Advanced IDE features for code improvement and maintenance

#### 3.1 Code Actions (`textDocument/codeAction`)
**Priority**: Medium
**Estimated Time**: 1 week

**Quick Fixes**:
- Add missing type annotations where inference fails
- Import/declare missing functions
- Fix common syntax errors (missing semicolons, brackets)

**Refactoring Actions**:
- Extract function from selected code
- Rename local variables
- Add explicit type annotations

#### 3.2 Inlay Hints (`textDocument/inlayHint`)
**Priority**: Medium
**Estimated Time**: 4-5 days

**Implementation Plan**:
- Show inferred types for `let` bindings
- Display parameter names in function calls
- Show return types for lambda expressions

**Example**:
```rust
let x/* : i64 */ = 42;
foo(/* a: */ 1, /* b: */ 2);
```

### Phase 4: Advanced Features (3-4 weeks)
**Goal**: Professional IDE experience with workspace-wide capabilities

#### 4.1 Workspace Symbols (`workspace/symbol`)
**Priority**: Medium
**Estimated Time**: 1 week

**Implementation Plan**:
- Build cross-file symbol index
- Support fuzzy search across entire workspace
- Handle symbol kinds and filtering
- Efficient incremental updates

#### 4.2 Signature Help (`textDocument/signatureHelp`)
**Priority**: Medium
**Estimated Time**: 4-5 days

**Implementation Plan**:
- Parse function call context
- Show parameter information with types
- Highlight current parameter being typed
- Support overloaded functions (if added to language)

#### 4.3 Call Hierarchy (`textDocument/prepareCallHierarchy`, `callHierarchy/*`)
**Priority**: Low
**Estimated Time**: 1 week

**Implementation Plan**:
- Build function call graph during analysis
- Support both incoming and outgoing calls
- Handle method calls and lambda invocations

#### 4.4 Folding Ranges (`textDocument/foldingRange`)
**Priority**: Low
**Estimated Time**: 3-4 days

**Implementation Plan**:
- Identify foldable constructs: functions, structs, blocks, comments
- Use existing AST span information
- Support different folding kinds (comment, region, imports)

## Technical Implementation Details

### Core Infrastructure Requirements

#### 1. Symbol Database (`crates/why_lib/src/lsp/symbol_index.rs`)
```rust
pub struct SymbolIndex {
    definitions: HashMap<SymbolId, Definition>,
    references: HashMap<SymbolId, Vec<Reference>>,
    by_position: IntervalTree<Position, SymbolId>,
    by_name: HashMap<String, Vec<SymbolId>>,
}

pub struct Definition {
    pub symbol_id: SymbolId,
    pub name: String,
    pub kind: SymbolKind,
    pub position: Range,
    pub type_info: Option<Type>,
    pub parent: Option<SymbolId>,
}
```

#### 2. Position Utilities (`crates/why_lib/src/lsp/position.rs`)
```rust
pub struct PositionUtils {
    rope: Rope,  // Efficient text representation
}

impl PositionUtils {
    pub fn offset_to_position(&self, offset: usize) -> Position;
    pub fn position_to_offset(&self, pos: Position) -> usize;
    pub fn span_to_range(&self, span: &Span) -> Range;
}
```

#### 3. Caching System (`crates/why_lib/src/lsp/cache.rs`)
```rust
pub struct AnalysisCache {
    parsed_asts: DashMap<Uri, Arc<Vec<TopLevelStatement<()>>>>,
    typed_asts: DashMap<Uri, Arc<Vec<TopLevelStatement<TypeInformation>>>>,
    symbol_indices: DashMap<Uri, Arc<SymbolIndex>>,
    file_versions: DashMap<Uri, i64>,
}
```

### Required Dependencies

Add to `Cargo.toml`:
```toml
[dependencies]
# Existing dependencies remain...
dashmap = "5.5"           # Concurrent hashmaps for caching
ropey = "1.6"             # Efficient text rope for position tracking
interval-tree = "0.2"     # Fast spatial queries for symbols
fuzzy-matcher = "0.3"     # Fuzzy string matching for completion
```

### Integration Points

#### 1. TypeChecker Extensions
Modify `crates/why_lib/src/typechecker/mod.rs`:
- Add symbol collection during type checking phase
- Store definition and reference locations
- Preserve type information for LSP queries

#### 2. LSP Server Enhancements
Modify `src/bin/yls.rs`:
- Add new LSP method handlers
- Implement incremental analysis
- Add comprehensive error handling and fallbacks

#### 3. Performance Optimizations
- **Incremental Updates**: Only re-analyze changed files
- **Background Processing**: Use tokio tasks for expensive operations
- **Memory Management**: Implement LRU cache for analyzed files
- **Cancellation**: Support request cancellation for long-running operations

### Error Handling Strategy

1. **Graceful Degradation**: Continue providing basic features when advanced analysis fails
2. **Partial Results**: Return partial completions/symbols when some analysis fails
3. **Error Recovery**: Attempt to provide helpful suggestions even with syntax errors
4. **User Feedback**: Clear error messages for configuration or setup issues

### Testing Strategy

#### 1. Unit Tests
- Test symbol resolution for all AST node types
- Verify position calculations and range conversions
- Test caching behavior and invalidation

#### 2. Integration Tests
- End-to-end LSP request/response testing
- Multi-file workspace scenarios
- Performance benchmarks for large codebases

#### 3. LSP Client Testing
Create test scripts using LSP clients:
```bash
#!/bin/bash
# test_lsp.sh - Basic functionality testing
echo "Testing go-to-definition..."
# Send LSP requests and verify responses
```

## Common Pitfalls & Solutions

### 1. Position Synchronization Issues
**Problem**: Mismatch between client and server document positions
**Solutions**:
- Implement robust text synchronization with document versioning
- Use UTF-16 code units consistently (LSP standard)
- Handle multi-byte characters correctly

### 2. Type Information Loss
**Problem**: Type context lost between analysis phases
**Solutions**:
- Cache full typed ASTs, not just untyped ones
- Preserve type information in symbol database
- Implement incremental type checking

### 3. Performance with Large Files
**Problem**: LSP operations become slow with large codebases
**Solutions**:
- Implement request cancellation tokens
- Use background tasks for heavy analysis
- Provide streaming results for large symbol queries
- Implement file size limits with graceful fallbacks

### 4. Multi-file Dependencies
**Problem**: Cross-file references and imports
**Solutions**:
- Build workspace-wide dependency graph
- Implement proper module resolution (when added to language)
- Handle circular dependencies gracefully

### 5. Memory Usage
**Problem**: Memory consumption grows with workspace size
**Solutions**:
- Implement LRU eviction for cached analysis results
- Use weak references where appropriate
- Provide configuration for cache limits

## Success Metrics

### Phase 1 Success Criteria
- [ ] Go-to-definition works for 95% of symbol types
- [ ] Find references returns complete results
- [ ] Document symbols show proper hierarchy
- [ ] No performance regression on existing features

### Phase 2 Success Criteria
- [ ] Hover shows accurate type information
- [ ] Completion provides relevant suggestions in context
- [ ] Semantic highlighting distinguishes all symbol types
- [ ] Features work with partially-invalid code

### Phase 3 Success Criteria
- [ ] Code actions fix common errors automatically
- [ ] Inlay hints provide useful type information
- [ ] No false positives in quick fixes
- [ ] Refactoring preserves semantic correctness

### Phase 4 Success Criteria
- [ ] Workspace symbols scale to 1000+ files
- [ ] Call hierarchy works for complex call chains
- [ ] All features maintain sub-100ms response times
- [ ] Complete feature parity with major language servers

## Future Enhancements

### Language Feature Support
- Module system and imports (when added to Y Lang)
- Generic types and constraints
- Trait/interface system
- Package management integration

### Advanced IDE Features
- Debugger integration (Debug Adapter Protocol)
- Test runner integration
- Integrated documentation generation
- Code coverage visualization
- Profiling and performance analysis

### Developer Experience
- LSP configuration options
- Telemetry and usage analytics
- Plugin system for extensions
- Integration with popular editors (VS Code, Neovim, Emacs)

This roadmap provides a comprehensive path from the current basic LSP implementation to a fully-featured language server that provides an excellent developer experience for Y Lang programmers.
