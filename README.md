# Y Lang

Why would anyone build a new (and rather experimental) language with no real world use case.

## Design

Y (pronounced as _why_) is based on the idea that everything is an expression and evaluates to a value. For example, `7` _and_ `3 + 4` directly evaluate to `7` (the latter one gets evaluated in the form of a binary expression), `3 > 5` to `false` and `"Hello"` evaluates to `"Hello"`.

Besides these "primitive" expressions, more complex expression, such as blocks (i.e., statements enclosed in `{` and `}`), if-statements and function (calls) also evaluate to a value. To simplify this approach, the value of last expression within one of these complex "structures" is automatically the value of this structure.

For example:

```why
{
    "Hello, World"
    3 + 4
}
```

evaluates to `7`. In this example, `"Hello, World"` is ignored, because the last expression does not depend on it. Another example:

```why
if 3 < 5 {
    "foo"
} else {
    "bar"
}
```

### Variables

To store the values of expressions, you are able to declare variables:

```why
let foo := "bar"
```

Variable definitions always start with the keyword `let` followed by an identifier (e.g., `foo`) and the "walrus operator" `:=` and the value. To assign a new value to a variable, you can use a similar pattern:

```why
foo = "another value"
```

Note that you do not use the `let` keyword nor `:=` in this case.

Following the idea of "everything evaluates to a value", you can "assign" complex structures (blocks, functions, function call, if statements, etc.) to a variable:

```
let foo := {
    let a := 16 // Yes, variable definitions also work in blocks
    a + 26
}

let some_variable := if foo > 30 {
    "foo"
} else {
    "bar"
}
```

### Type System

Y is strongly typed. Meaning, you can not assign a variable with a new value which differs its previous type. I.e, the following does not work:

```why
let foo := "bar"
foo = 42 // TypeError!
```

Due to that, if you assign an if statement to a variable, both blocks have to return a value of the same type:

```why
// works
let foo := if a > b {
    42
} else {
    1337
}

// TypeError!
let bar := if a > b {
    42
} else {
    "bar"
}
```

### Primitves

Y supports a couple of primitive types which are build directly into the lanuage:

- `int` for numbers (current 32 bit)
- `str` for string **constants**
- `bool` for boolean values
- `void` for "empty" values
- functions (see later for information on how to declare a function type)

More complex types are subject for futures features.

### Mutablity

Currently, Y only allows mutation of variables which are defined within the current scope (i.e., in the current block). You can still access variables defined in an outer scope (write-only):

```why
let foo := 42

if a > b {
    let bar := foo // works, because it is read-only
    bar = 1337
} else {
    foo = 1337 // TypeError!
}
```

### Functions

You can encapsulate behaviour in functions. Functions are (currently) the only place in Y where you need to explicitly annotate types (for parameters and return type):

```why
let add := (x : int, y : int) : int => {
    x + y
}
```

Function definitions work in a similar way like regular variable definitions, since functions are treated as first-class citizens in Y.

#### Function Type

If you want to declare a parameter of your function to be a function itself, you can do it like this:

```why
let foo := (bar : (int, int) -> int) : int => {
    bar(3, 4)
}
```

In this example, we declare a variable `foo` and assign it a function, which expects one parameter (in this case named `bar`) of type `(int, int) -> int`, meaning the provided function should accept two parameters of type `int` and produce/return a value of type `int`.

#### ⚠️ Known Limitations

Currently, you are not able to return functions from other functions or use values which are defined in an outer scope of a function. I am currently figuring out a way to achieve that.

## Pipeline

To turn a Y program into an executable (or interpret it), the compiler takes several steps.

### Parsing

As a first step, the parser tries to generate a more or less meaningfull AST from the given source code. While the parser relies on the grammar defined by `y-lang.pest`, the generated AST is a little more specific on the structure.

### Type Checker

In order to provide the security of strong types, the type checker checks the types of all expressions, variables and assignments. Furthermore, it checks if variables are defined in the currently available scope and if they are mutable (of needed).

### Interpreter & Compiler

As a last step, the generated AST either gets interpreted or compiled to assembly. This generated assembly get then compiled to an object file using NASM and then linked via `cc`.

## Usage

At the time of writing this, we do not provide binaries for Y. If you want to use or experiment with y, you can compile the toolchain yourself. For that you need rust and cargo installed on your system. If you want to actually compile a program, you also need `NASM` installed. This crate provides a binary called `why`.

You can use `why` to typecheck, interpret and compile your program:

```shell
why -f path/to/program.why # typechecking

why -f path/to/program.why -r # typecheck & run/interpret

why -f path/to/program.why -o path/to/output # typecheck and compile
```

## Operating Systems

Y is actively developed under macOS. I tested Linux to some point (and CI should test aswell), but I can not guarantee full compatibility.

## Contributing

> The code is the reincarnation of the mighty spaghetti monster. I had no real time to refactor anything or even write useful tests.

Even though I currently have no guide for contributing, feel free to open issues with feature requests. Be warned that I will probably not accept any PRs until I defined some guidelines for contributing or code/assembly style.
