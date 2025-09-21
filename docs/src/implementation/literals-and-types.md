# Literals and Types

This section covers implementing Y Lang's literal values and type system using Inkwell, focusing on the conceptual mapping between Y Lang constructs and LLVM representations.

## Primitive Type Mapping

Y Lang's type system maps directly to LLVM's type system, with each Y Lang type having a clear LLVM counterpart.

### Integer Literals

**Why integers need explicit typing**: LLVM requires all values to have concrete types. Y Lang's `i64` maps to LLVM's `i64` type, ensuring platform independence and consistent overflow behavior.

```rust
use inkwell::context::Context;

let context = Context::create();
let i64_type = context.i64_type();

// Creating integer constants
let zero = i64_type.const_int(0, false);     // false = unsigned interpretation
let positive = i64_type.const_int(42, false);
let negative = i64_type.const_int(-10i64 as u64, true); // true = signed interpretation
```

**Generated LLVM IR:**
```llvm
%1 = i64 0
%2 = i64 42
%3 = i64 -10
```

**Implementation steps**:
1. Get the target integer type from context
2. Use `const_int()` with appropriate signedness flag
3. Handle negative values by casting to unsigned representation

### Floating Point Literals

**Why separate float handling**: LLVM distinguishes between integer and floating-point arithmetic to enable proper IEEE 754 compliance and optimization.

```rust
let f64_type = context.f64_type();

// Creating float constants
let pi = f64_type.const_float(3.14159);
let euler = f64_type.const_float(2.71828);
let negative_float = f64_type.const_float(-1.5);
```

**Generated LLVM IR:**
```llvm
%1 = double 3.14159
%2 = double 2.71828
%3 = double -1.5
```

**IEEE 754 considerations**:
- LLVM uses IEEE 754 double precision for `f64`
- Special values (NaN, infinity) are handled automatically
- Exact decimal representation may differ due to binary encoding

### Boolean Literals

**Why single-bit booleans**: LLVM's `i1` type enables efficient boolean operations and memory usage, with clear true/false semantics.

```rust
let bool_type = context.bool_type();

// Creating boolean constants
let true_val = bool_type.const_int(1, false);   // true = 1
let false_val = bool_type.const_int(0, false);  // false = 0

// Alternative using LLVM's built-in constants
let true_val_alt = bool_type.const_all_ones();  // all bits set = true
let false_val_alt = bool_type.const_zero();     // all bits clear = false
```

**Generated LLVM IR:**
```llvm
%1 = i1 true
%2 = i1 false
```

**Boolean semantics**:
- Only `0` is false, all other values are true
- Comparisons and logical operations return `i1` values
- Can be promoted to larger integer types when needed

### Character Literals

**Why `i8` for characters**: Y Lang treats characters as UTF-8 bytes, allowing for efficient string composition and manipulation.

```rust
let i8_type = context.i8_type();

// Creating character constants
let char_a = i8_type.const_int(b'a' as u64, false);
let char_newline = i8_type.const_int(b'\n' as u64, false);
let char_unicode = i8_type.const_int(0xC3, false); // First byte of UTF-8 sequence
```

**Generated LLVM IR:**
```llvm
%1 = i8 97      ; 'a'
%2 = i8 10      ; '\n'
%3 = i8 195     ; 0xC3
```

**UTF-8 handling considerations**:
- Single-byte ASCII characters map directly
- Multi-byte Unicode requires sequence handling
- String operations must respect UTF-8 boundaries

## String Literals

**Why strings are complex**: LLVM doesn't have a built-in string type. Strings are implemented as arrays of bytes with additional metadata.

### Simple String Constants

```rust
// Method 1: Global string pointer (most common)
let hello_str = "Hello, World!";
let hello_global = builder.build_global_string_ptr(hello_str, "hello").unwrap();

// Method 2: String as array constant
let hello_bytes = hello_str.as_bytes();
let i8_type = context.i8_type();
let string_array_type = i8_type.array_type(hello_bytes.len() as u32);

let char_values: Vec<_> = hello_bytes.iter()
    .map(|&b| i8_type.const_int(b as u64, false))
    .collect();
let string_constant = i8_type.const_array(&char_values);
```

