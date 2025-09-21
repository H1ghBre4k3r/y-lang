# Data Structures

This section covers implementing Y Lang's composite data types using Inkwell, including arrays, structs, tuples, and their memory layout and access patterns.

## Arrays

**Why arrays need careful layout**: LLVM arrays are contiguous memory blocks with compile-time known sizes, enabling efficient indexing and bounds checking while maintaining memory safety.

### Array Declaration and Initialization

Y Lang arrays map to LLVM array types with explicit element types and sizes:

```rust
use inkwell::context::Context;

let context = Context::create();
let module = context.create_module("arrays");
let builder = context.create_builder();

let i64_type = context.i64_type();
let array_type = i64_type.array_type(5); // [i64; 5]

// Function context for arrays
let fn_type = context.void_type().fn_type(&[], false);
let function = module.add_function("test_arrays", fn_type, None);
let entry_block = context.append_basic_block(function, "entry");
builder.position_at_end(entry_block);

// Declare array: let arr: [i64; 5];
let array_alloca = builder.build_alloca(array_type, "arr").unwrap();
```

**Generated LLVM IR:**
```llvm
define void @test_arrays() {
entry:
  %arr = alloca [5 x i64]
  ret void
}
```

**Implementation steps**:
1. Determine element type and array size at compile time
2. Create LLVM array type with `array_type(size)`
3. Allocate stack space with `build_alloca`
4. Handle initialization through element-by-element stores or constant arrays

### Array Initialization with Constants

```rust
// Initialize with constant values: let arr = [1, 2, 3, 4, 5];
let values = [1, 2, 3, 4, 5].map(|v| i64_type.const_int(v, false));
let array_constant = i64_type.const_array(&values);

let array_alloca = builder.build_alloca(array_type, "arr").unwrap();
builder.build_store(array_alloca, array_constant).unwrap();
```

**Generated LLVM IR:**
```llvm
%arr = alloca [5 x i64]
store [5 x i64] [i64 1, i64 2, i64 3, i64 4, i64 5], ptr %arr
```

### Array Element Access

Array indexing requires calculating element addresses with GEP (GetElementPtr):

```rust
// Access element: arr[2]
let zero = i64_type.const_int(0, false);      // Array base offset
let index = i64_type.const_int(2, false);     // Element index

let element_ptr = unsafe {
    builder.build_gep(array_type, array_alloca, &[zero, index], "elem_ptr").unwrap()
};

// Load element value
let element_value = builder.build_load(i64_type, element_ptr, "element").unwrap();
```

**Generated LLVM IR:**
```llvm
%elem_ptr = getelementptr [5 x i64], ptr %arr, i64 0, i64 2
%element = load i64, ptr %elem_ptr
```

**GEP indexing explanation**:
- First index (0): Navigate through the array allocation pointer
- Second index (2): Select the 3rd element (0-based indexing)
- Result: Pointer to the specific array element

### Array Element Assignment

```rust
// Assignment: arr[1] = 42;
let one_index = i64_type.const_int(1, false);
let new_value = i64_type.const_int(42, false);

let target_ptr = unsafe {
    builder.build_gep(array_type, array_alloca, &[zero, one_index], "target_ptr").unwrap()
};

builder.build_store(target_ptr, new_value).unwrap();
```

**Generated LLVM IR:**
```llvm
%target_ptr = getelementptr [5 x i64], ptr %arr, i64 0, i64 1
store i64 42, ptr %target_ptr
```

### Dynamic Array Indexing

For runtime-computed indices, bounds checking becomes important:

```rust
// arr[runtime_index]
let runtime_index = builder.build_load(i64_type, index_var, "runtime_idx").unwrap().into_int_value();

// Bounds check (optional but recommended)
let array_len = i64_type.const_int(5, false);
let in_bounds = builder.build_int_compare(
    IntPredicate::ULT,
    runtime_index,
    array_len,
    "in_bounds"
).unwrap();

// For safety, use conditional access or trap on out-of-bounds
let safe_index = builder.build_select(
    in_bounds,
    runtime_index,
    zero, // Default to index 0 if out of bounds
    "safe_index"
).unwrap();

let elem_ptr = unsafe {
    builder.build_gep(array_type, array_alloca, &[zero, safe_index.into_int_value()], "dyn_ptr").unwrap()
};
```

**Generated LLVM IR:**
```llvm
%runtime_idx = load i64, ptr %index_var
%in_bounds = icmp ult i64 %runtime_idx, 5
%safe_index = select i1 %in_bounds, i64 %runtime_idx, i64 0
%dyn_ptr = getelementptr [5 x i64], ptr %arr, i64 0, i64 %safe_index
```

