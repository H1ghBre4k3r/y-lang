# Contributing to Y Lang

Thanks for your interest in contributing! This document explains how to set up your environment, the project layout, coding standards, workflows, and how to propose changes.

## Table of contents
- Code of conduct
- Project scope and architecture
- Getting started (dev setup)
- Build, test, lint
- Running the binaries
- Docs workflow
- Adding language features
- Commit, branch, and PR guidelines
- Issue reporting and triage
- Style and conventions
- Security and disclosure

## Code of conduct
Be respectful and constructive. Harassment, discrimination, or disrespectful behavior is not tolerated.

## Project scope and architecture
This repo contains:
- Core library (why_lib): lexer, parser, typechecker, codegen
- Binaries: yc (compiler), yfmt (formatter), yls (LSP server)
- mdBook docs in docs/
- Examples in examples/

High-level pipeline (why_lib):
1) Lexing → 2) Parsing (AST) → 3) Type checking (inference + validation) → 4) Codegen (LLVM via Inkwell)

Key paths:
- Grammar: crates/why_lib/src/grammar.rs
- Lexer: crates/why_lib/src/lexer/
- Parser + AST: crates/why_lib/src/parser/
- Type checker: crates/why_lib/src/typechecker/
- Codegen: crates/why_lib/src/codegen/
- Formatter: crates/why_lib/src/formatter/

## Getting started (dev setup)
Prerequisites
- Rust (stable)
- LLVM 18 (required by Inkwell)
- just (optional)
- mdBook (optional, for docs)

LLVM setup
- Ensure LLVM 18 is discoverable. If installed in a non-default prefix, set:
  - macOS (Homebrew): export LLVM_SYS_180_PREFIX="$(brew --prefix llvm@18)"
  - Other: set LLVM_SYS_180_PREFIX to the install prefix containing bin/llvm-config

Install tools (examples)
- macOS: brew install just mdbook llvm@18
- Linux: install equivalents via your package manager

## Build, test, lint
Build
- just build
- cargo build --workspace

Tests
- just test
- cargo test --workspace

Lint and formatting
- cargo fmt --all
- cargo clippy --workspace --all-targets -- -D warnings

Release builds
- just build-release
- cargo build --release --workspace

## Running the binaries
Compiler
- ./target/debug/yc path/to/file.why -o out/a.out
- Flags: -l (lexed), -p (parsed), -c (checked), -v (validated), --format, --format-output <path>, -o <out>

Formatter
- ./target/debug/yfmt path/or/dir [-i]

LSP server
- ./target/debug/yls (stdio). Point your editor’s LSP client to this binary.

## Docs workflow
- Build docs: mdbook build docs
- Serve docs: mdbook serve docs -o
- Sources live in docs/src; output goes to docs/book

## Adding language features
When adding or changing a language feature:
1) Grammar: update crates/why_lib/src/grammar.rs
2) AST: add/adjust nodes in crates/why_lib/src/parser/ast/**
3) Type system: update crates/why_lib/src/typechecker/**
4) Codegen: implement lowering in crates/why_lib/src/codegen/** (Inkwell/LLVM)
5) Formatter: update pretty-printer in crates/why_lib/src/formatter/**
6) Tests: add unit tests and runnable .why examples in examples/
7) Docs: update user docs and/or implementation docs in docs/src

Keep the two-phase type checking model in mind: signatures/structs first, then full checking/validation.

## Commit, branch, and PR guidelines
Branches
- Use short, descriptive names (e.g., feat/codegen-binary-ops, fix/parser-precedence)

Commits
- Keep commits focused. Prefer small, logically isolated changes.
- Write clear messages; imperative mood (e.g., “Add X”, “Fix Y”).

Pull requests
- Include a summary of the change and rationale.
- Add tests and docs when applicable.
- Ensure CI basics pass locally: build, fmt, clippy, tests.
- Reference related issues (if any).

## Issue reporting and triage
When filing an issue, include:
- Repro steps and expected vs actual behavior
- Version info (Rust, LLVM), OS
- Minimal .why snippet if parser/typechecker/codegen is involved

Labels and triage
- Good first issues: targeted, low-risk improvements
- Feature: language or compiler feature work
- Bug: defects in lexer/parser/typechecker/codegen/formatter/LSP

## Style and conventions
Rust
- Follow rustfmt defaults (cargo fmt)
- Fix clippy lints (cargo clippy)

Code structure
- Prefer small modules and clear layering per the compiler pipeline
- Avoid panics in library code where recoverable errors are possible; use anyhow/thiserror patterns as appropriate
- Don’t log or commit secrets; avoid unsafe unless justified

Testing
- Unit tests close to implementation
- Examples in examples/ runnable via yc

## Security and disclosure
- Do not include credentials, tokens, or secrets in code or tests.
- Report potential security issues privately to the maintainer.

Thank you for contributing!