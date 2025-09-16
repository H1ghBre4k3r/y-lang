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

Lambdas are anonymous functions that can be used inline or assigned to variables. They provide a concise way to create function values:

```why
// Anonymous lambda passed directly
let foo = takesFunction(\(x) => x * x);

// Lambda assigned to variable (type annotation required)
let bar: (i64, i64) -> i64 = \(x, y) => x + y;

// Multi-line lambda with block
let complex: (i64) -> i64 = \(x) => {
    let doubled = x * 2;
    doubled + 1
};
```

### Variable Capture in Lambdas

One of the most powerful features of lambdas is their ability to capture variables from their surrounding scope, creating **closures**:

```why
fn createAdder(x: i64): (i64) -> i64 {
    // This lambda captures the variable 'x'
    \(y) => x + y
}

fn main(): i64 {
    let addFive = createAdder(5);
    addFive(10)  // Returns 15
}
```

**Key points about variable capture:**
- Variables are captured **by value** at lambda creation time
- Captured values are copied into the lambda's environment
- Both simple variables and struct fields can be captured

```why
struct Point {
    x: i64;
    y: i64;
}

fn createTranslator(offset: Point): (Point) -> Point {
    \(p) => Point {
        x: p.x + offset.x,  // offset is captured
        y: p.y + offset.y
    }
}
```

For more details on closures and capture mechanics, see [Closures](./closures.md).

### Function Types

Functions introduce a new type category that can be used anywhere a type is expected:

Function types consist of the parameter types and return type: `(param_types...) -> return_type`

```why
// Function that takes a function parameter
fn takesFunction(f: (i64) -> i64): i64 {
    f(42)
}

// Function that returns a function
fn returnsFunction(): (i64, i64) -> i64 {
    \(x, y) => x * y
}

// Function variables
let myFunc: (i64) -> i64 = \(x) => x * 2;
let result = myFunc(5);  // result = 10
```

### Combining Functions and Lambdas

You can mix named functions and lambdas seamlessly:

```why
fn square(x: i64): i64 {
    x * x
}

fn main(): i64 {
    // Pass named function as value
    let squarer: (i64) -> i64 = square;

    // Create lambda that uses both
    let combo = \(x) => square(x) + x;

    combo(5)  // Returns 30 (25 + 5)
}
```
