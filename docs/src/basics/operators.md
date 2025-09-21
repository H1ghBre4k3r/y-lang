# Operators

Y provides a comprehensive set of operators for various operations including arithmetic, comparison, and assignment.

## Arithmetic Operators

Y supports standard arithmetic operations on numeric types:

```why
let a = 10;
let b = 3;

let sum = a + b;        // Addition: 13
let difference = a - b; // Subtraction: 7
let product = a * b;    // Multiplication: 30
let quotient = a / b;   // Division: 3 (integer division)

// Works with floating point too
let x = 10.5;
let y = 2.0;
let result = x / y;     // 5.25
```

## Comparison Operators

All comparison operators return boolean values:

```why
let x = 42;
let y = 69;

let equal = x == y;           // false
let not_equal = x != y;       // true
let less_than = x < y;        // true
let greater_than = x > y;     // false
let less_equal = x <= y;      // true
let greater_equal = x >= y;   // false
```

## Assignment Operators

The basic assignment operator is `=`:

```why
let mut counter = 0;
counter = 5;        // Simple assignment

let mut arr = &[1, 2, 3];
arr[0] = 100;       // Array element assignment

let mut person = Person { name: "Alice", age: 25 };
person.age = 26;    // Struct field assignment
```

## Operator Precedence

Operators have the following precedence (highest to lowest):

1. **Postfix operators** (function calls, array indexing, property access)
2. **Prefix operators** (unary minus, etc.)
3. **Multiplication and Division** (`*`, `/`)
4. **Addition and Subtraction** (`+`, `-`)
5. **Comparison operators** (`==`, `!=`, `<`, `>`, `<=`, `>=`)

```why
let result = 2 + 3 * 4;     // 14, not 20 (multiplication first)
let comparison = 5 < 3 + 4; // true (addition first, then comparison)
```

## Using Operators with Different Types

### Numeric Operations

```why
let int_result = 42 + 17;        // i64 + i64
let float_result = 3.14 + 2.86;  // f64 + f64
```

### String and Character Operations

Currently, Y doesn't support string concatenation with `+`, but you can work with individual characters:

```why
let ch1 = 'a';
let ch2 = 'b';
let text = "Hello";
```

### Boolean Operations

```why
let flag1 = true;
let flag2 = false;
let x = 10;

// Using comparison results
let result = x > 5;  // true
if (result) {
    // do something
}
```

## Practical Examples

### Mathematical Calculations

```why
fn calculate_area(radius: f64): f64 {
    const PI: f64 = 3.1415;
    return PI * radius * radius;
}

fn baz(x: i64): i64 {
    let intermediate = x * 2;
    return intermediate;
}
```

### Conditional Logic

```why
fn main(): i64 {
    let x = 12;
    let y = 24;

    if (x < y) {
        return x + y;
    } else {
        return x - y;
    }
}
```

### Loop Counters

```why
fn count_to_ten(): void {
    let mut i = 0;
    while (i < 10) {
        i = i + 1;  // Using arithmetic and assignment
    }
}
```

## Operator Overloading

Y supports operator overloading through instance methods (though this is an advanced feature):

```why
instance i64 {
    declare add(i64): i64;  // Custom addition behavior
}
```