## Structs

**Why structs need structured layout**: LLVM struct types enable efficient field access, proper alignment, and type safety for composite data, supporting both performance and correctness.

### Struct Type Definition

Y Lang structs map to LLVM struct types with named or anonymous fields:

```rust
// Y Lang: struct Point { x: i64, y: i64 }
let field_types = vec![i64_type.into(), i64_type.into()];
let point_type = context.struct_type(&field_types, false); // false = not packed
```

**Generated LLVM IR:**
```llvm
%Point = type { i64, i64 }
```

### Struct Variable Declaration and Initialization

```rust
// let point = Point { x: 10, y: 20 };
let point_alloca = builder.build_alloca(point_type, "point").unwrap();

// Initialize fields individually
let x_ptr = builder.build_struct_gep(point_type, point_alloca, 0, "x_ptr").unwrap();
let y_ptr = builder.build_struct_gep(point_type, point_alloca, 1, "y_ptr").unwrap();

let x_value = i64_type.const_int(10, false);
let y_value = i64_type.const_int(20, false);

builder.build_store(x_ptr, x_value).unwrap();
builder.build_store(y_ptr, y_value).unwrap();
```

**Generated LLVM IR:**
```llvm
%point = alloca { i64, i64 }
%x_ptr = getelementptr { i64, i64 }, ptr %point, i32 0, i32 0
%y_ptr = getelementptr { i64, i64 }, ptr %point, i32 0, i32 1
store i64 10, ptr %x_ptr
store i64 20, ptr %y_ptr
```

### Struct Constant Initialization

For compile-time known values, use struct constants:

```rust
// Efficient constant initialization
let field_values = vec![x_value.into(), y_value.into()];
let struct_constant = point_type.const_named_struct(&field_values);

let point_alloca = builder.build_alloca(point_type, "point").unwrap();
builder.build_store(point_alloca, struct_constant).unwrap();
```

**Generated LLVM IR:**
```llvm
%point = alloca { i64, i64 }
store { i64, i64 } { i64 10, i64 20 }, ptr %point
```

### Struct Field Access

```rust
// Access field: point.x
let x_ptr = builder.build_struct_gep(point_type, point_alloca, 0, "x_field").unwrap();
let x_value = builder.build_load(i64_type, x_ptr, "x_val").unwrap();

// Access field: point.y
let y_ptr = builder.build_struct_gep(point_type, point_alloca, 1, "y_field").unwrap();
let y_value = builder.build_load(i64_type, y_ptr, "y_val").unwrap();
```

**Generated LLVM IR:**
```llvm
%x_field = getelementptr { i64, i64 }, ptr %point, i32 0, i32 0
%x_val = load i64, ptr %x_field
%y_field = getelementptr { i64, i64 }, ptr %point, i32 0, i32 1
%y_val = load i64, ptr %y_field
```

### Struct Field Assignment

```rust
// Modify field: point.x = 42;
let x_ptr = builder.build_struct_gep(point_type, point_alloca, 0, "x_field").unwrap();
let new_x = i64_type.const_int(42, false);
builder.build_store(x_ptr, new_x).unwrap();
```

**Generated LLVM IR:**
```llvm
%x_field = getelementptr { i64, i64 }, ptr %point, i32 0, i32 0
store i64 42, ptr %x_field
```

### Nested Structs

Structs containing other structs require careful GEP indexing:

```rust
// Y Lang: struct Rectangle { top_left: Point, bottom_right: Point }
let rectangle_type = context.struct_type(&[
    point_type.into(),  // top_left field
    point_type.into(),  // bottom_right field
], false);

let rect_alloca = builder.build_alloca(rectangle_type, "rect").unwrap();

// Access nested field: rect.top_left.x
let top_left_ptr = builder.build_struct_gep(rectangle_type, rect_alloca, 0, "top_left").unwrap();
let x_ptr = builder.build_struct_gep(point_type, top_left_ptr, 0, "x_ptr").unwrap();
let x_value = builder.build_load(i64_type, x_ptr, "x_val").unwrap();
```

**Generated LLVM IR:**
```llvm
%Rectangle = type { { i64, i64 }, { i64, i64 } }
%rect = alloca { { i64, i64 }, { i64, i64 } }
%top_left = getelementptr { { i64, i64 }, { i64, i64 } }, ptr %rect, i32 0, i32 0
%x_ptr = getelementptr { i64, i64 }, ptr %top_left, i32 0, i32 0
%x_val = load i64, ptr %x_ptr
```

## Tuples

**Why tuples are like anonymous structs**: LLVM treats tuples as struct types without named fields, enabling efficient packing of heterogeneous data with positional access.

