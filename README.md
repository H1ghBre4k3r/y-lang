# Y Programming Language

> ⚠️ **Experimental Project**: Y is currently in active development and not ready for production use. The language specification, syntax, and features are subject to change.

Y is a modern, expression-oriented programming language with first-class functions, structural typing, and compile-time safety. Built with Rust and LLVM, Y aims to provide powerful abstractions while maintaining performance and reliability.

## 🚀 Current Status (v0.1 - 75% Complete)

- ✅ **Parser & Type System**: Excellent (95% complete) - sophisticated syntax parsing and type inference
- ✅ **Core Language Features**: Structs, functions, lambdas, control flow, basic arrays
- ✅ **LLVM Code Generation**: Good (75% complete) - stable for supported features
- ⚠️ **Known Limitations**: Some advanced assignments and closure capture not yet implemented

**What Works Now**: Basic programs, struct types with methods, non-capturing lambdas, property access  
**In Development**: Complex assignments, closure capture, empty array initialization

## 📋 Quick Example

```rust
// Struct types with instance methods
struct Point {
    x: i64;
    y: i64;
}

instance Point {
    fn distance_from_origin(): f64 {
        // Property access works
        let x_sq = this.x * this.x;
        let y_sq = this.y * this.y;
        sqrt(x_sq + y_sq)
    }
}

// First-class functions and lambdas
fn takes_function(f: (i64) -> i64): i64 {
    f(42)
}

fn main(): i64 {
    // Struct initialization
    let point = Point { x: 3, y: 4 };
    
    // Lambda expressions (non-capturing)
    let double = \(x) => x * 2;
    
    // Function calls
    let result = takes_function(double);
    
    // Control flow
    if result > 50 {
        point.distance_from_origin() as i64
    } else {
        result
    }
}
```

## 🛠️ Installation & Usage

### Prerequisites
- Rust 1.70+ with Cargo
- LLVM 15+ development libraries
- Just command runner (optional but recommended)

### Building from Source

```bash
# Clone the repository
git clone https://github.com/your-username/y-lang.git
cd y-lang

# Build the project
just build
# OR use Cargo directly
cargo build --workspace

# Run tests
just test
```

### Available Binaries

#### `yc` - Y Compiler
Compiles Y source files to native executables.

```bash
# Compile a Y program
cargo run --bin yc -- examples/simple.why -o output
./output

# Or after installation
yc examples/simple.why -o my_program
```

#### `yls` - Y Language Server
Provides IDE support via Language Server Protocol.

```bash
# Start the language server
cargo run --bin yls
```

#### `yfmt` - Y Code Formatter
Formats Y source code (planned feature).

```bash
# Format Y files (coming soon)
yfmt src/main.why
```

### Quick Start Commands

```bash
# Build in development mode
just build

# Build optimized release
just build-release

# Run all tests
just test

# Build all binaries
just bins

# Watch for changes and rebuild
just watch

# Install locally
just install
```

## 📚 Language Features

### ✅ Implemented Features

- **Expression-oriented syntax** - Everything returns a value
- **Struct types** with field access and instance methods
- **First-class functions** - Functions as values, higher-order functions
- **Lambda expressions** - `\(x, y) => x + y` syntax (non-capturing)
- **Static typing** with type inference
- **Control flow** - `if` expressions, `while` loops
- **Property access** - Dot notation for struct fields
- **Function pointers** - Pass functions as parameters

### 🔄 In Development

- **Complex assignments** - `obj.field = value`, `arr[index] = value`
- **Closure capture** - Lambdas accessing surrounding variables
- **Empty array initialization** - `&[]` syntax
- **Enhanced error messages** - Better compilation diagnostics

### 📋 Planned Features

- **Pattern matching** - `match` expressions with destructuring
- **Generic types** - `Vec<T>`, parameterized functions
- **Module system** - Import/export, package management
- **Standard library** - Collections, I/O, string operations
- **Async/await** - Concurrency primitives

## 📖 Documentation

- **[Language Examples](examples/)** - Working code samples demonstrating features
- **[Development Roadmap](ROADMAP.md)** - Current status and future plans
- **[Architecture Guide](docs/)** - Compiler internals and design decisions

## 🧪 Try It Out

Explore the working examples to see what Y can do:

```bash
# Basic function and struct example
cargo run --bin yc -- examples/simple.why -o simple && ./simple

# Lambda expressions and higher-order functions  
cargo run --bin yc -- examples/lambda.why -o lambda && ./lambda

# Object-oriented programming with methods
cargo run --bin yc -- examples/foo.why -o foo && ./foo
```

## 🏗️ Architecture

Y is built using:
- **Rust** - Systems programming language for the compiler
- **LLVM** - Backend for optimized native code generation via Inkwell
- **rust-sitter** - Parser generation framework
- **Tower LSP** - Language Server Protocol implementation

### Compiler Pipeline
1. **Lexing/Parsing** → Abstract Syntax Tree (AST)
2. **Type Checking** → Type inference and validation  
3. **Code Generation** → LLVM IR generation
4. **Optimization** → LLVM optimization passes
5. **Linking** → Native executable output

## 🤝 Contributing

Y is an open-source project welcoming contributions! Current priority areas:

1. **CodeGen Implementation** - Fix remaining `todo!()` implementations
2. **Language Features** - Implement pattern matching, generics
3. **Tooling** - Enhance LSP server, add formatter
4. **Documentation** - Examples, tutorials, API docs
5. **Testing** - Expand test coverage, integration tests

See [ROADMAP.md](ROADMAP.md) for detailed development priorities.

## 📊 Project Stats

- **Lines of Code**: ~15,000 (Rust)
- **Test Coverage**: 183 parser tests, comprehensive type checking
- **LLVM IR Quality**: Functional and correct for supported features
- **Example Success Rate**: 70%+ of examples compile successfully

## ⚖️ License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- **LLVM Project** - Powerful compiler infrastructure
- **Rust Community** - Excellent tooling and ecosystem
- **Language Design Inspiration** - Rust, Haskell, TypeScript, Swift

---

**Note**: Y is a research and educational project exploring modern language design. While functional for basic programs, it's not yet ready for production use. Star the repo to follow development progress!