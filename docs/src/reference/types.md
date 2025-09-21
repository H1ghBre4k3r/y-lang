# Built-in Types Reference

This page provides comprehensive reference information for all built-in types in the Y programming language, including their properties, memory layout, valid operations, and usage patterns.

## Primitive Types

### Integer Types

#### `i64` - 64-bit Signed Integer

- **Size**: 8 bytes (64 bits)
- **Range**: -9,223,372,036,854,775,808 to 9,223,372,036,854,775,807
- **Default numeric type**: Most integer literals default to `i64`

```why
let count: i64 = 42;
let negative: i64 = -1000;
let max_value: i64 = 9223372036854775807;
```

**Operations:**
- Arithmetic: `+`, `-`, `*`, `/`
- Comparison: `==`, `!=`, `<`, `>`, `<=`, `>=`
- Assignment: `=`

#### `u32` - 32-bit Unsigned Integer

- **Size**: 4 bytes (32 bits)
- **Range**: 0 to 4,294,967,295
- **Use case**: When you need a smaller integer type or explicitly unsigned values

```why
let port: u32 = 8080;
let size: u32 = 1024;
let max_u32: u32 = 4294967295;
```

**Operations:**
- Arithmetic: `+`, `-`, `*`, `/`
- Comparison: `==`, `!=`, `<`, `>`, `<=`, `>=`
- Assignment: `=`

### Floating Point Types

#### `f64` - 64-bit Floating Point

- **Size**: 8 bytes (64 bits)
- **Range**: ±1.7976931348623157E+308 (IEEE 754 double precision)
- **Precision**: ~15-17 decimal digits
- **Default floating type**: Floating literals default to `f64`

```why
let pi: f64 = 3.14159265358979;
let price: f64 = 99.99;
let scientific: f64 = 1.23e-4;  // 0.000123
```

**Operations:**
- Arithmetic: `+`, `-`, `*`, `/`
- Comparison: `==`, `!=`, `<`, `>`, `<=`, `>=`
- Assignment: `=`

**Special Values:**
- Positive infinity
- Negative infinity
- NaN (Not a Number)

### Boolean Type

#### `bool` - Boolean

- **Size**: 1 byte
- **Values**: `true` or `false`
- **Use case**: Logical operations, conditions, flags

```why
let is_ready: bool = true;
let debug_mode: bool = false;
let result: bool = x > 0;
```

**Operations:**
- Logical: `&&`, `||`, `!` (conceptual - may not be fully implemented)
- Comparison: `==`, `!=`
- Assignment: `=`

### Character Types

#### `char` - Unicode Character

- **Size**: Variable (1-4 bytes UTF-8)
- **Range**: Any valid Unicode code point
- **Use case**: Single characters, text processing

```why
let letter: char = 'a';
let digit: char = '5';
let unicode: char = '🚀';
let escape: char = '\n';
```

**Operations:**
- Comparison: `==`, `!=`, `<`, `>`, `<=`, `>=`
- Assignment: `=`

**Escape Sequences:**
- `\n` - Newline
- `\t` - Tab
- `\\` - Backslash
- `\'` - Single quote
- `\"` - Double quote

#### `str` - String Slice

- **Size**: Variable (UTF-8 encoded)
- **Properties**: Immutable sequence of characters
- **Use case**: Text data, string literals

```why
let greeting: str = "Hello, World!";
let empty: str = "";
let multiline: str = "Line 1\nLine 2";
```

**Operations:**
- Comparison: `==`, `!=`, `<`, `>`, `<=`, `>=` (lexicographic)
- Assignment: `=`
- Property access: `.len()` (when available)

## Composite Types

### Array Types

#### `&[T]` - Array Reference

- **Size**: Pointer to data + length information
- **Properties**: Reference to a contiguous sequence of elements of type `T`
- **Mutability**: Elements can be modified if array is mutable

```why
let numbers: &[i64] = &[1, 2, 3, 4, 5];
let empty: &[str] = &[];
let mut mutable_array: &[i64] = &[10, 20, 30];
```

**Operations:**
- Indexing: `array[index]`
- Assignment (to elements): `array[index] = value`
- Property access: `.length()` (conceptual)

**Memory Layout:**
```
&[i64] -> [pointer to data][length]
           |
           v
          [elem0][elem1][elem2]...
```

### Function Types

#### `(ParamTypes) -> ReturnType` - Function Type

- **Size**: Pointer size (8 bytes on 64-bit systems)
- **Properties**: Reference to executable code
- **Use case**: Function parameters, callbacks, stored procedures

```why
// Function taking no parameters, returning i64
let getter: () -> i64 = get_value;

// Function taking two i64s, returning i64
let binary_op: (i64, i64) -> i64 = add;

// Function taking a function, returning i64
let higher_order: ((i64) -> i64) -> i64 = apply_twice;
```

