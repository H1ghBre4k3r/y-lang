# Type Annotations

Type annotations in Y provide explicit type information that helps with code clarity, type checking, and compiler optimization. While Y has type inference capabilities, explicit type annotations are required in certain contexts and recommended for complex types.

## Basic Type Annotation Syntax

Type annotations use the colon (`:`) syntax:

```why
let variable_name: Type = value;
fn function_name(param: Type): ReturnType { ... }
```

## Variable Type Annotations

### Simple Types

```why
let age: i64 = 25;
let price: f64 = 99.99;
let name: str = "Alice";
let is_active: bool = true;
let grade: char = 'A';
```

### When Type Inference Isn't Enough

```why
// Empty arrays need explicit types
let numbers: &[i64] = &[];
let names: &[str] = &[];

// Ambiguous numeric literals
let count: u32 = 42;  // Specify u32 instead of default i64
```

## Function Type Annotations

### Function Parameters

All function parameters require type annotations:

```why
fn calculate_area(width: f64, height: f64): f64 {
    width * height
}

fn process_user(user: User, active: bool): void {
    // Function implementation
}
```

### Return Types

Functions must specify their return type:

```why
fn get_name(): str {
    "Anonymous"
}

fn is_valid(input: str): bool {
    input.len() > 0
}

fn process_data(): void {
    // No return value
}
```

## Complex Type Annotations

### Function Types

Function types use the `(param_types) -> return_type` syntax:

```why
// Function that takes two i64s and returns i64
let binary_op: (i64, i64) -> i64 = add;

// Function that takes no parameters and returns str
let getter: () -> str = get_default_name;

// Function that takes a function as parameter
let processor: ((i64) -> i64) -> i64 = apply_twice;
```

### Array Types

Array types specify the element type:

```why
let scores: &[i64] = &[95, 87, 92];
let names: &[str] = &["Alice", "Bob", "Charlie"];
let flags: &[bool] = &[true, false, true];

// Multidimensional arrays
let matrix: &[&[i64]] = &[&[1, 2], &[3, 4]];
```

### Struct Types

Struct types use the struct name:

```why
struct Point {
    x: f64;
    y: f64;
}

let origin: Point = Point { x: 0.0, y: 0.0 };
let location: Point = calculate_position();
```

## Advanced Type Annotations

### Higher-Order Function Types

Complex function types that involve functions as parameters or return values:

```why
// Function that takes a function and returns a function
let transformer: ((i64) -> i64) -> ((i64) -> i64) = create_transformer;

// Function that takes multiple function parameters
let combiner: ((i64) -> i64, (i64) -> i64, i64) -> i64 = combine_operations;

// Function returning a function that takes a function
let factory: () -> ((str) -> bool) = create_validator;
```

### Nested Function Types

```why
// Function that processes arrays with custom logic
let array_processor: (&[i64], (i64) -> bool) -> &[i64] = filter_array;

// Event handler type
let event_handler: (str, () -> void) -> void = register_handler;

// Complex data transformer
let data_transform: (&[str], (str) -> str, (str) -> bool) -> &[str] = process_strings;
```

## Examples from Y Code

### Function Variables with Type Annotations

```why
fn add(x: i64, y: i64): i64 {
    x + y
}

fn main(): i64 {
    // Explicit function type annotation
    let x: (i64) -> i64 = \(x) => x;

    // Using the annotated function
    let result = x(42);
    return result;
}
```

### Struct with Function Fields

```why
struct TestStruct {
    x: i64;
    bar: (i64, i64) -> i64;  // Function type in struct
}

let my_struct: TestStruct = TestStruct {
    x: 42,
    bar: add  // Function reference
};
```

### Array with Type Annotation

```why
fn main(): void {
    let x: &[i64] = &[];  // Empty array needs type annotation
    x[0];  // Access array element
}
```

## Type Annotations in Practice

### Configuration Objects

```why
struct DatabaseConfig {
    host: str;
    port: i64;
    timeout: f64;
    ssl_enabled: bool;
}

struct ServerConfig {
    database: DatabaseConfig;
    max_connections: i64;
    request_handler: (str) -> str;  // Function type
}

let config: ServerConfig = ServerConfig {
    database: DatabaseConfig {
        host: "localhost",
        port: 5432,
        timeout: 30.0,
        ssl_enabled: true
    },
    max_connections: 100,
    request_handler: process_request
};
```

### Generic-like Patterns

```why
// Processing functions with explicit types
let string_processor: (str) -> str = normalize_string;
let number_processor: (i64) -> i64 = validate_number;
let array_processor: (&[i64]) -> &[i64] = sort_array;

// Data validation
let email_validator: (str) -> bool = is_valid_email;
let age_validator: (i64) -> bool = is_valid_age;
let password_validator: (str) -> bool = is_strong_password;
```

### Callback Systems

```why
struct EventSystem {
    on_click: () -> void;
    on_hover: (i64, i64) -> void;  // x, y coordinates
    on_key: (char) -> bool;        // key pressed, return if handled
}

let ui_events: EventSystem = EventSystem {
    on_click: handle_click,
    on_hover: \(x, y) => update_cursor(x, y),
    on_key: \(key) => process_key_input(key)
};
```

## When Type Annotations Are Required

### Function Signatures

```why
// Required: All function parameters and return types
fn process(input: str, count: i64): bool {
    // Implementation
    return true;
}
```

### Empty Collections

```why
// Required: Empty arrays need type specification
let empty_numbers: &[i64] = &[];
let empty_strings: &[str] = &[];
```

### Ambiguous Contexts

```why
// Required when type can't be inferred
let processor: (str) -> bool = get_validator();
let data: &[i64] = get_array_data();
```

## When Type Annotations Are Optional

### Simple Variable Assignments

```why
// Optional: Type can be inferred
let name = "Alice";     // Inferred as str
let age = 25;           // Inferred as i64
let price = 99.99;      // Inferred as f64
let active = true;      // Inferred as bool
```

### Return Statements

```why
// Type inferred from function signature
fn get_count(): i64 {
    return 42;  // Type inferred as i64
}
```

## Best Practices

### Clarity Over Brevity

```why
// Good: Clear intent with explicit types
let user_validator: (User) -> bool = validate_user_data;
let error_handler: (str) -> void = log_error;

// Acceptable: When type is obvious
let name = "Alice";
let count = 42;
```

### Complex Function Types

```why
// Good: Break down complex types
type ValidationFunction = (str) -> bool;
type ProcessingFunction = (str, ValidationFunction) -> str;

let process_with_validation: ProcessingFunction = complex_processor;

// Less readable: Inline complex types
let processor: (str, (str) -> bool) -> str = complex_processor;
```

### Consistent Style

```why
// Good: Consistent annotation style
let config: Config = load_config();
let validator: (str) -> bool = create_validator();
let processor: (Config) -> Result = process_config;

// Inconsistent: Mixed annotation styles
let config = load_config();  // No annotation
let validator: (str) -> bool = create_validator();  // With annotation
let processor = process_config;  // No annotation
```

### Documentation Through Types

```why
// Good: Types that document intent
let error_logger: (str, i64) -> void = log_error_with_code;
let data_transformer: (&[str]) -> &[str] = clean_and_normalize;
let connection_manager: (str, i64) -> bool = establish_connection;

// Less clear: Generic types
let func1: (str, i64) -> void = log_error_with_code;
let func2: (&[str]) -> &[str] = clean_and_normalize;
```