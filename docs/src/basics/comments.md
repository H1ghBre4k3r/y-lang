# Comments

Comments are text annotations in your code that are ignored by the compiler. They're essential for documenting your code, explaining complex logic, and leaving notes for yourself and other developers.

## Single-Line Comments

Y supports single-line comments using `//`. Everything after `//` on that line is ignored by the compiler:

```why
// This is a single-line comment
let x = 42;  // This is also a comment

fn calculate_area(width: i64, height: i64): i64 {
    // Calculate the area of a rectangle
    width * height
}
```

## Block Comments

Y supports block comments using `/* */`. Everything between these markers is ignored, even across multiple lines:

```why
/*
This is a block comment.
It can span multiple lines.
*/

let result = calculate(
    42,
    /*
    This parameter represents the initial value
    for our calculation
    */
    100
);
```

### Nested Block Comments

Y supports nested block comments, which makes it easy to comment out large sections of code that already contain comments:

```why
/*
This is the outer comment.
/*
    This is a nested comment inside the outer one.
    /* And this is even more deeply nested! */
*/
The outer comment continues here.
*/
```

## Documentation Comments

While Y doesn't currently have a built-in documentation system like Rust's `///` or Java's `/** */`, you can use comments to document your code effectively:

### Function Documentation

```why
// Calculates the factorial of a positive integer.
//
// Parameters:
//   n: The number to calculate the factorial for (must be >= 0)
//
// Returns:
//   The factorial of n, or 1 if n is 0
//
// Examples:
//   factorial(5) returns 120
//   factorial(0) returns 1
fn factorial(n: i64): i64 {
    if n <= 1 {
        1
    } else {
        n * factorial(n - 1)
    }
}
```

### Struct Documentation

```why
// Represents a point in 2D space.
//
// This struct is used throughout the graphics system
// to represent coordinates, offsets, and dimensions.
struct Point {
    // The x-coordinate (horizontal position)
    x: i64;

    // The y-coordinate (vertical position)
    y: i64;
}
```

### Complex Algorithm Documentation

```why
// Implementation of binary search algorithm.
//
// This function searches for a target value in a sorted array
// using the divide-and-conquer approach. Time complexity: O(log n)
fn binary_search(arr: &[i64], target: i64): i64 {
    let mut left = 0;
    let mut right = arr.len() - 1;

    while left <= right {
        // Calculate middle index, avoiding overflow
        let mid = left + (right - left) / 2;

        if arr[mid] == target {
            return mid;  // Found the target
        } else if arr[mid] < target {
            left = mid + 1;  // Search right half
        } else {
            right = mid - 1; // Search left half
        }
    }

    -1  // Target not found
}
```

## Best Practices

### 1. Explain "Why", Not "What"

Good comments explain the reasoning behind code, not just what the code does:

```why
// Good: Explains why
// Use a binary search because the array is already sorted
// and we need O(log n) performance for frequent lookups
let index = binary_search(&sorted_data, target);

// Less helpful: Just restates what the code does
// Search for target in sorted_data
let index = binary_search(&sorted_data, target);
```

### 2. Keep Comments Current

Update comments when you change code:

```why
// Old comment that's now incorrect:
// Multiply by 2 to double the value
let result = x * 3;  // Code was changed but comment wasn't!

// Correct:
// Multiply by 3 to triple the value
let result = x * 3;
```

### 3. Use Comments for Complex Business Logic

```why
// Apply a 10% discount for orders over $100,
// but only for customers who have been active
// in the last 30 days (as per marketing requirements)
if order_total > 100 && customer.last_active_days <= 30 {
    order_total = order_total * 0.9;
}
```

### 4. Comment Temporary or Unusual Solutions

```why
// TODO: This is a temporary workaround for the memory leak
// in the third-party library. Remove this when library
// version 2.1 is released (fixes issue #1234)
unsafe_workaround_function();

// HACK: The API returns inconsistent data formats
// on Tuesdays (yes, really!), so we need special handling
if is_tuesday() {
    data = normalize_tuesday_format(data);
}
```

## Comment Conventions

### TODO Comments

Mark future work with TODO comments:

```why
// TODO: Implement error handling for network timeouts
// TODO: Add input validation for negative numbers
// TODO: Optimize this algorithm for better performance
```

### FIXME Comments

Mark known bugs or issues:

```why
// FIXME: This crashes when input array is empty
// FIXME: Memory leak in the loop below
// FIXME: Race condition when multiple threads access this
```

### NOTE Comments

Mark important implementation details:

```why
// NOTE: This function assumes the input is already sorted
// NOTE: Changing this value affects the entire system
// NOTE: Thread-safe, but not async-safe
```

## Commenting Out Code

Use comments to temporarily disable code during development:

```why
fn debug_mode() {
    // Normal operation
    let result = calculate_value();

    // Temporarily disabled for debugging
    // send_to_server(result);
    // update_database(result);

    // Alternative debug behavior
    println("Debug: result = {}", result);
}
```

### Block Comments for Large Sections

```why
/*
// Old implementation - keeping for reference
fn old_algorithm(data: &[i64]): i64 {
    let mut sum = 0;
    let mut i = 0;
    while i < data.len() {
        sum = sum + data[i];
        i = i + 1;
    }
    sum
}
*/

// New optimized implementation
fn new_algorithm(data: &[i64]): i64 {
    data.iter().sum()  // Much cleaner!
}
```

## Performance Considerations

Comments have zero runtime cost - they're completely removed during compilation:

```why
// This comment doesn't affect performance at all
let x = expensive_calculation();  // Neither does this one

/*
Even large block comments like this one
with multiple lines of explanation
have absolutely no impact on the final
compiled program's performance.
*/
```

## Common Anti-Patterns

### 1. Obvious Comments

Avoid comments that just restate what the code clearly shows:

```why
// Bad: Obvious comment
let i = i + 1;  // Increment i by 1

// Good: Explains purpose
let i = i + 1;  // Move to next item in the processing queue
```

### 2. Commented-Out Code Without Context

```why
// Bad: No explanation
let result = new_calculation();
// let result = old_calculation();

// Good: Explains why code is commented
let result = new_calculation();
// TODO: Remove old_calculation() after performance testing is complete
// let result = old_calculation();
```

### 3. Outdated Comments

```why
// Bad: Comment doesn't match code
// Return the username
fn get_user_id(): i64 {  // Function actually returns ID, not name!
    42
}
```

## Tools and Integration

### Language Server Support

The Y language server can use comments to provide better development experience:

- Hover information from function comments
- Code completion hints
- Contextual help

### Future Documentation Generation

While not currently implemented, Y may support automatic documentation generation from specially formatted comments in the future:

```why
/// Calculates the area of a circle.
///
/// # Arguments
/// * `radius` - The radius of the circle
///
/// # Returns
/// The area as a floating-point number
///
/// # Examples
/// ```
/// let area = circle_area(5.0);
/// assert_eq!(area, 78.54);
/// ```
fn circle_area(radius: f64): f64 {
    3.14159 * radius * radius
}
```

Comments are a crucial tool for writing maintainable, understandable code. Use them wisely to make your Y programs self-documenting and easier to work with over time.