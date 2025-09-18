# Contributing to Y Language

Thank you for your interest in contributing to the Y programming language! This guide will help you get started with contributing to the compiler, language design, documentation, and ecosystem.

## Table of Contents

- [Getting Started](#getting-started)
- [Development Workflow](#development-workflow)
- [Contribution Guidelines](#contribution-guidelines)
- [Code Style and Standards](#code-style-and-standards)
- [Testing Requirements](#testing-requirements)
- [Documentation Standards](#documentation-standards)
- [Pull Request Process](#pull-request-process)
- [Issue Guidelines](#issue-guidelines)
- [Community and Communication](#community-and-communication)

## Getting Started

### Prerequisites

Before contributing, ensure you have the required tools installed. Prefer official rustup components over cargo-installed stand‑alone tools when available:

```bash
# Install Rust toolchain (stable + components)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source "$HOME/.cargo/env"

# Add required components (preferred over cargo install)
rustup component add rustfmt clippy llvm-tools-preview

# Install auxiliary developer tools
cargo install just          # Command runner
cargo install mdbook        # Documentation site
cargo install cargo-watch   # Rebuild on change

# (Optional) test coverage & property testing helpers
cargo install cargo-tarpaulin
cargo install cargo-nextest

# Verify installation
rustc --version
cargo --version
just --version
```

### Setting Up Your Development Environment

1. **Fork and Clone**
   ```bash
   # Fork the repository on GitHub, then clone your fork
   git clone https://github.com/YOUR_USERNAME/y-lang.git
   cd y-lang

   # Add upstream remote
   git remote add upstream https://github.com/H1ghBre4k3r/y-lang.git
   ```

2. **Build and Test**
   ```bash
   # Build the project
   just build

   # Run tests to ensure everything works
   just test

   # Test with examples
   cargo run --bin yc examples/simple.why -o out/simple
   ./out/simple
   ```

3. **Documentation Setup**
   ```bash
   # Build and serve documentation locally
   cd docs
   mdbook serve
   # Visit http://localhost:3000
   ```

### Development Tools

**Just Commands** (see `justfile` for complete list):
```bash
just build           # Build in debug mode
just build-release   # Build optimized release
just test           # Run all tests
just test-unit      # Run only unit tests
just test-integration # Run integration tests
just watch          # Watch for changes and rebuild
just fmt            # Format code
just lint           # Run clippy
just doc            # Generate documentation
just clean          # Clean build artifacts
```

**Useful Cargo Commands**:
```bash
cargo check         # Quick syntax check
cargo clippy        # Advanced linting
cargo fmt           # Format code
cargo doc --open    # Generate and open Rust docs
cargo test -- --nocapture # Run tests with output
```

## Development Workflow

### Branch Strategy

We use a simplified Git workflow:

- **`main`** - Stable release branch
- **Feature branches** - `feature/description` or `fix/description`
- **Documentation** - `docs/description`

### Typical Development Cycle

1. **Create Feature Branch**
   ```bash
   git checkout main
   git pull upstream main
   git checkout -b feature/my-new-feature
   ```

2. **Develop and Test**
   ```bash
   # Make your changes
   vim src/some_file.rs

   # Test continuously
   just watch &

   # Run specific tests
   just test parser

   # Format and lint
   just fmt
   just lint
   ```

3. **Commit Changes**
   ```bash
   # Stage changes
   git add .

   # Commit with descriptive message
   git commit -m "feat: add conditional expressions to language

   - Add ternary operator (condition ? true_expr : false_expr)
   - Implement parsing in grammar.rs
   - Add type checking for conditional expressions
   - Generate LLVM IR for conditional branches
   - Add comprehensive tests and examples

   Closes #123"
   ```

4. **Push and Create PR**
   ```bash
   git push origin feature/my-new-feature
   # Create pull request on GitHub
   ```

### Hot Reload Development

For rapid development cycles:

```bash
# Terminal 1: Auto-rebuild on changes
just watch

# Terminal 2: Test your changes
cargo run --bin yc examples/test.why -o out/test && ./out/test

# Terminal 3: Run specific tests
cargo test parser -- --nocapture
```

## Contribution Guidelines

### What We're Looking For

**High Priority Contributions:**
- **Language Features**: New syntax, operators, control structures
- **Standard Library**: Built-in functions and data structures
- **Developer Tools**: IDE integration, debugging support
- **Performance**: Compilation speed and generated code optimization
- **Documentation**: Tutorials, examples, API documentation
- **Testing**: Comprehensive test coverage, property testing

**Medium Priority:**
- **Error Messages**: Improved error reporting and suggestions
- **Tooling**: Build system improvements, package management
- **Examples**: Real-world Y language programs
- **Benchmarks**: Performance measurement and tracking

**Lower Priority:**
- **Refactoring**: Code cleanup (unless fixing bugs)
- **Minor Formatting**: Style changes without functional improvement

### Types of Contributions

#### 1. Language Design
- Propose new language features in GitHub issues
- Provide use cases and examples
- Consider impact on existing code
- Discuss with maintainers before implementing

#### 2. Compiler Implementation
- Follow the [Extending the Compiler](docs/src/implementation/extending-compiler.md) guide
- Implement features across all compiler stages
- Ensure backward compatibility
- Add comprehensive tests

#### 3. Documentation
- Improve existing documentation
- Add examples and tutorials
- Update API documentation
- Fix typos and unclear explanations

#### 4. Testing
- Add unit tests for new features
- Create integration tests for full pipeline
- Write property-based tests where applicable
- Add regression tests for bug fixes

### Before Starting Work

1. **Check Existing Issues**: Look for existing issues or discussions
2. **Create an Issue**: For new features, create an issue to discuss approach
3. **Ask Questions**: Use GitHub Discussions for design questions
4. **Start Small**: Begin with small contributions to understand the codebase

### Feature Addition Checklist

Follow this sequence for any user-visible language feature (syntax / semantics):

1. Design
   - [ ] Open issue describing syntax + examples
   - [ ] Consider precedence / associativity interactions
   - [ ] Identify impact on type system & codegen

2. Grammar & AST
   - [ ] Update grammar definitions
   - [ ] Add / adjust AST node(s)
   - [ ] Add parser unit tests (success + error)

3. Type System
   - [ ] Extend TypeInformation / inference constraints if needed
   - [ ] Add type checker logic
   - [ ] Add inference tests (including edge cases)

4. Code Generation
   - [ ] Implement lowering & IR emission
   - [ ] Add codegen tests / example programs
   - [ ] Validate IR correctness (no invalid memory ops)

5. Runtime / Semantics
   - [ ] Ensure evaluation order matches spec
   - [ ] Add regression test for discovered corner cases

6. Tooling
   - [ ] Update formatter rules if syntax affects formatting
   - [ ] Update LSP / diagnostics (if applicable)

7. Documentation
   - [ ] Update language reference section
   - [ ] Add example in examples/ directory
   - [ ] Document invariants & limitations in implementation docs

8. Validation
   - [ ] Run full test suite (unit + integration)
   - [ ] Build in release mode (optimization issues?)
   - [ ] Benchmark if performance-sensitive

9. PR Finalization
   - [ ] Re-run PR checklist below
   - [ ] Ensure no leftover experimental flags

## Code Style and Standards

### Enforced Policies (CI Gate)

These are non-negotiable requirements; PRs failing any gate will be blocked:

| Policy | Enforcement | Command |
|--------|-------------|---------|
| Formatting | rustfmt --check | just fmt (ensures stable style) |
| Lint Cleanliness | clippy (deny warnings) | just lint (configured with -D warnings) |
| Build Integrity | cargo build --workspace | just build |
| Test Pass | cargo test --workspace | just test |
| Rustdoc Warnings | cargo doc (deny rustdoc warnings) | just doc |
| No Panics in New Code | code review + grep heuristic | (avoid unwrap/expect outside tests) |

Guidelines:
- Prefer Result over panic for recoverable failures.
- Use explicit error enums; avoid Stringly-typed errors.
- Keep functions ≤ ~80 lines; refactor otherwise.
- Avoid premature allocation; pass &str/&[T] where possible.
- Zero unsafe code unless justified with rationale + safety comment block.

Additions impacting performance MUST include a micro-benchmark or rationale in PR description.

### Rust Code Style

We follow standard Rust conventions with some project-specific guidelines:

#### Formatting
```bash
# Use rustfmt with project configuration
cargo fmt

# Check formatting without making changes
cargo fmt -- --check
```

#### Naming Conventions
- **Modules**: `snake_case` (e.g., `type_checker`, `code_gen`)
- **Functions**: `snake_case` (e.g., `parse_expression`, `infer_type`)
- **Types**: `PascalCase` (e.g., `Expression`, `TypeInformation`)
- **Constants**: `SCREAMING_SNAKE_CASE` (e.g., `MAX_RECURSION_DEPTH`)
- **Variables**: `snake_case` (e.g., `current_token`, `return_type`)

#### Documentation
```rust
/// Brief one-line description of the function.
///
/// More detailed explanation of what the function does, including
/// its behavior, assumptions, and any important implementation details.
///
/// # Arguments
///
/// * `param1` - Description of first parameter
/// * `param2` - Description of second parameter
///
/// # Returns
///
/// Description of return value and its meaning.
///
/// # Errors
///
/// Description of when and why the function might fail.
///
/// # Examples
///
/// ```rust
/// let result = my_function(42, "hello");
/// assert_eq!(result, expected_value);
/// ```
pub fn my_function(param1: i32, param2: &str) -> Result<String, MyError> {
    // Implementation
}
```

#### Error Handling
```rust
// Prefer explicit error types
pub enum ParseError {
    UnexpectedToken { expected: String, found: String, position: Span },
    UnterminatedString { position: Span },
    InvalidNumber { value: String, position: Span },
}

// Use ? operator for error propagation
fn parse_number(&mut self) -> Result<i64, ParseError> {
    let token = self.next_token()?;
    token.as_number().ok_or_else(|| ParseError::InvalidNumber {
        value: token.text.clone(),
        position: token.span,
    })
}
```

#### Testing Style
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_function_name_describes_scenario() {
        // Arrange
        let input = create_test_input();
        let expected = ExpectedResult::new();

        // Act
        let result = function_under_test(input);

        // Assert
        assert_eq!(result, expected);
    }

    #[test]
    fn test_error_case_with_descriptive_name() {
        let invalid_input = create_invalid_input();

        let result = function_under_test(invalid_input);

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ErrorType::SpecificVariant));
    }
}
```

### Commit Message Format

We use conventional commits for clear history:

```
<type>(<scope>): <description>

<body>

<footer>
```

**Types:**
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Code style changes (formatting, no logic changes)
- `refactor`: Code refactoring
- `test`: Adding or updating tests
- `chore`: Maintenance tasks

**Scopes:**
- `lexer`: Lexical analysis
- `parser`: Parsing & AST lowering
- `typechecker`: Inference & validation
- `codegen`: LLVM IR generation
- `runtime`: (future) runtime support
- `cli`: Command-line interface
- `lsp`: Language server / editor integration
- `docs`: Documentation (mdBook / rustdoc)
- `tests`: Test harnesses & fixtures
- `infra`: CI, build system, scripts

Rules:
- Use present tense (feat(parser): add X)
- Body wraps at ~100 cols
- Reference issues in footer (Closes #NN)
- Multiple breaking changes require a BREAKING CHANGE: footer summarizing them
- Avoid mixing unrelated changes; perform logical splits

**Examples:**
```
feat(parser): add support for conditional expressions

Implement ternary operator (condition ? true_expr : false_expr) with:
- Grammar rules for conditional expressions
- AST node representation
- Parsing logic with proper precedence

Closes #45

fix(typechecker): resolve infinite loop in recursive type checking

The type checker was entering infinite loops when checking mutually
recursive function types. Added cycle detection to break loops.

Fixes #67

docs(implementation): add guide for extending the compiler

Added comprehensive guide covering:
- Development setup and workflow
- Step-by-step feature addition process
- Common patterns and debugging tips
```

## Testing Requirements

### Test Coverage Expectations

Test Tiers (run time budgets approximate):
- Tier 1 (Fast < 1s): Unit tests (mandatory every PR)
- Tier 2 (Medium 1–5s): Core integration (executed in CI)
- Tier 3 (Extended 5–20s): Property + stress (nightly / optional pre-merge)
- Tier 4 (Benchmarks): Performance tracking (run locally before perf PRs)

All contributions must include appropriate tests:

#### 1. Unit Tests
- **Required for**: All new functions and methods
- **Location**: In same file as implementation (`#[cfg(test)]` modules)
- **Coverage**: Test both success and error cases

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_simple_expression() {
        let input = "42 + 24";
        let result = parse_expression(input).unwrap();

        assert!(matches!(result, Expression::Binary { .. }));
    }

    #[test]
    fn parse_expression_handles_invalid_syntax() {
        let input = "42 +";
        let result = parse_expression(input);

        assert!(result.is_err());
    }
}
```

#### 2. Integration Tests
- **Required for**: New language features
- **Location**: `tests/` directory
- **Coverage**: Test full compilation pipeline

```rust
// tests/integration_test.rs
use y_lang::compile_file;

#[test]
fn conditional_expression_integration() {
    let source = r#"
        fn main(): i64 {
            let x = 5;
            return x > 3 ? 10 : 20;
        }
    "#;

    let result = compile_and_run_string(source).unwrap();
    assert_eq!(result.exit_code, 10);
}
```

#### 3. Property Tests
- **Recommended for**: Complex algorithms, parsers
- **Tool**: `proptest` crate
- **Purpose**: Test invariants across many inputs

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn roundtrip_parse_format(expr in any_expression()) {
        let formatted = format_expression(&expr);
        let reparsed = parse_expression(&formatted).unwrap();
        assert_eq!(expr, reparsed);
    }
}
```

### Running Tests

```bash
# Run all tests
just test

# Run specific test module
cargo test parser

# Run tests with output
cargo test -- --nocapture

# Run tests with specific pattern
cargo test conditional

# Run ignored tests (slow tests)
cargo test -- --ignored

# Generate test coverage report
cargo install cargo-tarpaulin
cargo tarpaulin --out Html
```

### Test Organization

```
tests/
├── integration/
│   ├── language_features/
│   │   ├── conditionals.rs
│   │   ├── loops.rs
│   │   └── functions.rs
│   ├── error_handling/
│   │   ├── parse_errors.rs
│   │   └── type_errors.rs
│   └── examples/
│       └── example_programs.rs
├── unit/
│   ├── parser_tests.rs
│   ├── typechecker_tests.rs
│   └── codegen_tests.rs
└── property/
    ├── parser_roundtrip.rs
    └── type_preservation.rs
```

## Performance & Profiling Guidance

Goals:
- Keep incremental compile fast (< 500ms for small examples)
- Avoid quadratic passes unless justified

When adding potentially expensive logic:
- Measure with `cargo build --timings`
- Add criterion benchmarks for micro performance claims

Memory:
- Prefer borrowing (&str, &[T]) over cloning
- Preallocate with with_capacity in tight loops
- Minimize Rc<RefCell<..>> on hot paths

LLVM:
- Reuse values to avoid redundant loads
- Group related IR emission to reduce builder repositioning

Profiling Commands:
```bash
cargo build --release
cargo build --timings
valgrind --tool=callgrind ./target/release/yc examples/simple.why -o out/simple # if available
```

Document rationale when complexity increases for performance.

## Documentation Standards

### Types of Documentation

#### 1. API Documentation (Rustdoc)
- **Required for**: All public APIs
- **Format**: Rust doc comments (`///`)
- **Content**: Purpose, parameters, return values, examples

#### 2. Implementation Documentation (mdBook)
- **Location**: `docs/src/implementation/`
- **Audience**: Contributors and maintainers
- **Content**: Architecture, algorithms, design decisions

#### 3. User Documentation (mdBook)
- **Location**: `docs/src/`
- **Audience**: Y language users
- **Content**: Language guide, tutorials, reference

#### 4. Examples
- **Location**: `examples/`
- **Purpose**: Demonstrate language features
- **Requirements**: Must compile and run correctly

### Documentation Guidelines

#### Rustdoc Coverage Requirements

Minimum coverage targets (monitored manually for now):
- 100% for all public enums / structs / traits
- 100% for all public functions & methods
- Module-level `//!` for every top-level module in crates/why_lib/src
- Safety section required for any unsafe block (must explain invariants)

Checklist before opening PR:
- Run: cargo doc --workspace --no-deps
- Ensure: No warnings related to missing docs (treat as errors soon)
- Provide at least one runnable example (/// ```rust ... ```) for new public APIs

Style:
- First line: imperative short summary (< 80 chars)
- Blank line, then extended rationale if needed
- Use intra-doc links: [`TypeName`], [`module::Item`]
- Prefer # Errors / # Panics sections when relevant

Example minimal:
```rust
/// Builds the typed AST from a parsed module.
///
/// Performs name resolution and attaches `TypeInformation` placeholders that
/// are later solved during inference.
pub fn lower(parsed: ParsedModule) -> TypedModule { /* ... */ }
```

#### Writing Style
- **Clear and Concise**: Use simple, direct language
- **Examples**: Include code examples for complex concepts
- **Structure**: Use headings and lists for organization
- **Links**: Cross-reference related documentation

#### Code Examples
```rust
// Good: Clear, commented example
/// Parse a function declaration from tokens.
///
/// # Examples
///
/// ```rust
/// let tokens = tokenize("fn add(x: i64, y: i64): i64 { return x + y; }");
/// let function = parse_function(&mut tokens).unwrap();
/// assert_eq!(function.name, "add");
/// assert_eq!(function.parameters.len(), 2);
/// ```
```

#### Updating Documentation
```bash
# Build documentation locally
cd docs
mdbook serve

# Generate Rust API docs
cargo doc --open

# Check for broken links
mdbook test

# Spell check (if available)
aspell check docs/src/implementation/parsing.md
```

## Pull Request Process

### Before Submitting

1. **Ensure Tests Pass**
   ```bash
   just test
   just lint
   just fmt
   ```

2. **Update Documentation**
   - Add/update relevant documentation
   - Include examples if applicable
   - Update CHANGELOG.md if making user-visible changes

3. **Check Compatibility**
   - Ensure existing examples still work
   - Test with both debug and release builds
   - Verify no breaking changes to public APIs

### PR Checklist

- [ ] Code follows project style guidelines (fmt + clippy clean)
- [ ] All tests pass locally (unit + integration)
- [ ] New code has unit tests
- [ ] New feature has integration test and example (if user-facing)
- [ ] Public APIs documented (rustdoc + examples)
- [ ] Implementation docs updated where architecture changed
- [ ] Commit messages follow conventional format
- [ ] No merge conflicts with main branch
- [ ] Performance impact measured or rationale provided (n/a if trivial)
- [ ] Breaking changes clearly documented with migration notes
- [ ] No new unsafe blocks (or safety rationale documented)
- [ ] No leftover dbg!/println!/todo!/unimplemented! macros
- [ ] All examples still compile & run

### PR Description Template

```markdown
## Description
Brief description of the changes and their purpose.

## Type of Change
- [ ] Bug fix (non-breaking change which fixes an issue)
- [ ] New feature (non-breaking change which adds functionality)
- [ ] Breaking change (fix or feature that would cause existing functionality to not work as expected)
- [ ] Documentation update

## Testing
Description of tests added or updated to verify the changes.

## Performance Impact
Description of any performance implications (positive or negative).

## Related Issues
Fixes #(issue number)
Closes #(issue number)
```

### CI Expectations

Current CI (subject to evolution) runs:
- build (debug + release)
- formatting (rustfmt --check)
- lint (clippy -D warnings)
- tests (unit + integration)
- docs (cargo doc) – warnings treated as failures when possible

Future additions may include:
- code coverage threshold gate
- benchmark regression detection
- link + spell check for docs

If CI fails:
1. Read failing job logs fully
2. Reproduce locally (same command) before pushing fix
3. Amend commit instead of piling fix commits, unless multiple failures

### Review Process

1. **Automated Checks**: CI runs tests, linting, and formatting checks
2. **Code Review**: Maintainers review code for quality and design
3. **Discussion**: Address feedback and make requested changes
4. **Approval**: At least one maintainer approval required
5. **Merge**: Maintainer merges after all checks pass

### Review Criteria

**Code Quality:**
- Follows project conventions
- Handles errors appropriately
- Includes appropriate comments
- Has good test coverage

**Design:**
- Fits well with existing architecture
- Doesn't introduce unnecessary complexity
- Considers future extensibility
- Maintains backward compatibility

**Documentation:**
- Public APIs are documented
- Complex algorithms are explained
- User-facing changes are documented

## Issue Guidelines

### Reporting Bugs

Use the bug report template:

```markdown
## Bug Description
Clear and concise description of the bug.

## Steps to Reproduce
1. Go to '...'
2. Run command '...'
3. See error

## Expected Behavior
What you expected to happen.

## Actual Behavior
What actually happened.

## Environment
- OS: [e.g., Ubuntu 20.04]
- Rust version: [e.g., 1.70.0]
- Y compiler version: [e.g., 0.1.0]

## Additional Context
Any other context about the problem.

## Minimal Example
```why
// Minimal Y code that reproduces the issue
fn main(): void {
    // problem code here
}
```

### Feature Requests

Use the feature request template:

```markdown
## Feature Summary
Brief description of the feature.

## Motivation
Why is this feature needed? What problem does it solve?

## Detailed Description
Detailed explanation of the feature and how it should work.

## Examples
```why
// Example of how the feature would be used
fn example() {
    // new feature syntax
}
```

## Alternatives Considered
Other ways to solve the problem.

## Implementation Notes
Any thoughts on how this could be implemented.
```

### Issue Labels

- **bug**: Something isn't working
- **enhancement**: New feature or request
- **documentation**: Improvements or additions to documentation
- **good first issue**: Good for newcomers
- **help wanted**: Extra attention is needed
- **question**: Further information is requested
- **wontfix**: This will not be worked on

## Community and Communication

### Code of Conduct

We are committed to providing a welcoming and inclusive environment. Please read and follow our [Code of Conduct](CODE_OF_CONDUCT.md).

### Communication Channels

- **GitHub Issues**: Bug reports, feature requests
- **GitHub Discussions**: Design discussions, questions
- **Pull Requests**: Code review and discussion

### Getting Help

1. **Documentation**: Check the [implementation docs](docs/src/implementation/)
2. **Examples**: Look at existing code and examples
3. **Issues**: Search existing issues for similar problems
4. **Discussions**: Ask questions in GitHub Discussions
5. **Code Review**: Learn from reviewing others' pull requests

### Mentorship

New contributors are always welcome! If you're interested in contributing but don't know where to start:

1. Look for issues labeled "good first issue"
2. Read the implementation documentation
3. Ask questions in GitHub Discussions
4. Start with small improvements (documentation, tests)
5. Gradually work on larger features

### Recognition

Contributors are recognized in:
- `CONTRIBUTORS.md` file
- Release notes for significant contributions
- GitHub contributors graph
- Special mentions for major features

Thank you for contributing to Y language! Your contributions help make Y a better programming language for everyone.