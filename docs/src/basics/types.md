# Data Types

Y is a statically-typed language where every value has a type. The type system includes built-in primitive types and user-defined types like structs.

## Primitive Types

### Numeric Types

Y supports several numeric types with explicit bit-width specifications:

**Integer Types:**
- `i64` - 64-bit signed integer (most common)
- `u32` - 32-bit unsigned integer

**Floating Point Types:**
- `f64` - 64-bit floating point number

```why
let age: i64 = 25;
let count: u32 = 42;
let pi: f64 = 3.1415;
let price = 133.7;  // Type inferred as f64
```

### Character and String Types

- `char` - Single Unicode character
- `str` - String literal (immutable string slice)

```why
let letter = 'a';
let greeting = "Hello, World!";
let test_char = 'b';
let test_str = "test";
```

### Boolean Type

- `bool` - Boolean values (`true` or `false`)

```why
let is_ready = true;
let is_finished = false;
```

## Function Types

Y treats functions as first-class values with explicit type signatures:

```why
// Function type: takes two i64 parameters, returns i64
let add_func: (i64, i64) -> i64 = add;

// Lambda with function type
let identity: (i64) -> i64 = \(x) => x;

// Function taking a function as parameter
fn takes_function(func: (i64, i64) -> i64): i64 {
    func(42, 69)
}
```

## Array Types

Arrays are reference types denoted with `&[T]`:

```why
let numbers: &[i64] = &[1, 2, 3, 4, 5];
let empty_array: &[i64] = &[];
let chars = &['a', 'b', 'c'];  // Type: &[char]
```

## User-Defined Types

You can define custom types using structs:

```why
struct Person {
    name: str;
    age: i64;
}

let person = Person {
    name: "Alice",
    age: 30
};
```

## Type Inference

Y can automatically infer types in many cases:

```why
let x = 42;        // Inferred as i64
let y = 3.14;      // Inferred as f64
let name = "Bob";  // Inferred as str
let flag = true;   // Inferred as bool
```

## Explicit Type Annotations

When type inference isn't sufficient or for clarity, you can specify types explicitly:

```why
let foo: u32 = 42;
let process: (str) -> void = printf;
let numbers: &[i64] = &[];
```

## Constants

Constants are immutable values known at compile time:

```why
const PI: f64 = 3.1415;
const MAX_SIZE: i64 = 1000;
```

## Type Compatibility

Y has strict type checking. Different numeric types don't automatically convert:

```why
let x: i64 = 42;
let y: u32 = 100;
// let z = x + y;  // Error: type mismatch
```
