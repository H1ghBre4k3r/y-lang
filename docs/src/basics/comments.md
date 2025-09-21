# Comments

Comments are used to add explanatory notes to your code that are ignored by the compiler. They help make code more readable and maintainable.

## Line Comments

Y supports single-line comments using `//`. Everything after `//` on that line is treated as a comment:

```why
// This is a comment
let x = 42;  // This is also a comment

fn main(): i64 {
    // Calculate the result
    let a = 10;
    let b = 20;
    return a + b;  // Return the sum
}
```

## Comment Best Practices

### Explaining Intent

Use comments to explain *why* something is done, not just *what* is done:

```why
// Good: Explains the reasoning
let buffer_size = 1024;  // Use power of 2 for optimal memory alignment

// Less helpful: Just restates the code
let x = 42;  // Set x to 42
```

### Documenting Complex Logic

```why
fn complex_calculation(input: i64): i64 {
    // Apply custom business rule: multiply by 3, add offset
    let intermediate = input * 3;
    let offset = 10;

    // Ensure result stays within acceptable range
    if (intermediate + offset > 1000) {
        return 1000;
    } else {
        return intermediate + offset;
    }
}
```

### Temporary Debugging

Comments can be used to temporarily disable code during development:

```why
fn debug_function(): void {
    let x = calculate_value();
    // let y = expensive_operation();  // Temporarily disabled

    process_result(x);
}
```

## Comments in Examples

Looking at real Y code from the examples:

```why
struct TestStruct {
    x: i64;
    bar: (i64, i64) -> i64;
}

fn main(): i64 {
    let a = add(42, 1337);

    // Create mutable array with initial values
    let mut arr = &[42, 1337];

    // Access array elements
    let b = explicit_return_add(arr[0], arr2[3]);

    // Initialize struct with function reference
    let my_struct = TestStruct {
        x: 42,
        bar: add  // Function as struct field
    };

    return 0;
}
```

## Multi-line Explanations

For longer explanations, use multiple single-line comments:

```why
// This function implements a custom sorting algorithm
// optimized for small arrays (< 10 elements).
// For larger arrays, consider using a different approach.
fn small_array_sort(arr: &[i64]): &[i64] {
    // Implementation here...
    return arr;
}
```

## Comment Style Guidelines

- Use clear, concise language
- Keep comments up-to-date with code changes
- Avoid obvious comments that just restate the code
- Use proper grammar and punctuation
- Be consistent with comment style throughout your codebase

## What Not to Comment

Avoid commenting obvious code:

```why
// Bad examples:
let x = 42;        // Assign 42 to x
i = i + 1;         // Increment i
return result;     // Return the result

// Good examples:
let timeout = 30;  // Connection timeout in seconds
i = i + 1;         // Move to next element
return result;     // Early return to avoid expensive calculation
```