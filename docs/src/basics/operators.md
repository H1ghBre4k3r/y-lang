# Operators

Y provides a comprehensive set of operators for performing calculations, comparisons, and logical operations.

## Arithmetic Operators

### Basic Arithmetic

```why
let a = 10;
let b = 3;

let sum = a + b;        // Addition: 13
let difference = a - b; // Subtraction: 7
let product = a * b;    // Multiplication: 30
let quotient = a / b;   // Division: 3 (integer division)
let remainder = a % b;  // Modulo: 1
```

### Unary Operators

```why
let x = 5;
let negative = -x;  // Unary minus: -5

// Note: Unary plus (+x) is not currently supported
```

### Floating Point Arithmetic

```why
let x = 10.0;
let y = 3.0;

let result = x / y;  // Floating point division: 3.333...
```

## Comparison Operators

All comparison operators return boolean values:

```why
let a = 10;
let b = 5;

let equal = a == b;         // Equality: false
let not_equal = a != b;     // Inequality: true
let less = a < b;           // Less than: false
let less_equal = a <= b;    // Less than or equal: false
let greater = a > b;        // Greater than: true
let greater_equal = a >= b; // Greater than or equal: true
```

### Type Compatibility

Comparisons require compatible types:

```why
let x = 42;
let y = 42.0;

// This would be an error (comparing i64 with f64):
// let result = x == y;

// Convert to same type first:
let result = x == (y as i64);  // Explicit cast (if supported)
```

## Logical Operators

### Boolean Logic

```why
let a = true;
let b = false;

let and_result = a && b;  // Logical AND: false
let or_result = a || b;   // Logical OR: true
let not_result = !a;      // Logical NOT: false
```

### Short-Circuit Evaluation

Logical operators use short-circuit evaluation:

```why
fn expensive_check(): bool {
    // This might not be called if first condition is false
    true
}

let result = false && expensive_check();  // expensive_check() not called
let result2 = true || expensive_check();  // expensive_check() not called
```

## Assignment Operators

### Basic Assignment

```why
let mut x = 5;
x = 10;  // Simple assignment
```

### Compound Assignment (Future Feature)

Note: These may be added in future versions:

```why
// Potential future syntax:
// x += 5;   // Equivalent to: x = x + 5;
// x -= 3;   // Equivalent to: x = x - 3;
// x *= 2;   // Equivalent to: x = x * 2;
// x /= 4;   // Equivalent to: x = x / 4;
```

## Operator Precedence

Y follows standard mathematical precedence rules (highest to lowest):

1. **Unary operators**: `-`, `!`
2. **Multiplicative**: `*`, `/`, `%`
3. **Additive**: `+`, `-`
4. **Comparison**: `<`, `<=`, `>`, `>=`
5. **Equality**: `==`, `!=`
6. **Logical AND**: `&&`
7. **Logical OR**: `||`
8. **Assignment**: `=`

### Using Parentheses

Use parentheses to override default precedence:

```why
let result1 = 2 + 3 * 4;      // 14 (3 * 4 first, then + 2)
let result2 = (2 + 3) * 4;    // 20 (2 + 3 first, then * 4)

let condition = (x > 0) && (y < 10);  // Clear grouping
```

## Type-Specific Operators

### String Operations

```why
let name = "Alice";
let greeting = "Hello, " + name;  // String concatenation (if supported)

// String comparison
let is_same = name == "Alice";
```

### Array Indexing

```why
let numbers = &[1, 2, 3, 4, 5];
let first = numbers[0];   // Array access: 1
let third = numbers[2];   // Array access: 3
```

### Struct Field Access

```why
struct Point {
    x: i64;
    y: i64;
}

let point = Point { x: 10, y: 20 };
let x_coord = point.x;  // Field access: 10
```

### Function Call Operator

```why
fn add(a: i64, b: i64): i64 {
    a + b
}

let result = add(5, 3);  // Function call: 8

// Lambda call
let multiply = \(x, y) => x * y;
let product = multiply(4, 5);  // 20
```

## Advanced Operator Usage

### Chaining Comparisons

```why
let x = 5;
let in_range = (x >= 0) && (x <= 10);  // Check if x is in range [0, 10]
```

### Combining Logical Operations

```why
fn is_valid_age(age: i64): bool {
    (age >= 0) && (age <= 150)
}

fn can_vote(age: i64, is_citizen: bool): bool {
    (age >= 18) && is_citizen
}

let age = 25;
let citizen = true;
let eligible = is_valid_age(age) && can_vote(age, citizen);
```

### Expression-Based Operations

Y is expression-oriented, so operators can be used in various contexts:

```why
// In conditional expressions
let result = if (x > 0) { x * 2 } else { x };

// In function returns
fn absolute_value(x: i64): i64 {
    if x >= 0 { x } else { -x }
}

// In variable initialization
let max_value = if a > b { a } else { b };
```

## Common Patterns

### Mathematical Calculations

```why
// Distance formula
fn distance(x1: f64, y1: f64, x2: f64, y2: f64): f64 {
    let dx = x2 - x1;
    let dy = y2 - y1;
    sqrt(dx * dx + dy * dy)  // Assuming sqrt function exists
}
```

### Range Checking

```why
fn in_bounds(value: i64, min: i64, max: i64): bool {
    (value >= min) && (value <= max)
}
```

### Conditional Logic

```why
fn sign(x: i64): i64 {
    if x > 0 {
        1
    } else if x < 0 {
        -1
    } else {
        0
    }
}
```

### Accumulation

```why
fn sum_range(start: i64, end: i64): i64 {
    let mut total = 0;
    let mut i = start;

    while i <= end {
        total = total + i;
        i = i + 1;
    }

    total
}
```

## Best Practices

### 1. Use Parentheses for Clarity

```why
// Good: Clear intent
let result = (a * b) + (c * d);

// Less clear: Relies on precedence knowledge
let result = a * b + c * d;
```

### 2. Break Complex Expressions

```why
// Good: Readable steps
let width_ratio = screen_width / base_width;
let height_ratio = screen_height / base_height;
let scale_factor = if width_ratio < height_ratio { width_ratio } else { height_ratio };

// Less readable: Everything in one expression
let scale_factor = if (screen_width / base_width) < (screen_height / base_height) {
    screen_width / base_width
} else {
    screen_height / base_height
};
```

### 3. Consistent Comparison Ordering

```why
// Good: Consistent ordering (variable first)
if age >= 18 { ... }
if score > threshold { ... }

// Also acceptable but be consistent
if 18 <= age { ... }
if threshold < score { ... }
```

### 4. Use Logical Operators Effectively

```why
// Good: Clear logical flow
if is_valid_user && has_permission && !is_banned {
    allow_access();
}

// Good: Early return pattern
if !is_valid_user || !has_permission || is_banned {
    return false;
}
```

Operators are fundamental building blocks that enable Y programs to perform calculations, make decisions, and manipulate data effectively while maintaining type safety and clear semantics.