### Tuple Type Definition and Creation

```rust
// Y Lang: (i64, f64, bool)
let f64_type = context.f64_type();
let bool_type = context.bool_type();

let tuple_type = context.struct_type(&[
    i64_type.into(),
    f64_type.into(),
    bool_type.into(),
], false);

// Create tuple: (42, 3.14, true)
let tuple_alloca = builder.build_alloca(tuple_type, "tuple").unwrap();

// Initialize elements
let elem0_ptr = builder.build_struct_gep(tuple_type, tuple_alloca, 0, "elem0").unwrap();
let elem1_ptr = builder.build_struct_gep(tuple_type, tuple_alloca, 1, "elem1").unwrap();
let elem2_ptr = builder.build_struct_gep(tuple_type, tuple_alloca, 2, "elem2").unwrap();

builder.build_store(elem0_ptr, i64_type.const_int(42, false)).unwrap();
builder.build_store(elem1_ptr, f64_type.const_float(3.14)).unwrap();
builder.build_store(elem2_ptr, bool_type.const_int(1, false)).unwrap();
```

**Generated LLVM IR:**
```llvm
%tuple = alloca { i64, double, i1 }
%elem0 = getelementptr { i64, double, i1 }, ptr %tuple, i32 0, i32 0
%elem1 = getelementptr { i64, double, i1 }, ptr %tuple, i32 0, i32 1
%elem2 = getelementptr { i64, double, i1 }, ptr %tuple, i32 0, i32 2
store i64 42, ptr %elem0
store double 3.14, ptr %elem1
store i1 true, ptr %elem2
```

### Tuple Element Access

```rust
// Access tuple elements: tuple.0, tuple.1, tuple.2
let elem0_ptr = builder.build_struct_gep(tuple_type, tuple_alloca, 0, "get_0").unwrap();
let elem0_val = builder.build_load(i64_type, elem0_ptr, "val_0").unwrap();

let elem1_ptr = builder.build_struct_gep(tuple_type, tuple_alloca, 1, "get_1").unwrap();
let elem1_val = builder.build_load(f64_type, elem1_ptr, "val_1").unwrap();

let elem2_ptr = builder.build_struct_gep(tuple_type, tuple_alloca, 2, "get_2").unwrap();
let elem2_val = builder.build_load(bool_type, elem2_ptr, "val_2").unwrap();
```

**Generated LLVM IR:**
```llvm
%get_0 = getelementptr { i64, double, i1 }, ptr %tuple, i32 0, i32 0
%val_0 = load i64, ptr %get_0
%get_1 = getelementptr { i64, double, i1 }, ptr %tuple, i32 0, i32 1
%val_1 = load double, ptr %get_1
%get_2 = getelementptr { i64, double, i1 }, ptr %tuple, i32 0, i32 2
%val_2 = load i1, ptr %get_2
```

### Tuple Destructuring

Y Lang tuple destructuring can be implemented through multiple GEP operations:

```rust
// Y Lang: let (x, y, flag) = tuple;
let x_alloca = builder.build_alloca(i64_type, "x").unwrap();
let y_alloca = builder.build_alloca(f64_type, "y").unwrap();
let flag_alloca = builder.build_alloca(bool_type, "flag").unwrap();

// Extract and store each element
let elem0_ptr = builder.build_struct_gep(tuple_type, tuple_alloca, 0, "extract_0").unwrap();
let elem0_val = builder.build_load(i64_type, elem0_ptr, "x_val").unwrap();
builder.build_store(x_alloca, elem0_val).unwrap();

let elem1_ptr = builder.build_struct_gep(tuple_type, tuple_alloca, 1, "extract_1").unwrap();
let elem1_val = builder.build_load(f64_type, elem1_ptr, "y_val").unwrap();
builder.build_store(y_alloca, elem1_val).unwrap();

let elem2_ptr = builder.build_struct_gep(tuple_type, tuple_alloca, 2, "extract_2").unwrap();
let elem2_val = builder.build_load(bool_type, elem2_ptr, "flag_val").unwrap();
builder.build_store(flag_alloca, elem2_val).unwrap();
```

**Generated LLVM IR:**
```llvm
%x = alloca i64
%y = alloca double
%flag = alloca i1
%extract_0 = getelementptr { i64, double, i1 }, ptr %tuple, i32 0, i32 0
%x_val = load i64, ptr %extract_0
store i64 %x_val, ptr %x
%extract_1 = getelementptr { i64, double, i1 }, ptr %tuple, i32 0, i32 1
%y_val = load double, ptr %extract_1
store double %y_val, ptr %y
%extract_2 = getelementptr { i64, double, i1 }, ptr %tuple, i32 0, i32 2
%flag_val = load i1, ptr %extract_2
store i1 %flag_val, ptr %flag
```

