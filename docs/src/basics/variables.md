# Variables

Variables are fundamental building blocks in Y programs. They store values during program execution. Y emphasizes immutability by default, requiring explicit declaration when mutability is needed.

## Variable Declaration

The basic syntax for declaring a variable uses the `let` keyword:

```why
let x = 12;
let a = baz(x);
let test_char = 'a';
let test_str = "test";
```

A variable declaration consists of:
- The `let` keyword
- An identifier (variable name)
- The `=` assignment operator
- An expression that provides the initial value

## Type Annotations

Y can infer types automatically, but you can also explicitly specify the type:

```why
let foo: u32 = 42;
let x: (i64) -> i64 = \(x) => x;  // Function type
let arr: &[i64] = &[];            // Array reference type
```

## Mutability

Variables are **immutable by default**. Once assigned, their value cannot be changed:

```why
let foo = 42;
foo = 1337; // Error: cannot assign to immutable variable
```

To create a mutable variable, use the `mut` keyword:

```why
let mut foo = 42;
foo = 1337;  // Valid: foo is mutable

let mut i = 0;
while (i < 10) {
    i = i + 1;  // Mutating i in a loop
}
```

## Practical Examples

From real Y programs:

```why
fn main(): i64 {
    // Immutable variables
    let x = 12;
    let a = baz(x);

    // Mutable array
    let mut arr = &[42, 1337];
    arr[0] = 100;  // Modifying array contents

    // Mutable struct
    let mut my_struct = TestStruct {
        x: 42,
        bar: add
    };
    my_struct.x = 100;  // Modifying struct field

    return x + a;
}
```

## Best Practices

- Prefer immutable variables when possible - they prevent accidental mutations and make code easier to reason about
- Use `mut` only when you actually need to modify the variable's value
- Choose descriptive variable names that clearly indicate their purpose