**Operations:**
- Function call: `function(arguments)`
- Assignment: `=`
- Comparison: `==`, `!=` (identity comparison)

### Special Types

#### `void` - No Value

- **Size**: 0 bytes
- **Use case**: Functions that don't return a value
- **Note**: Cannot be stored in variables, only used as return type

```why
fn print_message(msg: str): void {
    printf(msg);
    // No return value
}
```

## Type Conversion and Compatibility

### Implicit Conversions

Y has **no implicit type conversions**. All type conversions must be explicit.

```why
let int_val: i64 = 42;
let uint_val: u32 = 100;

// This would be an error:
// let result = int_val + uint_val;  // Error: type mismatch

// Explicit conversion required (conceptual):
// let result = int_val + to_i64(uint_val);
```

### Type Compatibility Rules

- **Exact match required**: Variables must match their declared types exactly
- **No automatic promotion**: `u32` doesn't automatically become `i64`
- **No automatic widening**: `f32` (if it existed) wouldn't become `f64`

## Memory Layout and Alignment

### Primitive Type Sizes

| Type | Size (bytes) | Alignment (bytes) |
|------|--------------|-------------------|
| `i64` | 8 | 8 |
| `u32` | 4 | 4 |
| `f64` | 8 | 8 |
| `bool` | 1 | 1 |
| `char` | 1-4 | 1 |
| `str` | Variable | 1 |

### Array Memory Layout

```why
let array: &[i64] = &[1, 2, 3];
```

Memory layout:
```
array variable: [data_ptr: 8 bytes][length: 8 bytes]
                     |
                     v
heap/stack data: [1: 8 bytes][2: 8 bytes][3: 8 bytes]
```

### Struct Memory Layout

```why
struct Example {
    flag: bool;    // 1 byte
    count: i64;    // 8 bytes (may have 7 bytes padding before)
    value: f64;    // 8 bytes
}
```

Memory layout (with alignment):
```
[flag: 1 byte][padding: 7 bytes][count: 8 bytes][value: 8 bytes]
Total size: 24 bytes
```

## Type Usage Patterns

### Choosing Numeric Types

```why
// Use i64 for general integer values
let age: i64 = 25;
let count: i64 = 1000;

// Use u32 for specific cases requiring unsigned values
let port: u32 = 8080;
let size: u32 = 1024;

// Use f64 for floating point calculations
let price: f64 = 99.99;
let percentage: f64 = 0.15;
```

### Working with Strings and Characters

```why
// String literals for text data
let message: str = "Hello, World!";
let filename: str = "data.txt";

// Characters for individual character processing
let separator: char = ',';
let newline: char = '\n';

// Arrays of characters for mutable text processing
let mut buffer: &[char] = &['H', 'e', 'l', 'l', 'o'];
```

### Function Type Patterns

```why
// Simple callback
let callback: () -> void = cleanup;

// Data processor
let processor: (str) -> str = normalize_text;

// Predicate function
let validator: (i64) -> bool = is_valid_age;

// Higher-order function
let mapper: (&[i64], (i64) -> i64) -> &[i64] = transform_array;
```

## Type Limits and Constraints

### Integer Overflow

Y's integer types have defined overflow behavior (implementation-dependent):

```why
let max_i64: i64 = 9223372036854775807;
// let overflow = max_i64 + 1;  // Behavior depends on implementation
```

### Floating Point Precision

```why
let precise: f64 = 0.1 + 0.2;  // May not exactly equal 0.3
let comparison: bool = precise == 0.3;  // May be false due to precision
```

### Array Bounds

```why
let array: &[i64] = &[1, 2, 3];
let valid: i64 = array[0];     // Valid: index 0
let valid2: i64 = array[2];    // Valid: index 2
// let invalid = array[3];     // Runtime error: out of bounds
```

## Best Practices

### Type Selection

- Use `i64` for general-purpose integers
- Use `u32` only when you specifically need unsigned semantics
- Use `f64` for all floating-point calculations
- Use `bool` for true/false values, not integers
- Use `char` for individual characters, `str` for text

### Type Annotations

```why
// Explicit when type isn't obvious
let empty_array: &[i64] = &[];
let function_var: (i64) -> bool = validator;

// Can be omitted when obvious
let count = 42;           // Obviously i64
let name = "Alice";       // Obviously str
let ready = true;         // Obviously bool
```

### Safe Operations

```why
// Check bounds before array access
fn safe_get(array: &[i64], index: i64): i64 {
    if (index >= 0 && index < array.length()) {
        return array[index];
    } else {
        return 0;  // Or appropriate default/error handling
    }
}

// Validate function parameters
fn safe_divide(a: f64, b: f64): f64 {
    if (b != 0.0) {
        return a / b;
    } else {
        return 0.0;  // Or appropriate error handling
    }
}
```