## Memory Layout and Alignment

**Why layout matters**: Understanding memory layout enables performance optimization and proper alignment for different architectures.

### Struct Padding and Alignment

```rust
// Struct with different-sized fields
let mixed_struct_type = context.struct_type(&[
    context.i8_type().into(),   // 1 byte
    i64_type.into(),            // 8 bytes
    context.i16_type().into(),  // 2 bytes
], false); // Natural alignment

// Packed struct (no padding)
let packed_struct_type = context.struct_type(&[
    context.i8_type().into(),
    i64_type.into(),
    context.i16_type().into(),
], true); // Packed alignment
```

**Generated LLVM IR:**
```llvm
; Natural alignment (with padding)
%MixedStruct = type { i8, i64, i16 }  ; Likely 24 bytes with padding

; Packed alignment (no padding)
%PackedStruct = type <{ i8, i64, i16 }>  ; Exactly 11 bytes
```

### Array of Structs

Combining arrays and structs for complex data layouts:

```rust
// Array of points: [Point; 3]
let point_array_type = point_type.array_type(3);
let points_alloca = builder.build_alloca(point_array_type, "points").unwrap();

// Access specific point: points[1].x
let zero = i64_type.const_int(0, false);
let index_1 = i64_type.const_int(1, false);

let point_ptr = unsafe {
    builder.build_gep(point_array_type, points_alloca, &[zero, index_1], "point_1").unwrap()
};

let x_ptr = builder.build_struct_gep(point_type, point_ptr, 0, "point_1_x").unwrap();
let x_value = builder.build_load(i64_type, x_ptr, "x_val").unwrap();
```

**Generated LLVM IR:**
```llvm
%points = alloca [3 x { i64, i64 }]
%point_1 = getelementptr [3 x { i64, i64 }], ptr %points, i64 0, i64 1
%point_1_x = getelementptr { i64, i64 }, ptr %point_1, i32 0, i32 0
%x_val = load i64, ptr %point_1_x
```

## Advanced Data Structure Patterns

### Generic-Like Structs

Using LLVM's type system to simulate generics:

```rust
// Different instantiations of "generic" container
fn create_container_type<'ctx>(context: &'ctx Context, element_type: BasicTypeEnum<'ctx>) -> StructType<'ctx> {
    context.struct_type(&[
        element_type,                           // data
        context.i64_type().into(),             // size
        context.i64_type().into(),             // capacity
    ], false)
}

let int_container = create_container_type(&context, i64_type.into());
let float_container = create_container_type(&context, f64_type.into());
```

### Optional Types (Sum Types)

Implementing Option<T> using tagged unions:

```rust
// Option<i64> as tagged union
let option_type = context.struct_type(&[
    context.i8_type().into(),   // Tag: 0 = None, 1 = Some
    i64_type.into(),            // Value (only valid if tag == 1)
], false);

// Create Some(42)
let some_42 = builder.build_alloca(option_type, "some_42").unwrap();

let tag_ptr = builder.build_struct_gep(option_type, some_42, 0, "tag_ptr").unwrap();
let val_ptr = builder.build_struct_gep(option_type, some_42, 1, "val_ptr").unwrap();

builder.build_store(tag_ptr, context.i8_type().const_int(1, false)).unwrap(); // Some
builder.build_store(val_ptr, i64_type.const_int(42, false)).unwrap();
```

**Generated LLVM IR:**
```llvm
%Option_i64 = type { i8, i64 }
%some_42 = alloca { i8, i64 }
%tag_ptr = getelementptr { i8, i64 }, ptr %some_42, i32 0, i32 0
%val_ptr = getelementptr { i8, i64 }, ptr %some_42, i32 0, i32 1
store i8 1, ptr %tag_ptr
store i64 42, ptr %val_ptr
```

### Dynamic Arrays (Vectors)

Implementing growable arrays with heap allocation:

```rust
// Vector representation: { ptr, length, capacity }
let ptr_type = context.ptr_type(Default::default());
let vector_type = context.struct_type(&[
    ptr_type.into(),        // data pointer
    i64_type.into(),        // length
    i64_type.into(),        // capacity
], false);

let vec_alloca = builder.build_alloca(vector_type, "vector").unwrap();

// Initialize empty vector
let null_ptr = ptr_type.const_null();
let zero_len = i64_type.const_zero();
let zero_cap = i64_type.const_zero();

let data_ptr = builder.build_struct_gep(vector_type, vec_alloca, 0, "data_field").unwrap();
let len_ptr = builder.build_struct_gep(vector_type, vec_alloca, 1, "len_field").unwrap();
let cap_ptr = builder.build_struct_gep(vector_type, vec_alloca, 2, "cap_field").unwrap();

builder.build_store(data_ptr, null_ptr).unwrap();
builder.build_store(len_ptr, zero_len).unwrap();
builder.build_store(cap_ptr, zero_cap).unwrap();
```

