# Advanced Topics

This section covers the technical implementation details of the Y language compiler and runtime system. These documents are intended for compiler developers, contributors, and those interested in understanding how Y works under the hood.

## Contents

- [Closure Implementation](./closure-implementation.md) - Technical details of how closures and variable capture work
- [Compiler Architecture](./compiler-architecture.md) - Overview of the compilation pipeline and major components

## Target Audience

These documents assume familiarity with:
- Compiler design and implementation
- LLVM IR (Intermediate Representation)
- Rust programming language
- Memory management concepts
- Type system implementation

## Contributing

If you're working on the Y compiler and need to document new implementation details, please add them to this section. Keep the user-facing documentation in the `basics/` and `intermediate/` sections separate from these technical implementation details.