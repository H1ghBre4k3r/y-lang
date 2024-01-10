# Datatypes

In Y, every value has a type associated with it. Some types are built into the language, some are user defined. Examples for built-in types are:

- numeric types (e.g. `u32`, `f64`, `i64`)
  - the letter denotes the type of number (e.g., `u` for unsigned, `i` for signed integers, and `f` for floating point numbers)
  - the number denotes the actual size of the underlying number _in bits_
- characters (`char`)
- string literals (`str`)
- boolean values (`bool`)

These basic types do only provide limited methods or functions to interact with them. However, you can perform certain arithmetic operations on them.

Y is able to infer the types of man variables. In some cases, however, you are required to explicitly declare the type of a variable:

```why
let foo: u32 = 42;
```