**Generated LLVM IR:**
```llvm
%Vector = type { ptr, i64, i64 }
%vector = alloca { ptr, i64, i64 }
%data_field = getelementptr { ptr, i64, i64 }, ptr %vector, i32 0, i32 0
%len_field = getelementptr { ptr, i64, i64 }, ptr %vector, i32 0, i32 1
%cap_field = getelementptr { ptr, i64, i64 }, ptr %vector, i32 0, i32 2
store ptr null, ptr %data_field
store i64 0, ptr %len_field
store i64 0, ptr %cap_field
```

## Error Handling and Validation

### Bounds Checking for Data Structures

```rust
fn safe_array_access<'ctx>(
    builder: &Builder<'ctx>,
    array_ptr: PointerValue<'ctx>,
    array_type: ArrayType<'ctx>,
    index: IntValue<'ctx>,
    element_type: BasicTypeEnum<'ctx>
) -> Result<BasicValueEnum<'ctx>, String> {
    let array_len = array_type.len() as u64;
    let len_const = element_type.into_int_type().const_int(array_len, false);

    // Runtime bounds check
    let in_bounds = builder.build_int_compare(
        IntPredicate::ULT,
        index,
        len_const,
        "bounds_check"
    ).map_err(|e| format!("Failed bounds check: {}", e))?;

    // Could add trap or error handling here
    let zero = element_type.into_int_type().const_zero();
    let elem_ptr = unsafe {
        builder.build_gep(array_type, array_ptr, &[zero, index], "safe_elem")
            .map_err(|e| format!("GEP failed: {}", e))?
    };

    builder.build_load(element_type, elem_ptr, "safe_load")
        .map_err(|e| format!("Load failed: {}", e))
}
```

### Type Safety for Struct Fields

```rust
fn safe_struct_field_access<'ctx>(
    builder: &Builder<'ctx>,
    struct_ptr: PointerValue<'ctx>,
    struct_type: StructType<'ctx>,
    field_index: u32,
    expected_type: BasicTypeEnum<'ctx>
) -> Result<BasicValueEnum<'ctx>, String> {
    let field_types = struct_type.get_field_types();

    if field_index as usize >= field_types.len() {
        return Err(format!("Field index {} out of bounds", field_index));
    }

    let actual_type = field_types[field_index as usize];
    if actual_type != expected_type {
        return Err(format!("Type mismatch: expected {:?}, got {:?}", expected_type, actual_type));
    }

    let field_ptr = builder.build_struct_gep(struct_type, struct_ptr, field_index, "field")
        .map_err(|e| format!("Struct GEP failed: {}", e))?;

    builder.build_load(expected_type, field_ptr, "field_val")
        .map_err(|e| format!("Field load failed: {}", e))
}
```

## Performance Optimization Strategies

### Minimizing Memory Operations

```rust
// Efficient: Load struct once, extract fields as needed
let struct_val = builder.build_load(point_type, point_alloca, "point_val").unwrap();
let x_val = builder.build_extract_value(struct_val.into_struct_value(), 0, "x").unwrap();
let y_val = builder.build_extract_value(struct_val.into_struct_value(), 1, "y").unwrap();

// Less efficient: Multiple GEP + load operations
let x_ptr = builder.build_struct_gep(point_type, point_alloca, 0, "x_ptr").unwrap();
let x_val_slow = builder.build_load(i64_type, x_ptr, "x_slow").unwrap();
```

### Prefer Stack Allocation When Possible

```rust
// Good: Stack allocation for known-size data
let local_array = builder.build_alloca(array_type, "local").unwrap();

// Only use heap allocation when necessary (dynamic size, large data, etc.)
```

### Leverage LLVM's Optimization Passes

```rust
// LLVM can optimize away unnecessary loads/stores and GEP chains
// Structure your code to enable these optimizations:
// 1. Use consistent naming
// 2. Avoid redundant memory operations
// 3. Let LLVM handle layout optimization
```

This comprehensive coverage of data structures provides the foundation for implementing Y Lang's composite types in LLVM, emphasizing proper memory layout, type safety, and performance considerations for arrays, structs, tuples, and advanced patterns.
