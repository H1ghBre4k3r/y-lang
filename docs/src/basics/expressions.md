# Expressions

Y is an expression-oriented language, meaning that most constructs evaluate to a value. Understanding expressions is fundamental to writing effective Y code.

## What is an Expression?

An expression is a piece of code that evaluates to a value. In Y, almost everything is an expression, including:

- Literals (numbers, strings, booleans)
- Variable references
- Function calls
- Arithmetic operations
- Conditionals (if-else)
- Blocks
- Lambdas

## Basic Expressions

### Literal Expressions

```why
42          // Integer literal
3.14        // Floating point literal
"hello"     // String literal
'a'         // Character literal
true        // Boolean literal
false       // Boolean literal
```

### Variable Expressions

```why
let x = 42;
let y = x;  // x is an expression that evaluates to 42
```

### Arithmetic Expressions

```why
let a = 10;
let b = 5;

let sum = a + b;        // Addition expression
let product = a * 2;    // Multiplication expression
let complex = (a + b) * 2 - 1;  // Complex arithmetic expression
```

## Function Call Expressions

Function calls are expressions that evaluate to the return value:

```why
fn add(x: i64, y: i64): i64 {
    x + y  // This is also an expression (the last one in the function)
}

let result = add(10, 20);  // Function call expression
let nested = add(add(1, 2), add(3, 4));  // Nested function calls
```

## Conditional Expressions

If-else constructs are expressions in Y:

```why
let max = if (a > b) {
    a
} else {
    b
};

// Can be used directly in other expressions
let result = if (x > 0) { x } else { -x } + 10;
```

## Block Expressions

Blocks evaluate to the value of their last expression:

```why
let result = {
    let x = 10;
    let y = 20;
    x + y  // This value becomes the result of the block
};  // result is 30
```

## Lambda Expressions

Lambdas are expressions that create anonymous functions:

```why
let add_one = \(x) => x + 1;
let multiply = \(x, y) => x * y;

// Using lambda expressions directly
let numbers = &[1, 2, 3];
let transformed = map(numbers, \(x) => x * 2);
```

## Array Expressions

Array literals are expressions:

```why
let numbers = &[1, 2, 3, 4, 5];
let empty = &[];
let mixed = &[add(1, 2), multiply(3, 4), 42];
```

## Struct Initialization Expressions

Creating structs is also an expression:

```why
struct Point {
    x: i64;
    y: i64;
}

let origin = Point { x: 0, y: 0 };
let point = Point {
    x: calculate_x(),
    y: calculate_y()
};
```

## Property Access Expressions

Accessing struct fields and calling methods:

```why
let person = Person { name: "Alice", age: 25 };
let name = person.name;        // Property access expression
let id = person.get_id();      // Method call expression
```

## Practical Examples

### Expression-Heavy Function

```why
fn calculate_distance(x1: f64, y1: f64, x2: f64, y2: f64): f64 {
    // Everything here is composed of expressions
    let dx = x2 - x1;
    let dy = y2 - y1;
    sqrt(dx * dx + dy * dy)  // Final expression becomes return value
}
```

### Chaining Expressions

```why
fn main(): i64 {
    let my_struct = TestStruct {
        x: 42,
        bar: add
    };

    // Chaining property access and function call expressions
    let result = my_struct.bar(10, 20);

    // Complex expression with multiple parts
    let final_result = if (result > 0) {
        result + my_struct.x
    } else {
        my_struct.x * 2
    };

    return final_result;
}
```

### Expression Composition

```why
fn process_data(): i64 {
    let data = &[1, 2, 3, 4, 5];

    // Composed expression using array access, function calls, and arithmetic
    let result = process_value(data[0]) +
                 process_value(data[1]) * 2 +
                 if (data[2] > 3) { data[2] } else { 0 };

    return result;
}
```

## Statement vs Expression

While most things in Y are expressions, some constructs are statements:

```why
// Statements (don't evaluate to values):
let x = 42;              // Variable declaration
x = 100;                 // Assignment
return x;                // Return statement

// Expressions (evaluate to values):
x + y                    // Arithmetic
if (x > 0) { x } else { -x }  // Conditional
{                        // Block
    let temp = x + 1;
    temp * 2
}
```

## Best Practices

1. **Leverage expression-oriented style**: Use the fact that if-else and blocks are expressions
2. **Keep expressions readable**: Break complex expressions into intermediate variables when needed
3. **Use the last expression in functions**: Instead of explicit `return`, let the last expression be the return value
4. **Compose expressions thoughtfully**: Balance conciseness with clarity

```why
// Good: Clear and expressive
fn clamp(value: i64, min: i64, max: i64): i64 {
    if (value < min) {
        min
    } else if (value > max) {
        max
    } else {
        value
    }
}

// Also good: Breaking down complex logic
fn complex_calculation(input: i64): i64 {
    let base = input * 2;
    let adjusted = base + 10;
    if (adjusted > 100) { 100 } else { adjusted }
}
```