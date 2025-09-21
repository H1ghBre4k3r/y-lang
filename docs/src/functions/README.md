# Functions and Methods

Functions are fundamental building blocks in Y that allow you to organize code into reusable, testable units. Y treats functions as first-class values, supporting both named functions and anonymous lambda expressions.

## Overview

Y supports several types of callable constructs:

- **Named Functions** - Traditional function declarations with explicit names
- **Lambda Expressions** - Anonymous functions that can be assigned to variables or passed as arguments
- **Instance Methods** - Functions associated with specific types through instance blocks
- **External Declarations** - Declarations for functions implemented outside Y (like C functions)

## Function Features

Y functions support:
- **Explicit type signatures** with parameter and return types
- **First-class values** - functions can be stored in variables and passed as arguments
- **Expression-oriented** - functions can end with expressions instead of explicit returns
- **Type inference** in many contexts
- **Higher-order functions** - functions that take or return other functions

## Basic Function Syntax

```why
fn function_name(param1: Type1, param2: Type2): ReturnType {
    // function body
    expression_or_return
}
```

## Lambda Syntax

```why
let lambda_var = \(param1, param2) => expression;
```

## Method Syntax

```why
instance TypeName {
    fn method_name(param: Type): ReturnType {
        // method body
    }
}
```

## Example Usage

```why
// Named function
fn add(x: i64, y: i64): i64 {
    x + y
}

// Lambda function
let multiply = \(x, y) => x * y;

// Function as struct field
struct Calculator {
    operation: (i64, i64) -> i64;
}

let calc = Calculator {
    operation: add
};

// Higher-order function
fn apply_twice(func: (i64) -> i64, value: i64): i64 {
    func(func(value))
}
```

The following sections provide detailed information about each type of function and method construct in Y.