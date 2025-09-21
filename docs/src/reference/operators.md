# Operator Precedence Reference

This page provides the complete operator precedence and associativity rules for Y. Understanding these rules is crucial for writing correct expressions and knowing when parentheses are needed.

## Precedence Levels

Y operators are organized into precedence levels, with higher numbers indicating higher precedence (evaluated first).

| Precedence | Operators | Associativity | Description |
|------------|-----------|---------------|-------------|
| 5 | `()` `[]` `.` | Left | Postfix operators |
| 4 | `-` (unary) | Right | Prefix operators |
| 3 | `*` `/` | Left | Multiplicative |
| 2 | `+` `-` | Left | Additive |
| 1 | `==` `!=` `<` `>` `<=` `>=` | Left | Comparison |
| 0 | `=` | Right | Assignment |

## Operator Details

### Postfix Operators (Precedence 5)

These operators have the highest precedence and bind most tightly:

```why
// Function calls
function_name(arg1, arg2)
object.method(parameter)

// Array indexing
array[index]
matrix[row][column]

// Property access
struct_instance.field
object.method

// Chaining postfix operators
user.get_address().get_street()[0]
```

**Examples:**
```why
let result = calculate(x, y).format().length();  // ((calculate(x, y)).format()).length()
let value = data[i].process(flag);               // (data[i]).process(flag)
```

### Prefix Operators (Precedence 4)

Currently limited to unary minus:

```why
-expression    // Unary negation
```

**Examples:**
```why
let negative = -42;           // Unary minus
let result = -function_call(); // -(function_call())
let value = -array[0];        // -(array[0])
```

### Multiplicative Operators (Precedence 3)

```why
expression * expression    // Multiplication
expression / expression    // Division
```

**Left-associative:** `a * b * c` equals `(a * b) * c`

**Examples:**
```why
let area = width * height;
let rate = distance / time;
let complex = a * b / c * d;  // ((a * b) / c) * d
```

### Additive Operators (Precedence 2)

```why
expression + expression    // Addition
expression - expression    // Subtraction
```

**Left-associative:** `a + b + c` equals `(a + b) + c`

**Examples:**
```why
let total = base + tax + tip;     // (base + tax) + tip
let difference = end - start;
let calculation = a + b - c + d;  // ((a + b) - c) + d
```

### Comparison Operators (Precedence 1)

```why
expression == expression   // Equality
expression != expression   // Inequality
expression < expression    // Less than
expression > expression    // Greater than
expression <= expression   // Less than or equal
expression >= expression   // Greater than or equal
```

**Left-associative:** Multiple comparisons chain left-to-right

**Examples:**
```why
let is_equal = x == y;
let is_valid = age >= 18 && age <= 65;  // Note: && not shown in grammar
let in_range = min <= value && value <= max;
```

### Assignment Operator (Precedence 0)

```why
lvalue = expression    // Assignment
```

**Right-associative:** `a = b = c` equals `a = (b = c)`

**Examples:**
```why
x = 42;
array[i] = value;
struct_instance.field = new_value;
```

## Precedence Examples

### Arithmetic Expressions

```why
// Without parentheses
let result1 = 2 + 3 * 4;        // 2 + (3 * 4) = 14
let result2 = 10 - 6 / 2;       // 10 - (6 / 2) = 7
let result3 = a + b * c - d;    // a + (b * c) - d

// With explicit parentheses for clarity
let result4 = (2 + 3) * 4;      // 20
let result5 = (10 - 6) / 2;     // 2
```

### Mixed Arithmetic and Comparison

```why
// Arithmetic before comparison
let is_positive = x + y > 0;         // (x + y) > 0
let in_bounds = i * 2 < array.length(); // (i * 2) < (array.length())

// Explicit grouping
let complex_check = (a + b) * (c - d) >= threshold;
```

### Function Calls and Property Access

```why
// Postfix operators have highest precedence
let result = object.method().value + 10;    // ((object.method()).value) + 10
let data = array[index].process() * factor; // ((array[index]).process()) * factor

// Method chaining
let formatted = user.get_name().to_lower().trim();
```

### Complex Expressions

```why
// Multiple precedence levels
let complex = base + offset * scale > threshold;
// Parsed as: (base + (offset * scale)) > threshold

let calculation = func(a + b) * array[i] - constant;
// Parsed as: (func(a + b) * array[i]) - constant

let validation = struct_obj.validate(input.trim()) == expected_result;
// Parsed as: (struct_obj.validate(input.trim())) == expected_result
```

## Common Precedence Pitfalls

### Unary Minus vs Binary Minus

```why
let a = 5;
let b = 3;

let result1 = a + -b;    // a + (-b) = 5 + (-3) = 2
let result2 = a - b;     // a - b = 5 - 3 = 2
let result3 = a+-b;      // Same as result1, but less readable
```

### Array Access vs Multiplication

```why
let array = &[1, 2, 3, 4];
let index = 1;
let multiplier = 2;

let value = array[index] * multiplier;  // (array[index]) * multiplier = 2 * 2 = 4
// NOT array[(index * multiplier)]
```

### Function Calls vs Arithmetic

```why
fn get_value(): i64 { 42 }
fn calculate(x: i64): i64 { x * 2 }

let result = get_value() + calculate(10);  // 42 + 20 = 62
let scaled = get_value() * 2 + 1;          // (42 * 2) + 1 = 85
```

### Method Calls vs Comparison

```why
struct Counter {
    value: i64;
}

instance Counter {
    fn get(): i64 { this.value }
}

let counter = Counter { value: 42 };
let is_large = counter.get() > 30;  // (counter.get()) > 30 = true
```

## Best Practices

### Use Parentheses for Clarity

Even when precedence rules make them unnecessary, parentheses can improve readability:

```why
// Technically correct but potentially confusing
let result = a + b * c - d / e;

// Clearer with explicit grouping
let result = a + (b * c) - (d / e);

// Very clear with intermediate variables
let product = b * c;
let quotient = d / e;
let result = a + product - quotient;
```

### Break Complex Expressions

```why
// Hard to read
let complex = object.method(param1 + param2 * factor).process() > threshold && flag;

// Better
let adjusted_param = param2 * factor;
let method_result = object.method(param1 + adjusted_param);
let processed = method_result.process();
let is_valid = processed > threshold && flag;
```

### Consistent Spacing

```why
// Good: Consistent spacing helps show precedence
let result = a + b * c;
let check = value >= min && value <= max;

// Less clear: Inconsistent spacing
let result = a+b*c;
let check = value>=min&&value<=max;
```

## Associativity Examples

### Left Associativity

Most operators are left-associative:

```why
// Addition (left-associative)
let sum = a + b + c + d;  // ((a + b) + c) + d

// Subtraction (left-associative)
let diff = a - b - c;     // (a - b) - c

// Multiplication (left-associative)
let product = a * b * c;  // (a * b) * c

// Division (left-associative)
let quotient = a / b / c; // (a / b) / c
```

### Right Associativity

Assignment is right-associative:

```why
// Assignment (right-associative)
let mut a: i64 = 0;
let mut b: i64 = 0;
let mut c: i64 = 0;

a = b = c = 42;  // a = (b = (c = 42))
// Result: a = 42, b = 42, c = 42
```

This precedence reference should help you write correct and readable Y expressions without unexpected operator precedence issues.