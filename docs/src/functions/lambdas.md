# Lambda Expressions

Lambda expressions in Y are anonymous functions that can be created inline. They provide a concise way to define functions without explicit names, especially useful for short operations and higher-order function programming.

## Lambda Syntax

The basic lambda syntax uses the `\` character followed by parameters and a body:

```why
\(param1, param2) => expression
```

### Simple Lambdas

```why
// Identity function
let identity = \(x) => x;

// Simple arithmetic
let add_one = \(x) => x + 1;
let multiply = \(x, y) => x * y;

// Boolean operations
let is_even = \(n) => n % 2 == 0;
```

## Lambda Types

Lambdas have function types that can be explicitly specified:

```why
let doubler: (i64) -> i64 = \(x) => x * 2;
let comparator: (i64, i64) -> bool = \(a, b) => a > b;
let processor: (str) -> void = \(s) => printf(s);
```

## Using Lambdas

### As Variables

```why
let square = \(n) => n * n;
let result = square(5);  // 25

let max = \(a, b) => if (a > b) { a } else { b };
let bigger = max(10, 20);  // 20
```

### As Function Arguments

```why
fn apply_to_number(n: i64, func: (i64) -> i64): i64 {
    func(n)
}

// Using lambda directly as argument
let result = apply_to_number(10, \(x) => x * 3);  // 30

fn apply_operation(x: i64, y: i64, op: (i64, i64) -> i64): i64 {
    op(x, y)
}

// Lambda for custom operations
let sum = apply_operation(5, 7, \(a, b) => a + b);      // 12
let product = apply_operation(3, 4, \(a, b) => a * b);  // 12
```

## Lambdas in Structs

Lambdas can be stored in struct fields:

```why
struct Processor {
    transform: (i64) -> i64;
    name: str;
}

let doubler_proc = Processor {
    transform: \(x) => x * 2,
    name: "Doubler"
};

let result = doubler_proc.transform(21);  // 42
```

## Examples from Y Code

### Basic Lambda Usage

```why
fn main(): i64 {
    // Lambda with explicit type annotation
    let x: (i64) -> i64 = \(x) => x;

    // Using the lambda
    let result = x(42);

    return result;
}
```

### Lambdas as Return Values

```why
fn foobar(): (i64) -> i64 {
    return \(x) => x;  // Return lambda expression
}

fn create_multiplier(factor: i64): (i64) -> i64 {
    return \(x) => x * factor;
}

// Usage
let times_three = create_multiplier(3);
let result = times_three(10);  // 30
```

### Lambdas with Complex Logic

```why
// Multi-statement lambdas (using blocks)
let complex_processor = \(x) => {
    let doubled = x * 2;
    let incremented = doubled + 1;
    incremented
};

// Conditional lambdas
let abs_value = \(x) => if (x < 0) { -x } else { x };

// Lambda with multiple parameters
let distance = \(x1, y1, x2, y2) => {
    let dx = x2 - x1;
    let dy = y2 - y1;
    sqrt(dx * dx + dy * dy)
};
```

## Higher-Order Programming

### Function Composition

```why
fn compose(f: (i64) -> i64, g: (i64) -> i64): (i64) -> i64 {
    return \(x) => f(g(x));
}

let add_one = \(x) => x + 1;
let double = \(x) => x * 2;

let add_then_double = compose(double, add_one);
let result = add_then_double(5);  // (5 + 1) * 2 = 12
```

### Array Processing (Conceptual)

```why
// If Y had higher-order array functions:
fn map(arr: &[i64], func: (i64) -> i64): &[i64] {
    // Implementation would go here
    return arr;  // Placeholder
}

// Usage with lambdas
let numbers = &[1, 2, 3, 4, 5];
let doubled = map(numbers, \(x) => x * 2);  // [2, 4, 6, 8, 10]
let squared = map(numbers, \(x) => x * x);  // [1, 4, 9, 16, 25]
```

### Event Handlers (Conceptual)

```why
struct Button {
    label: str;
    on_click: () -> void;
}

let save_button = Button {
    label: "Save",
    on_click: \() => printf("Saving data...\n")
};

let cancel_button = Button {
    label: "Cancel",
    on_click: \() => printf("Operation cancelled\n")
};
```

## Practical Examples

### Mathematical Operations

```why
struct MathOperations {
    sin: (f64) -> f64;
    cos: (f64) -> f64;
    square: (f64) -> f64;
}

let math = MathOperations {
    sin: \(x) => external_sin(x),  // Assuming external function
    cos: \(x) => external_cos(x),
    square: \(x) => x * x
};
```

### Data Transformation

```why
fn transform_data(value: i64, transformer: (i64) -> i64): i64 {
    transformer(value)
}

// Different transformations
let normalized = transform_data(150, \(x) => x / 100);      // 1
let clamped = transform_data(150, \(x) => if (x > 100) { 100 } else { x });  // 100
let negated = transform_data(150, \(x) => -x);              // -150
```

### Configuration and Callbacks

```why
struct Config {
    validator: (str) -> bool;
    formatter: (str) -> str;
    processor: (str) -> void;
}

let email_config = Config {
    validator: \(email) => contains(email, "@"),  // Conceptual
    formatter: \(email) => to_lowercase(email),   // Conceptual
    processor: \(email) => send_email(email)      // Conceptual
};
```

## Lambda Limitations

1. **Single Expression**: Lambdas work best with single expressions (though blocks can be used)
2. **Type Inference**: May need explicit type annotations in some contexts
3. **Closure Scope**: Currently limited closure capabilities

## Best Practices

### When to Use Lambdas

```why
// Good: Short, simple operations
let double = \(x) => x * 2;
let is_positive = \(x) => x > 0;

// Good: One-time use in function calls
process_array(data, \(x) => x + 1);

// Consider named function: Complex logic
fn complex_validation(input: str): bool {
    // Multiple checks and logic
    return true;  // Placeholder
}
```

### Readability

```why
// Good: Clear and concise
let operations = &[
    \(x) => x + 1,
    \(x) => x * 2,
    \(x) => x - 1
];

// Less readable: Too complex for lambda
let complex = \(x) => {
    let temp = x * 2;
    let adjusted = temp + 10;
    if (adjusted > 100) { 100 } else { adjusted }
};

// Better as named function:
fn complex_transform(x: i64): i64 {
    let temp = x * 2;
    let adjusted = temp + 10;
    if (adjusted > 100) { 100 } else { adjusted }
}
```

### Type Clarity

```why
// Good: Explicit types when needed
let processor: (str) -> bool = \(s) => validate(s);

// Good: Clear parameter names
let distance_calc = \(x1, y1, x2, y2) =>
    sqrt((x2 - x1) * (x2 - x1) + (y2 - y1) * (y2 - y1));

// Less clear: Unclear parameters
let calc = \(a, b, c, d) => sqrt((c - a) * (c - a) + (d - b) * (d - b));
```