# Pre-Requisites

At the current stage, the language only exists as a "theoretical concept", since the only stable (and somewhat working) components are the lexer and the parser. Due to that, the only pre-requisites for "using" the language are a working machine, terminal access with Rust and Git installed and available. First, clone the repository:

```sh
git clone https://github.com/H1ghBre4k3r/pesca-lang.git
cd pesca-lang
```

After that, you can just build and use the executable:

```sh
cargo build --release
./target/release/pesca-lang <FILE_NAME>
```

This will print you the lexed tokens, as well as the parse AST.