**Generated LLVM IR:**
```llvm
; Method 1: Global string pointer
@hello = private constant [14 x i8] c"Hello, World!\00"

; Method 2: Array constant
%1 = [13 x i8] [i8 72, i8 101, i8 108, i8 108, i8 111, i8 44, i8 32,
                i8 87, i8 111, i8 114, i8 108, i8 100, i8 33]
```

### String with Length Information

Y Lang strings likely need length information for bounds checking and iteration:

```rust
// String representation: { ptr, length }
let ptr_type = context.ptr_type(Default::default());
let string_struct_type = context.struct_type(&[
    ptr_type.into(),     // data pointer
    i64_type.into(),     // length
], false);

// Create string literal with metadata
let hello_ptr = builder.build_global_string_ptr("Hello", "hello_data").unwrap();
let hello_len = i64_type.const_int(5, false);

// Allocate string struct
let string_alloca = builder.build_alloca(string_struct_type, "string_literal").unwrap();

// Set data pointer
let ptr_field = builder.build_struct_gep(string_struct_type, string_alloca, 0, "data_ptr").unwrap();
builder.build_store(ptr_field, hello_ptr).unwrap();

// Set length
let len_field = builder.build_struct_gep(string_struct_type, string_alloca, 1, "len_ptr").unwrap();
builder.build_store(len_field, hello_len).unwrap();
```

**Generated LLVM IR:**
```llvm
@hello_data = private constant [6 x i8] c"Hello\00"

%string_literal = alloca { ptr, i64 }
%data_ptr = getelementptr { ptr, i64 }, ptr %string_literal, i32 0, i32 0
store ptr @hello_data, ptr %data_ptr
%len_ptr = getelementptr { ptr, i64 }, ptr %string_literal, i32 0, i32 1
store i64 5, ptr %len_ptr
```

## Void Type

**Why void exists**: Represents functions and expressions that don't return meaningful values, enabling proper type checking and optimization.

```rust
let void_type = context.void_type();

// Used in function signatures
let void_fn_type = void_type.fn_type(&[], false);

// Used in return statements
builder.build_return(None).unwrap(); // returns void
```

**Generated LLVM IR:**
```llvm
define void @some_function() {
  ret void
}
```

## Type Conversions

Y Lang requires explicit type conversions between different primitive types. LLVM provides specific instructions for each conversion type.

### Integer Conversions

```rust
// Widening (safe)
let i32_type = context.i32_type();
let small_int = i32_type.const_int(42, false);
let widened = builder.build_int_z_extend(small_int, i64_type, "widened").unwrap();

// Narrowing (potentially unsafe)
let large_int = i64_type.const_int(1000, false);
let narrowed = builder.build_int_truncate(large_int, i32_type, "narrowed").unwrap();

// Signed vs unsigned interpretation
let signed_val = builder.build_int_s_extend(small_int, i64_type, "signed_ext").unwrap();
```

**Generated LLVM IR:**
```llvm
%widened = zext i32 42 to i64
%narrowed = trunc i64 1000 to i32
%signed_ext = sext i32 42 to i64
```

### Float-Integer Conversions

```rust
// Float to integer
let float_val = f64_type.const_float(3.14);
let float_to_int = builder.build_float_to_signed_int(float_val, i64_type, "f_to_i").unwrap();

// Integer to float
let int_val = i64_type.const_int(42, false);
let int_to_float = builder.build_signed_int_to_float(int_val, f64_type, "i_to_f").unwrap();
```

**Generated LLVM IR:**
```llvm
%f_to_i = fptosi double 3.14 to i64
%i_to_f = sitofp i64 42 to double
```

### Boolean Conversions

