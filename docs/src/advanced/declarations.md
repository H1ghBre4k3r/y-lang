# External Declarations

External declarations in Y allow you to interface with functions and values implemented outside of Y, such as C library functions, system calls, or runtime-provided functionality. This enables Y programs to leverage existing libraries and system capabilities.

## Declaration Syntax

Use the `declare` keyword to declare external functions and values:

```why
declare function_name: (param_types) -> return_type;
declare variable_name: Type;
```

## Function Declarations

### Basic Function Declarations

```why
declare printf: (str) -> void;
declare malloc: (i64) -> void;
declare strlen: (str) -> i64;
declare sqrt: (f64) -> f64;
```

### Using Declared Functions

Once declared, external functions can be used like regular Y functions:

```why
declare printf: (str) -> void;

fn main(): void {
    printf("Hello from Y!\n");
}
```

## Examples from Y Code

### System Functions

```why
declare printf: (str) -> void;

fn baz(x: i64): i64 {
    let intermediate = x * 2;
    return intermediate;
}

fn main(): i64 {
    printf("Foo\n");  // Using external printf
    let x = 12;
    let a = baz(x);
    return x + a;
}
```

### Variable Declarations

```why
struct System {}

instance System {
    fn answer(): i64 {
        42
    }

    declare print(i64): void;  // External method
}

declare Sys: System;  // External global variable

fn main(): void {
    Sys.print(Sys.answer());  // Using external system
}
```

## Method Declarations

External methods can be declared within instance blocks:

```why
struct TestStruct {
    x: i64;
}

instance TestStruct {
    fn get_x(): i64 {
        this.x
    }

    declare get_id(): i64;  // External method implementation
}

instance str {
    declare len(): i64;     // External string length
}

instance i64 {
    declare add(i64): i64;  // External arithmetic operation
}
```

## Common Use Cases

### Standard Library Functions

```why
// C standard library functions
declare malloc: (i64) -> void;
declare free: (void) -> void;
declare memcpy: (void, void, i64) -> void;
declare strcmp: (str, str) -> i64;

// Math library functions
declare sin: (f64) -> f64;
declare cos: (f64) -> f64;
declare sqrt: (f64) -> f64;
declare pow: (f64, f64) -> f64;
```

### System Calls

```why
// File operations
declare open: (str, i64) -> i64;
declare read: (i64, void, i64) -> i64;
declare write: (i64, void, i64) -> i64;
declare close: (i64) -> i64;

// Process operations
declare getpid: () -> i64;
declare exit: (i64) -> void;
```

### Custom Runtime Functions

```why
// Custom Y runtime functions
declare gc_collect: () -> void;
declare debug_print: (str) -> void;
declare get_timestamp: () -> i64;
declare allocate_array: (i64) -> void;
```

## Integration Patterns

### Wrapper Functions

Create Y functions that wrap external declarations for better ergonomics:

```why
declare c_strlen: (str) -> i64;
declare c_strcmp: (str, str) -> i64;

fn string_length(s: str): i64 {
    c_strlen(s)
}

fn strings_equal(a: str, b: str): bool {
    c_strcmp(a, b) == 0
}
```

### Error Handling

```why
declare c_malloc: (i64) -> void;
declare c_free: (void) -> void;

fn safe_allocate(size: i64): void {
    if (size > 0) {
        return c_malloc(size);
    } else {
        return null;  // Or appropriate error handling
    }
}
```

### Type-Safe Interfaces

```why
// Raw external functions
declare raw_read_file: (str) -> void;
declare raw_write_file: (str, void) -> i64;

// Type-safe wrappers
fn read_text_file(filename: str): str {
    // Implementation that ensures str return type
    return convert_to_string(raw_read_file(filename));
}

fn write_text_file(filename: str, content: str): bool {
    let result = raw_write_file(filename, string_to_bytes(content));
    return result >= 0;
}
```

## Platform-Specific Declarations

### Unix/Linux

```why
declare fork: () -> i64;
declare exec: (str, &[str]) -> i64;
declare waitpid: (i64, void, i64) -> i64;
declare signal: (i64, void) -> void;
```

### Windows

```why
declare CreateProcess: (str, str, void, void, bool, i64, void, str, void, void) -> bool;
declare CloseHandle: (void) -> bool;
declare GetLastError: () -> i64;
```

## Real-World Integration Example

```why
// Graphics library integration
declare sdl_init: (i64) -> i64;
declare sdl_create_window: (str, i64, i64, i64, i64, i64) -> void;
declare sdl_destroy_window: (void) -> void;
declare sdl_quit: () -> void;

struct Window {
    title: str;
    width: i64;
    height: i64;
    handle: void;  // Opaque handle
}

fn create_window(title: str, width: i64, height: i64): Window {
    let handle = sdl_create_window(title, 100, 100, width, height, 0);
    Window {
        title: title,
        width: width,
        height: height,
        handle: handle
    }
}

fn main(): i64 {
    if (sdl_init(0x20) < 0) {
        return 1;  // Error
    }

    let window = create_window("Y Lang App", 800, 600);

    // Main loop would go here

    sdl_destroy_window(window.handle);
    sdl_quit();
    return 0;
}
```

## Network Programming

```why
// Socket operations
declare socket: (i64, i64, i64) -> i64;
declare bind: (i64, void, i64) -> i64;
declare listen: (i64, i64) -> i64;
declare accept: (i64, void, void) -> i64;
declare send: (i64, void, i64, i64) -> i64;
declare recv: (i64, void, i64, i64) -> i64;

struct Server {
    socket_fd: i64;
    port: i64;
}

fn create_server(port: i64): Server {
    let sock = socket(2, 1, 0);  // AF_INET, SOCK_STREAM, 0
    // Additional setup would go here
    Server {
        socket_fd: sock,
        port: port
    }
}
```

## Best Practices

### Clear Naming

```why
// Good: Clear external function names
declare c_printf: (str) -> void;
declare libc_malloc: (i64) -> void;
declare posix_open: (str, i64) -> i64;

// Less clear: Ambiguous names
declare func1: (str) -> void;
declare ext_call: (i64) -> void;
```

### Documentation

```why
// File system operations from POSIX
declare open: (str, i64) -> i64;    // Open file, returns file descriptor
declare read: (i64, void, i64) -> i64;  // Read from fd, returns bytes read
declare close: (i64) -> i64;        // Close file descriptor

// Graphics library bindings
declare gl_clear: (i64) -> void;    // Clear OpenGL buffers
declare gl_swap: () -> void;        // Swap front/back buffers
```

### Error Handling

```why
declare system_call: (i64) -> i64;

fn safe_system_call(arg: i64): bool {
    let result = system_call(arg);
    return result >= 0;  // Assuming negative values indicate errors
}
```

### Type Safety

```why
// Instead of raw pointers, use appropriate Y types when possible
declare unsafe_memory_op: (void, i64) -> void;

// Wrap in safer interface
fn safe_memory_operation(data: &[i64]): bool {
    if (data.length() > 0) {
        unsafe_memory_op(data_pointer(data), data.length());
        return true;
    } else {
        return false;
    }
}
```

## Linking and Compilation

External declarations typically require:
1. **Header inclusion** - Corresponding C headers during compilation
2. **Library linking** - Linking against libraries that provide the implementations
3. **ABI compatibility** - Ensuring parameter and return types match the external interface

Example build command:
```bash
yc program.why -lc -lm -lpthread  # Link against libc, libm, pthread
```