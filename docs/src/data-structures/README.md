# Data Structures

Y provides several built-in data structures for organizing and storing data. This section covers the fundamental data structures available in the language.

## Overview

Y supports the following primary data structures:

- **Arrays** - Ordered collections of elements of the same type
- **Structs** - Custom data types that group related fields together

## Arrays

Arrays in Y are reference types that store multiple values of the same type in an ordered sequence. They use the `&[T]` syntax where `T` is the element type.

```why
let numbers = &[1, 2, 3, 4, 5];
let chars = &['a', 'b', 'c'];
let empty: &[i64] = &[];
```

Arrays support indexing for accessing and modifying elements:

```why
let mut arr = &[10, 20, 30];
let first = arr[0];  // Access: 10
arr[1] = 99;         // Modify: [10, 99, 30]
```

## Structs

Structs allow you to create custom data types by grouping related fields:

```why
struct Person {
    name: str;
    age: i64;
}

let person = Person {
    name: "Alice",
    age: 30
};
```

Structs support:
- Field access via dot notation
- Mutable field modification
- Nesting of other structs
- Methods through instance blocks

## Choosing the Right Data Structure

- **Use arrays** when you need an ordered collection of the same type of data
- **Use structs** when you need to group different types of data that belong together
- **Combine both** for complex data modeling (arrays of structs, structs with array fields)

## Example: Combining Data Structures

```why
struct Student {
    name: str;
    grades: &[i64];
}

let students = &[
    Student {
        name: "Alice",
        grades: &[95, 87, 92]
    },
    Student {
        name: "Bob",
        grades: &[88, 79, 94]
    }
];
```

The following pages provide detailed information about each data structure type.