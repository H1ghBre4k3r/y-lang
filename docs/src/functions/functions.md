# Functions

Functions in Y allow you to encapsulate reusable code with explicit type signatures. They are first-class values that can be stored in variables, passed as arguments, and returned from other functions.

## Function Declaration

The basic syntax for declaring a function:

```why
fn function_name(parameter: Type): ReturnType {
    // function body
}
```

### Simple Functions

```why
fn greet(): void {
    printf("Hello, World!\n");
}

fn add(x: i64, y: i64): i64 {
    x + y
}

fn get_answer(): i64 {
    42
}
```

## Function Parameters

Functions can accept multiple parameters with explicit types:

```why
fn calculate_area(width: f64, height: f64): f64 {
    width * height
}

fn format_name(first: str, last: str): str {
    // String concatenation would be here if supported
    first  // For now, just return first name
}
```

## Return Types

Functions must specify their return type:

```why
fn divide(a: i64, b: i64): i64 {
    a / b
}

fn is_positive(x: i64): bool {
    x > 0
}

fn do_nothing(): void {
    // No return value
}
```

## Return Statements

Functions can use explicit `return` statements or end with an expression:

```why
// Explicit return
fn explicit_return_add(x: i64, y: i64): i64 {
    return x + y;
}

// Expression return (no semicolon on last line)
fn add(x: i64, y: i64): i64 {
    x + y
}

// Mixed approach
fn baz(x: i64): i64 {
    let intermediate = x * 2;
    return intermediate;
}
```

## Functions as First-Class Values

Functions can be stored in variables and passed around:

```why
fn add(x: i64, y: i64): i64 {
    x + y
}

fn multiply(x: i64, y: i64): i64 {
    x * y
}

// Store function in variable
let operation: (i64, i64) -> i64 = add;

// Use the function variable
let result = operation(10, 20);  // 30

// Function as struct field
struct Calculator {
    op: (i64, i64) -> i64;
    name: str;
}

let calc = Calculator {
    op: multiply,
    name: "Multiplier"
};
```

## Higher-Order Functions

Functions can take other functions as parameters:

```why
fn takes_function(func: (i64, i64) -> i64): i64 {
    func(42, 69)
}

fn apply_operation(x: i64, y: i64, op: (i64, i64) -> i64): i64 {
    op(x, y)
}

// Usage
let result1 = takes_function(add);       // 111
let result2 = apply_operation(10, 5, multiply);  // 50
```

## Function Examples from Y Code

### Basic Function Usage

```why
declare printf: (str) -> void;

fn baz(x: i64): i64 {
    let intermediate = x * 2;
    return intermediate;
}

fn main(): i64 {
    printf("Foo\n");
    let x = 12;
    let a = baz(x);
    return x + a;
}
```

### Functions Returning Functions

```why
fn foobar(): (i64) -> i64 {
    return \(x) => x;  // Returns a lambda
}

fn create_adder(base: i64): (i64) -> i64 {
    return \(x) => x + base;
}
```

### Functions with Complex Parameters

```why
fn process_struct(data: TestStruct): i64 {
    return data.x;
}

fn takes_array(arr: &[i64]): i64 {
    arr[0]
}
```

## Function Patterns

### Factory Functions

```why
fn create_point(x: f64, y: f64): Point {
    Point { x: x, y: y }
}

fn create_default_config(): Config {
    Config {
        debug: false,
        port: 8080,
        timeout: 30
    }
}
```

### Utility Functions

```why
fn max(a: i64, b: i64): i64 {
    if (a > b) { a } else { b }
}

fn clamp(value: i64, min: i64, max: i64): i64 {
    if (value < min) {
        min
    } else if (value > max) {
        max
    } else {
        value
    }
}
```

### Recursive Functions

```why
fn factorial(n: i64): i64 {
    if (n <= 1) {
        1
    } else {
        n * factorial(n - 1)
    }
}

fn fibonacci(n: i64): i64 {
    if (n <= 1) {
        n
    } else {
        fibonacci(n - 1) + fibonacci(n - 2)
    }
}
```

## Function Type Signatures

Function types use the `(param_types) -> return_type` syntax:

```why
// Function that takes no parameters and returns i64
let getter: () -> i64 = get_answer;

// Function that takes two i64s and returns i64
let binary_op: (i64, i64) -> i64 = add;

// Function that takes a function and returns i64
let higher_order: ((i64) -> i64) -> i64 = some_function;

// Complex function type
let complex: (str, &[i64], (i64) -> bool) -> &[str] = process_data;
```

## Main Function

Every Y program must have a main function:

```why
fn main(): i64 {
    // Program entry point
    // Return 0 for success, non-zero for error
    return 0;
}

// Or with void return
fn main(): void {
    // Program logic here
}
```

## External Function Declarations

You can declare functions implemented externally (like C functions):

```why
declare printf: (str) -> void;
declare malloc: (i64) -> void;
declare strlen: (str) -> i64;

// Usage
fn main(): void {
    printf("Hello from Y!\n");
}
```

## Best Practices

### Function Naming

```why
// Good: Clear, descriptive names
fn calculate_total_price(items: &[Item]): f64 { ... }
fn is_valid_email(email: str): bool { ... }
fn format_currency(amount: f64): str { ... }

// Less ideal: Unclear names
fn calc(x: &[Item]): f64 { ... }
fn check(s: str): bool { ... }
fn fmt(n: f64): str { ... }
```

### Function Size

Keep functions focused on a single responsibility:

```why
// Good: Single responsibility
fn validate_age(age: i64): bool {
    age >= 0 && age <= 150
}

fn calculate_tax(amount: f64, rate: f64): f64 {
    amount * rate
}

// Better than one large function handling everything
fn process_order(order: Order): OrderResult {
    validate_order(order);
    calculate_total(order);
    apply_discounts(order);
    finalize_order(order)
}
```

### Type Annotations

Always provide explicit type annotations for function parameters and return types:

```why
// Good: Clear types
fn process_data(input: &[i64], threshold: i64): &[i64] { ... }

// Required: Y needs explicit function signatures
fn calculate(x: i64, y: i64): i64 { x + y }
```

### Error Handling

Design functions to handle edge cases:

```why
fn safe_divide(a: i64, b: i64): i64 {
    if (b == 0) {
        return 0;  // Or handle error appropriately
    } else {
        return a / b;
    }
}

fn get_array_element(arr: &[i64], index: i64): i64 {
    // Bounds checking would go here
    return arr[index];
}
```