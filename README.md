# Y Lang

![Crates.io](https://img.shields.io/crates/v/y-lang) [![CI Checks](https://github.com/H1ghBre4k3r/y-lang/actions/workflows/ci_checks.yml/badge.svg?branch=main)](https://github.com/H1ghBre4k3r/y-lang/actions/workflows/ci_checks.yml) ![Crates.io](https://img.shields.io/crates/l/y-lang) ![GitHub issues](https://img.shields.io/github/issues/H1ghBre4k3r/y-lang)

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

You can optionally explicitly end an expression with a semicolon:

```why
if 3 < 5 {
    "foo"
} else {
    "bar"
};
```

In some situations this is required, since Y would interpret the provided expressions in a different way. See section about functions.

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

-   `int` for numbers (currently 64 bit)
-   `char` for characters (8 bit values, therefore, small `ints` can be used)
-   `str` for string **constants**
-   `bool` for boolean values
-   `void` for "empty" values
-   functions (see later for information on how to declare a function type)

Furthermore, you can specify references as function parameters. References work like regular variables (or rather like their "underlying" variable), but they also effect their "source":

```why
// declare function with parameter of type integer-reference
let foo := (a: &int): void => {
    a = a * 2 // <- this assigns a new value to the underlying variable of `a`
}

let bar := 2

foo(bar) // pass `bar` as a parameter, which will automatically be converted to a reference
```

Currently, you can only pass identifiers as references.

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

### Control Flow

Y supports different types of control flow statements.

#### Loops

If you want to repeat instructions multiple times, you can bundle them in a loop. Currently, there is only one kind of loop: `while` loops, e.g.:

```why
let mut x := 0
while x < 5 {
    doSomething()
    x = x + 1
}
```

The head of the while loop must contain an expression which evaluates to a boolean value, while the body of the loop may contain anything. Therefore, a construct like this is valid Y:

```why
while {
    let foo := bar()
    foo < 5
} {
    doSomething()
}
```

**Note:** By default, loops in Y evaluate to the type `void`. Using the return value of a loop is, therefore, undefined behaviour.

### Functions

You can encapsulate behaviour in functions. Functions are (currently) the only place in Y where you need to explicitly annotate types (for parameters and return type):

```why
let add := (x : int, y : int) : int => {
    x + y
}
```

Function definitions work in a similar way like regular variable definitions, since functions are treated as first-class citizens in Y.

#### Call-Postfix

To call a function, you can postfix any expression which evaluates to a function (currently only identifiers) with `([param, [param, ...]])` to call it the given arguments.

This may lead to certain pitfalls, since Y is _not_ whitespace-sensitive! For example, the following will lead to a type error:

```why
let foo := (some_condition: bool) : int => {
    if some_condition {
        print("Condition is true!")
    }
    (3 + 4)
}
```

Here, `(3 + 4)` (although it is intended as the return expression of the function) is interpreted as a call to the result of the if expression. To prevent this, you have to explicitly terminate the if expression with a semicolon:

```why
let foo := (some_condition: bool) : int => {
    if some_condition {
        print("Condition is true!")
    }; // <- semicolon to terminate expression
    (3 + 4)
}
```

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

#### Arrays & Indexing

Y contains different ways of working with array-like structures: `TupleArray` and `ArraySlice`.

##### TupleArray

`TupleArray` is the standard array type from other languages. Length and type of the contained elements need to be known at compile time:

```why
// this creates an array of 10 integers, filled with all 0
let foo := [0; 10]
```

Symmetric to this, you can define a type for this:

```why
let bar := (some_array: [int; 10]): void => { ... }
```

Accessing an element in this array works by providing an index:

```why
// get the value at index 5 (i.e., the 6th position)
let a := some_array[5]

// the the value at index 3
some_array[3] = 42
```

##### ArraySlice

On the other hand, `ArraySlice` represents an array of undefined (or unknown) size. Therefore, you can not directly define one, but you can specify it as a type for a function parameter:

```why
let baz := (some_slice: &[int]): void => { ... }
```

Indexing works the same as for `TupleArray`.

**Note:** Y (at the point of writing this) does not perform any reliable bounds checks.

##### Indexing Strings

In Y, strings and arrays are (to some extend) convertible to one another. You can index strings the same way as arrays:

```why
let foo := "Hello, World!"
foo[2] = 'n'
print(foo) // "Henlo, World!"
```

##### Type Conversion

Some types are convertible into other. For example, a `TupleArray` can be converted to an `ArraySlice`, but not the other way around. `ArraySlice` and `TupleArray` of type `char` can be converted into `str` (**you** have to ensure that the last byte is `0`). And, last but not least, `str` can be converted to `ArraySlice` of type `char`.

### Modules

You can split your code up into modules. Modules are just other files ending with `.why` and can be imported by their name (without the respective file ending):

```why
import foo

foo::bar()
```

Here, we import a module named foo (from a file `foo.why`) and call an exported function (i.e., `bar`) by its "full name". By default, you have to specify the full resolved name to an imported function in the form of `module::function()`.

You can also import modules from other directories:

```why
import some::dir::foo

some::dir::foo::bar()
```

If you want to directly call a function without specifying the module name, you have to import the module as a wildcard:

```why
import foo::*

bar()
```

This can be useful when importing common functions from a utility module.

Imports are traversed recursively. So if you import module `foo`, which imports module `bar`, both modules are parsed, type checked and compiled. However, if you want to use module `bar` in your root module, you have to import it there aswell. To avoid double parsing and checking of modules, the loader keeps track of already loaded modules and just references them (if already present).

#### ⚠️ Non-Function-Exports

Please note that all non-function-members of a module (i.e., all other variables etc.) are **not** exported. They are completely "erased" from the program. Therefore, your exported functions are not allowed to use any other variables other than other exported functions.

In the future, we plan to add support for exporting constants, but until then be aware of this limitation.

### Declarations

If you want to declare a function (or a variable) which is already pre-defined (or comes from another source), you can do so via the `declare` keyword. A declaration consists of the name of the variable to declare and a corresponding type annotation. E.g.:

```why
declare print : (str) -> void
```

### Builtins

Currently, Y provides a single builtin function: `syscall_4` (for calling syscalls with 4 arguments). To use it, you have to declare it somewhere in your program:

```why
declare syscall_4 : (int, any, any, any) -> any
```

Note: The first parameter is the identifier for this syscall.

If you want to have an overview of currentl available syscall abstractions, have a look at `std.why` in the examples folder.

### Compiler Directives

Y support (more or less) conditional compilation depending on the current operating system. To declare something is "OS"-dependant, you have to annotate it accordingly:

```why
#[os == "linux"]
let value := "We are on linux"

#[os == "macos"]
let value := "We are on macOS"
```

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
why path/to/program.why # typechecking

why path/to/program.why -r # typecheck & run/interpret

why path/to/program.why -o path/to/output # typecheck and compile
```

## Operating Systems

Y is actively developed under macOS. I tested Linux to some point (and CI should test aswell), but I can not guarantee full compatibility.

## Contributing

> The code is the reincarnation of the mighty spaghetti monster. I had no real time to refactor anything or even write useful tests.

Even though I currently have no guide for contributing, feel free to open issues with feature requests. Be warned that I will probably not accept any PRs until I defined some guidelines for contributing or code/assembly style.
