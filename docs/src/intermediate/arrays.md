# Arrays

Arrays in Y are collections of elements of the same type, stored in contiguous memory. They provide efficient random access and are fundamental for many programming tasks.

## Array Syntax

Arrays in Y use the reference syntax with `&[type]`:

```why
// Array literal with explicit values
let numbers = &[1, 2, 3, 4, 5];

// Empty array with type annotation
let empty: &[i64] = &[];

// Array with type annotation
let scores: &[f64] = &[85.5, 92.0, 78.5];
```

## Array Access

Use square brackets for indexing (zero-based):

```why
let numbers = &[10, 20, 30, 40, 50];

let first = numbers[0];   // 10
let third = numbers[2];   // 30
let last = numbers[4];    // 50
```

## Mutable Arrays

Arrays can be mutable, allowing modification of elements:

```why
let mut scores = &[85, 90, 78, 92, 88];

scores[0] = 95;  // Modify first element
scores[2] = 82;  // Modify third element

// scores is now &[95, 90, 82, 92, 88]
```

## Array Properties

### Length

Get array length using `.len()` method:

```why
let numbers = &[1, 2, 3, 4, 5];
let count = numbers.len();  // 5

let empty: &[i64] = &[];
let empty_count = empty.len();  // 0
```

## Array Operations

### Iteration

Loop through array elements:

```why
let numbers = &[1, 2, 3, 4, 5];

// Using while loop with index
let mut i = 0;
while i < numbers.len() {
    let value = numbers[i];
    // Process value...
    i = i + 1;
}
```

### Common Patterns

#### Finding Elements

```why
fn find_value(arr: &[i64], target: i64): i64 {
    let mut i = 0;
    while i < arr.len() {
        if arr[i] == target {
            return i;  // Return index if found
        }
        i = i + 1;
    }
    -1  // Return -1 if not found
}
```

#### Sum Calculation

```why
fn sum_array(arr: &[i64]): i64 {
    let mut total = 0;
    let mut i = 0;
    while i < arr.len() {
        total = total + arr[i];
        i = i + 1;
    }
    total
}
```

#### Maximum Value

```why
fn find_max(arr: &[i64]): i64 {
    if arr.len() == 0 {
        return 0;  // Or handle empty array case
    }

    let mut max = arr[0];
    let mut i = 1;
    while i < arr.len() {
        if arr[i] > max {
            max = arr[i];
        }
        i = i + 1;
    }
    max
}
```

## Multi-dimensional Arrays

Arrays of arrays create multi-dimensional structures:

```why
// 2D array (array of arrays)
let matrix = &[
    &[1, 2, 3],
    &[4, 5, 6],
    &[7, 8, 9]
];

// Access 2D elements
let value = matrix[1][2];  // Gets 6 (row 1, column 2)

// Mutable 2D array
let mut grid = &[
    &[0, 0, 0],
    &[0, 0, 0],
    &[0, 0, 0]
];

grid[1][1] = 5;  // Set center element
```

## Array Best Practices

### 1. Use Descriptive Names

```why
// Good: Clear purpose
let temperatures = &[23.5, 25.0, 22.8, 24.1];
let user_scores = &[85, 92, 78, 90];

// Less clear
let data = &[23.5, 25.0, 22.8, 24.1];
let values = &[85, 92, 78, 90];
```

### 2. Check Bounds

Always ensure array access is within bounds:

```why
fn safe_get(arr: &[i64], index: i64): i64 {
    if index >= 0 && index < arr.len() {
        arr[index]
    } else {
        0  // Default value or handle error
    }
}
```

### 3. Prefer Immutable When Possible

```why
// Good: Immutable when data doesn't change
let configuration = &[1, 2, 4, 8, 16];

// Only mutable when needed
let mut buffer = &[0, 0, 0, 0, 0];
```

## Memory Considerations

### Stack vs Heap

- Array data may be allocated on the heap
- Array references themselves are typically stack-allocated
- Large arrays should be used carefully to avoid stack overflow

```why
// Small arrays are fine
let small = &[1, 2, 3, 4, 5];

// Very large arrays should be considered carefully
let large = &[0; 10000];  // Creates array with 10000 zeros
```

### Copying Behavior

Arrays are copied when assigned or passed to functions:

```why
let original = &[1, 2, 3];
let copy = original;  // Creates a copy

fn process_array(arr: &[i64]) {
    // arr is a copy of the original
}
```

## Common Use Cases

### Buffers and Storage

```why
// Buffer for reading data
let mut buffer = &[0; 1024];

// Storage for processing results
let mut results = &[0; 100];
```

### Lookup Tables

```why
// Days in each month
const DAYS_IN_MONTH: &[i64] = &[31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];

fn days_in_month(month: i64): i64 {
    if month >= 1 && month <= 12 {
        DAYS_IN_MONTH[month - 1]
    } else {
        0
    }
}
```

### Mathematical Operations

```why
// Vector operations
fn dot_product(a: &[f64], b: &[f64]): f64 {
    if a.len() != b.len() {
        return 0.0;  // Handle size mismatch
    }

    let mut result = 0.0;
    let mut i = 0;
    while i < a.len() {
        result = result + (a[i] * b[i]);
        i = i + 1;
    }
    result
}
```

## Array Algorithms

### Sorting (Bubble Sort Example)

```why
fn bubble_sort(arr: &mut [i64]) {
    let n = arr.len();
    let mut i = 0;

    while i < n {
        let mut j = 0;
        while j < n - 1 - i {
            if arr[j] > arr[j + 1] {
                // Swap elements
                let temp = arr[j];
                arr[j] = arr[j + 1];
                arr[j + 1] = temp;
            }
            j = j + 1;
        }
        i = i + 1;
    }
}
```

### Binary Search

```why
fn binary_search(arr: &[i64], target: i64): i64 {
    let mut left = 0;
    let mut right = arr.len() - 1;

    while left <= right {
        let mid = left + (right - left) / 2;

        if arr[mid] == target {
            return mid;
        } else if arr[mid] < target {
            left = mid + 1;
        } else {
            right = mid - 1;
        }
    }

    -1  // Not found
}
```

Arrays are fundamental data structures in Y that provide efficient storage and access patterns for collections of data. Use them effectively to build robust and performant applications.