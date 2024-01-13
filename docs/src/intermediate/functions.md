# Functions

Functions are a way to group a set of instructions and give them a name. In Y, you have two different possibilities to work with functions: "Classic" Functions and Lambdas.

## Classic Functions

Classic functions are declared using the `fn` keyword.

```why
fn square(x: i64): i64 {
    return x * x;
}

let foo = square(42);
```

Functions can accept an arbitrary amount of arguments and return a value. Both, arguments and return value, have to be annotated with a type.

## Lambdas

Lambdas can be either used as anonymous functions or be assigned to a variable:

```why
let foo = takesFunction((x) => x * x);

let bar: (i32, i32) -> i32 = (x, y) = x + y;
```

When assigning them to a variable, you have to explicitly annotate the type of the lambda.

### Function Types

As you can see in the above example, using functions introduces a new type: Functions.

Function types consist of the list of arguments and the type of the return value. Using that, you can annotate every variable, parameter or even return type of a function to be a function:

```why
fn takesFunction(f: (i32) -> i32): i32 {
    return f(42)
}

fn returnsFunction(): (i32, i32) -> i32 {
    return (x, y) => x * y;
}
```
