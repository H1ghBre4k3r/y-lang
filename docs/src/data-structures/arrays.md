# Arrays

Arrays in Y are reference types that store multiple values of the same type in an ordered sequence. They provide efficient access to elements by index.

## Array Syntax

Arrays use the `&[T]` type syntax, where `T` is the element type:

```why
let numbers: &[i64] = &[1, 2, 3, 4, 5];
let characters: &[char] = &['a', 'b', 'c'];
let booleans: &[bool] = &[true, false, true];
```

## Creating Arrays

### Array Literals

The most common way to create arrays is using array literal syntax:

```why
let fruits = &["apple", "banana", "orange"];
let primes = &[2, 3, 5, 7, 11];
let mixed_numbers = &[42, 1337, 0];
```

### Empty Arrays

Empty arrays require explicit type annotation:

```why
let empty_numbers: &[i64] = &[];
let empty_strings: &[str] = &[];
```

### Arrays from Expressions

Array elements can be any expression:

```why
fn get_value(): i64 { 42 }

let computed = &[
    get_value(),
    10 + 20,
    if (true) { 100 } else { 0 }
];
```

## Accessing Array Elements

Use square bracket notation with zero-based indexing:

```why
let numbers = &[10, 20, 30, 40, 50];

let first = numbers[0];   // 10
let third = numbers[2];   // 30
let last = numbers[4];    // 50
```

## Modifying Arrays

Arrays can be mutable, allowing you to change element values:

```why
let mut scores = &[85, 92, 78];

scores[0] = 95;     // Change first element
scores[2] = 88;     // Change third element
// scores is now &[95, 92, 88]
```

Note: You can only modify elements, not add or remove them (arrays have fixed size).

## Array Examples from Y Programs

### Basic Array Operations

```why
fn main(): void {
    let mut arr = &[42, 1337];
    let arr2 = &[1337, 5];

    // Access elements
    let first = arr[0];
    let value = arr2[3];  // Note: This might be out of bounds

    // Modify elements
    arr[0] = 100;
    arr[1] = 200;
}
```

### Arrays with Different Types

```why
fn working_with_arrays(): void {
    // Character arrays
    let mut char_array = &['a', 'b'];
    char_array[1] = 'z';

    // Mixed content (all same type)
    let test_char = 'a';
    let mut foo = &[test_char, 'b'];
    foo[1] = test_char;
}
```

### Arrays in Structs

```why
struct Container {
    data: &[i64];
    size: i64;
}

fn create_container(): Container {
    Container {
        data: &[1, 2, 3, 4, 5],
        size: 5
    }
}
```

## Array Type Compatibility

Arrays are strictly typed - all elements must be the same type:

```why
// Valid:
let numbers = &[1, 2, 3];           // All i64
let words = &["hello", "world"];    // All str

// Invalid:
// let mixed = &[1, "hello", true]; // Error: mixed types
```

## Working with Array References

Arrays in Y are reference types, meaning they refer to data stored elsewhere:

```why
let original = &[1, 2, 3];
let reference = original;  // Both refer to the same array data

let mut mutable_ref = &[4, 5, 6];
modify_array(mutable_ref);  // Function can modify the array
```

## Practical Examples

### Processing Array Data

```why
fn sum_array(arr: &[i64]): i64 {
    let mut total = 0;
    let mut i = 0;

    // Manual iteration (Y doesn't have for loops yet)
    while (i < array_length(arr)) {
        total = total + arr[i];
        i = i + 1;
    }

    return total;
}
```

### Array as Function Parameter

```why
fn process_scores(scores: &[i64]): i64 {
    let first_score = scores[0];
    let last_score = scores[scores.length() - 1];  // Assuming length method
    return (first_score + last_score) / 2;
}
```

### Arrays in Complex Data Structures

```why
struct Matrix {
    rows: &[&[i64]];  // Array of arrays
    width: i64;
    height: i64;
}

struct DataSet {
    values: &[f64];
    labels: &[str];
    metadata: &[bool];
}
```

## Array Limitations and Considerations

1. **Fixed Size**: Arrays have a fixed size determined at creation
2. **Bounds Checking**: Accessing out-of-bounds indices may cause runtime errors
3. **Homogeneous**: All elements must be the same type
4. **Reference Type**: Arrays are references, not value types

## Best Practices

1. **Initialize with known data**: Prefer creating arrays with initial values
2. **Use meaningful names**: Choose descriptive variable names for arrays
3. **Bounds awareness**: Be careful with index calculations to avoid out-of-bounds access
4. **Type consistency**: Ensure all elements are the same type
5. **Mutability**: Only make arrays mutable when you need to modify elements

```why
// Good practices:
let player_scores = &[95, 87, 92, 88];  // Descriptive name
let mut high_scores = &[100, 95, 90];   // Mutable only when needed

// Less ideal:
let a = &[1, 2, 3];                     // Non-descriptive name
let mut data = &[1, 2, 3];              // Unnecessary mutability
```