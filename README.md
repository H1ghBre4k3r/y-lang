# Y Lang

An experimental, expression-centric programming language implemented in Rust. Y treats most constructs as expressions and targets LLVM via Inkwell. This repo includes the core library (why_lib) and three binaries: yc (compiler), yfmt (formatter), and yls (language server).

> Status: Early-stage and evolving. See ROADMAP.md for planned work.

## Table of contents
- Features
- Project structure
- Installation
- Quick start
- Usage reference (yc, yfmt, yls)
- Language examples
- Documentation
- Code generation status (checklist)
- Development
- Editor/LSP integration
- Building the docs site
- Troubleshooting
- License

## Features
- Expression-first design (blocks/if/while as expressions)
- Static typing with inference and annotations
- Functions as first-class values, lambdas (design in progress)
- Structs and instances (design and parser support present)
- LLVM-based code generation

## Project structure
- Root crate (y-lang)
  - Binaries
    - src/bin/yc.rs — compiler frontend/driver
    - src/bin/yfmt.rs — formatter CLI
    - src/bin/yls.rs — language server (LSP over stdio)
- crates/
  - why_lib — core implementation (lexer, parser, typechecker, codegen)
  - lex_derive — procedural macros for lexer generation
- docs/
  - src — mdBook sources for language/implementation docs
  - book — prebuilt HTML (if present)
- examples/ — sample .why programs
- ROADMAP.md — planned features and milestones

## Installation

Prerequisites
- Rust toolchain (latest stable)
- LLVM 18 (required by Inkwell 0.6 with llvm18-1)
- just (optional task runner)
- mdBook (optional, for building docs)

Tips for LLVM 18
- Ensure your environment exposes LLVM 18. If your package manager installs it in a non-default prefix, set an env var like:
  - macOS (Homebrew): `export LLVM_SYS_180_PREFIX="$(brew --prefix llvm@18)"`
  - Otherwise: point `LLVM_SYS_180_PREFIX` to your LLVM 18 install directory (the one containing bin/llvm-config)

Install optional tools
- macOS (Homebrew): `brew install just mdbook llvm@18`
- Linux: install equivalents from your package manager

Build
- With just: `just build`
- With Cargo: `cargo build --workspace`
- Release build: `just build-release` or `cargo build --release --workspace`

Binaries are built in `target/debug/` (or `target/release/`).

## Quick start

Compile and run an example

```bash
# Compile examples/hello.why to an executable
./target/debug/yc examples/hello.why -o out/hello
./out/hello
```

Format code

```bash
# Print formatted code to stdout
./target/debug/yfmt examples/hello.why

# Format in place (file or directory)
./target/debug/yfmt examples -i
```

Language server (LSP over stdio)

```bash
# Starts an LSP server on stdio; configure your editor to launch this binary
./target/debug/yls
```

## Usage reference

yc

```text
USAGE: yc <file> [options]

Options:
  -l, --print-lexed        Print the lexed source tree
  -p, --print-parsed       Print the parsed AST
  -c, --print-checked      Print the typechecked AST
  -v, --print-validated    Print the validated AST
      --format             Pretty-print formatted source to stdout
      --format-output <p>  Write formatted source to a file
  -o <path>                Output executable path (default: a.out)
```

yfmt

```text
USAGE: yfmt <path> [options]

Args:
  path                     File or directory to format

Options:
  -i, --in-place           Edit files in place
```

yls
- Implements an LSP server over stdio. Point your editor’s LSP client to the `yls` binary.

## Language examples

Hello world and simple functions (from examples/hello.why and examples/simple.why)

```why
declare printf: (str) -> void;

fn println(val: str): void {
    printf(val);
    printf("\n");
}

fn add(x: i64, y: i64): i64 {
    x + y
}

fn main(): i64 {
    println("Hello, world!");
    let a = add(42, 1337);
    a
}
```

From examples/simple.why

```why
declare printf: (str) -> void;

fn baz(x: i64): i64 {
    let intermediate = x * 2;
    return intermediate;
}

fn main(): i64 {
    printf("Foo\n");
    let x = 12;
    let a = baz(x);
    return x + a;
}
```

Structs and instance methods (from examples/struct.why)

```why
struct FooStruct {
    id: i64;
    amount: f64;
}

instance FooStruct {
    fn get_id(): i64 { this.id }
    fn get_amount(): f64 { this.amount }
}
```

More examples live in the `examples/` directory.

## Documentation
- Language and implementation docs (mdBook sources): `docs/src/`
- Prebuilt HTML docs (if present): `docs/book/`
- Roadmap: [ROADMAP.md](./ROADMAP.md)

## Code generation status (checklist)

Expressions
- [x] Integer and floating-point literals
- [x] Boolean literals
- [x] String literals (as global string pointers)
- [x] Identifiers (load from alloca when applicable)
- [x] Prefix operators: logical not (!bool), numeric minus (-int/-float)
- [x] Binary arithmetic: +, -, *, / for int/float
- [ ] Comparisons and logical ops (==, !=, <, &&, ||, ...)
- [ ] Character literals
- [ ] Arrays (literal, index, mutation)
- [ ] Struct literals and field access
- [ ] If expressions
- [ ] While loops
- [ ] Lambda/closure literals

Blocks and statements
- [x] Block expressions (yield last expression)
- [x] Variable initialization (alloca + store)
- [ ] Assignment updates
- [ ] Constants

Functions
- [x] Function declaration (prototype) emission
- [x] Function definition with parameter allocas and returns
- [x] Calls (direct/indirect via function pointers)
- [ ] Functions returning/accepting strings, chars, aggregates (full support)

Types and layout
- [x] Integer, float, bool mapping
- [x] Tuples and structs (type mapping)
- [ ] Arrays
- [ ] References and advanced aggregates

Control flow and misc
- [ ] If/else lowering
- [ ] While lowering
- [ ] PHI nodes and advanced flow

Update this checklist as features land.

## Development

Common tasks (using just)
- `just build` / `just b` — build the project
- `just test` / `just t` — run tests across the workspace
- `just watch` — watch and rebuild binaries
- `just bins` / `just bins-release` — build all binaries

With Cargo
- `cargo build` — build the workspace
- `cargo test --workspace` — run all tests
- `cargo run --bin yc -- <file>` — compile a .why file
- `cargo run --bin yfmt -- <path>` — format a file or directory
- `cargo run --bin yls` — start the language server

## Editor/LSP integration
- yls speaks LSP over stdio. Configure your editor’s LSP client to:
  - Launch the built `yls` binary
  - Attach to files with the `.why` extension
  - Capabilities: full document sync, formatting, basic diagnostics
- For most editors, you can point a “custom”/“external” LSP to `./target/debug/yls`.

## Building the docs site
- Build once: `mdbook build docs`
- Serve with live reload: `mdbook serve docs -o`
- Output goes to `docs/book/`

## Troubleshooting
- LLVM version errors (e.g., LLVM_SYS_180): ensure LLVM 18 is installed and set `LLVM_SYS_180_PREFIX` to its prefix if needed.
- Linker/toolchain issues: ensure a C toolchain is installed (e.g., Xcode Command Line Tools on macOS; build-essential/clang on Linux).
- Undefined externs (e.g., printf): declare the function in .why and ensure your system toolchain provides the symbol at link time.

## License
GPL-3.0
