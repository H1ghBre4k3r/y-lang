# Variables and Constants

Y provides two main ways to store values: variables (which can change) and constants (which cannot change after initialization).

## Variables

Variables in Y are declared using the `let` keyword. By default, variables are **immutable**, meaning their value cannot be changed after initialization.

### Immutable Variables

```why
let x = 42;
let name = "Alice";
let is_ready = true;
```

### Mutable Variables

To create a variable that can be modified, use the `mut` keyword:

```why
let mut counter = 0;
counter = counter + 1;  // This is allowed

let mut items = &[1, 2, 3];
items[0] = 10;  // This is allowed
```

### Type Annotations

Y has powerful type inference, but you can explicitly specify types when needed:

```why
// Type inferred from value
let x = 42;  // x is i64

// Explicit type annotation
let y: i64 = 42;
let z: f64 = 3.14;

// Required for complex types or when type cannot be inferred
let numbers: &[i64] = &[];
let callback: (i64) -> i64 = \(x) => x * 2;
```

### Initialization

Variables must be initialized when declared - Y doesn't allow uninitialized variables:

```why
let x: i64;  // Error: variable must be initialized
let y = 42;  // Correct
```

## Constants

Constants are compile-time values that never change. They're declared with the `const` keyword and must have explicit type annotations:

```why
const PI: f64 = 3.14159;
const MAX_SIZE: i64 = 1000;
const GREETING: str = "Hello, World!";
```

### Constant Rules

1. **Must have type annotations**: `const VALUE: i64 = 42;`
2. **Must be compile-time constants**: Cannot depend on runtime values
3. **Immutable by nature**: Cannot be reassigned
4. **Global scope**: Can be accessed from anywhere in the program

```why
const BUFFER_SIZE: i64 = 1024;

fn process_data() {
    let buffer: &[i64] = &[0; BUFFER_SIZE];  // Using constant
    // ... process data
}
```

## Scoping

Variables follow lexical scoping rules:

```why
fn main(): i64 {
    let x = 10;

    {
        let y = 20;
        let x = 30;  // Shadows outer x
        // x is 30, y is 20 here
    }

    // x is 10 here, y is not accessible
    x
}
```

### Variable Shadowing

Y allows variable shadowing, where a new variable with the same name hides the previous one:

```why
let x = 5;
let x = x + 1;  // New variable x with value 6
let x = "hello";  // New variable x with different type
```

This is different from reassignment - each `let` creates a new variable.

## Assignment vs Initialization

There's an important distinction between initialization and assignment:

```why
// Initialization (creates new variable)
let x = 42;

// Assignment (changes existing mutable variable)
let mut y = 10;
y = 20;  // Assignment

// This would be an error:
let z = 5;
z = 10;  // Error: cannot assign to immutable variable
```

## Working with Complex Types

### Struct Fields

```why
struct Point {
    x: i64;
    y: i64;
}

let mut point = Point { x: 0, y: 0 };
point.x = 10;  // Modifying field of mutable struct
```

### Array Elements

```why
let mut numbers = &[1, 2, 3, 4, 5];
numbers[0] = 10;  // Modifying array element
```

### Function Variables

Functions can be stored in variables:

```why
fn add(a: i64, b: i64): i64 {
    a + b
}

let operation: (i64, i64) -> i64 = add;
let result = operation(5, 3);  // result is 8
```

## Memory Considerations

### Stack vs Heap

- **Simple values** (numbers, booleans, characters) are stored on the stack
- **Complex values** (arrays, strings, closures with captures) may use heap allocation
- **References** point to data that may be on stack or heap

```why
let x = 42;           // Stack allocated
let arr = &[1, 2, 3]; // Array data may be heap allocated
let closure = \(y) => x + y;  // Captures x, may use heap for environment
```

## Best Practices

### 1. Prefer Immutability

Use immutable variables by default, only add `mut` when you need to modify the value:

```why
// Good
let total = calculate_sum(&numbers);

// Less ideal (unless you need to modify total)
let mut total = calculate_sum(&numbers);
```

### 2. Use Descriptive Names

```why
// Good
let user_count = get_active_users();
let is_valid = validate_input(data);

// Less clear
let x = get_active_users();
let flag = validate_input(data);
```

### 3. Group Related Constants

```why
// Good organization
const DEFAULT_WIDTH: i64 = 800;
const DEFAULT_HEIGHT: i64 = 600;
const DEFAULT_TITLE: str = "Y Application";
```

### 4. Minimize Scope

Declare variables as close to their usage as possible:

```why
fn process_data(input: &[i64]): i64 {
    // Process input first
    let filtered = filter_data(input);

    // Then declare variables for final calculation
    let sum = calculate_sum(&filtered);
    let average = sum / filtered.len();

    average
}
```

## Common Patterns

### Conditional Initialization

```why
let value = if condition {
    expensive_calculation()
} else {
    default_value()
};
```

### Loop Counters

```why
let mut i = 0;
while i < 10 {
    process_item(i);
    i = i + 1;
}
```

### Accumulation

```why
let mut total = 0;
let mut i = 0;
while i < numbers.len() {
    total = total + numbers[i];
    i = i + 1;
}
```

### Builder Pattern

```why
let mut config = Config::new();
config.set_width(800);
config.set_height(600);
config.set_title("My App");
```

This foundation of variables and constants enables all other language features in Y, providing the building blocks for more complex programs while maintaining memory safety and clear semantics.