```rust
// Integer to boolean (zero = false, nonzero = true)
let int_val = i64_type.const_int(5, false);
let zero = i64_type.const_zero();
let to_bool = builder.build_int_compare(
    IntPredicate::NE,
    int_val,
    zero,
    "to_bool"
).unwrap();

// Boolean to integer
let bool_val = bool_type.const_int(1, false);
let to_int = builder.build_int_z_extend(bool_val, i64_type, "to_int").unwrap();
```

**Generated LLVM IR:**
```llvm
%to_bool = icmp ne i64 5, 0
%to_int = zext i1 true to i64
```

## Advanced Type Concepts

### Type Equivalence and Compatibility

**Why type checking matters**: LLVM performs strict type checking. Operations between incompatible types will fail at IR generation time.

```rust
// These are the same type (interned by context)
let i64_a = context.i64_type();
let i64_b = context.i64_type();
assert_eq!(i64_a, i64_b); // Same type instance

// These are different types
let i32_type = context.i32_type();
// builder.build_int_add(i64_val, i32_val, "error"); // Would fail!
```

### Type Size and Alignment

```rust
use inkwell::targets::{TargetData, TargetMachine};

// Get type sizes (requires target data)
let target_machine = Target::from_name("x86-64").unwrap()
    .create_target_machine(/* ... */).unwrap();
let target_data = target_machine.get_target_data();

let i64_size = i64_type.size_of().unwrap();
let string_size = string_struct_type.size_of().unwrap();
```

### Opaque Pointers

Modern LLVM uses opaque pointers that don't encode pointee types:

```rust
// All pointers are the same type
let ptr_type = context.ptr_type(Default::default());

// Type information comes from load/store operations
let loaded_i64 = builder.build_load(i64_type, ptr, "loaded").unwrap();
let loaded_f64 = builder.build_load(f64_type, ptr, "loaded_f").unwrap();
```

## Implementation Patterns

### Literal Processing Pipeline

1. **Lexical Analysis**: Recognize literal tokens (numbers, strings, booleans)
2. **Type Inference**: Determine appropriate LLVM type
3. **Constant Creation**: Generate LLVM constant values
4. **Type Checking**: Verify compatibility with context

### Dynamic vs Static Typing

Y Lang appears to use static typing with inference:

```rust
// Known at compile time
let static_val = i64_type.const_int(42, false);

// Runtime value (from variable, computation)
let runtime_val = builder.build_load(i64_type, some_ptr, "runtime").unwrap();

// Mixed operations
let result = builder.build_int_add(static_val, runtime_val, "mixed").unwrap();
```

### Error Handling for Type Operations

```rust
// Safe type checking before operations
fn safe_add(builder: &Builder, left: IntValue, right: IntValue, name: &str) -> Result<IntValue, String> {
    if left.get_type() != right.get_type() {
        return Err(format!("Type mismatch: {:?} vs {:?}", left.get_type(), right.get_type()));
    }

    builder.build_int_add(left, right, name)
        .map_err(|e| format!("Failed to build add: {}", e))
}
```

## Performance Considerations

### Constant Folding

LLVM automatically performs constant folding:

```rust
// These will be computed at compile time
let a = i64_type.const_int(10, false);
let b = i64_type.const_int(20, false);
let sum = a.const_add(b); // Computed immediately, not at runtime
```

### Type Interning

The Context automatically interns types, making type operations efficient:

```rust
// Multiple calls return the same type instance
let type1 = context.i64_type();
let type2 = context.i64_type();
// type1 and type2 are the same object in memory
```

### Memory Layout Optimization

```rust
// Packed structs save memory but may hurt performance
let packed_struct = context.struct_type(&[
    i8_type.into(),
    i64_type.into(),
], true); // true = packed

// Regular structs have natural alignment
let aligned_struct = context.struct_type(&[
    i8_type.into(),
    i64_type.into(),
], false); // false = natural alignment
```

This comprehensive coverage of literals and types provides the foundation for implementing Y Lang's type system in LLVM, focusing on the conceptual reasoning behind each choice and common implementation patterns.