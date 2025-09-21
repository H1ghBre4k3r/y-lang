# Inkwell Reference Guide

This guide explains how to implement Y Lang constructs using Inkwell, focusing on the conceptual reasoning behind LLVM patterns rather than just implementation details. Each section provides simplified examples and shows the resulting LLVM IR.

## Table of Contents

- [Foundation Concepts](#foundation-concepts)
- [Basic Types and Literals](#basic-types-and-literals)
- [Variables and Memory](#variables-and-memory)
- [Binary Operations](#binary-operations)
- [Functions](#functions)
- [Control Flow](#control-flow)
- [Data Structures](#data-structures)
- [Advanced Constructs](#advanced-constructs)

## Foundation Concepts

### Why Inkwell Abstractions Matter

LLVM operates on three key abstractions that Inkwell exposes:

- **Context**: The global LLVM state container
- **Module**: A compilation unit containing functions and globals
- **Builder**: The instruction generator positioned within basic blocks

**Why this separation?** LLVM separates concerns: the Context manages types and constants, the Module organizes code units, and the Builder handles the sequential nature of instruction generation.

```rust
use inkwell::context::Context;

let context = Context::create();
let module = context.create_module("my_module");
let builder = context.create_builder();
```

### Type System Mapping

Y Lang types map to LLVM types through a conceptual bridge:

| Y Lang Type | LLVM Type | Reasoning |
|-------------|-----------|-----------|
| `i64` | `i64` | Direct mapping for integers |
| `f64` | `double` | IEEE 754 double precision |
| `bool` | `i1` | Single bit for true/false |
| `char` | `i8` | UTF-8 byte representation |
| `string` | `ptr` | Pointer to memory region |
| `struct` | `%struct.name` | Aggregate type |
| `array` | `[N x type]` | Fixed-size homogeneous collection |

**Why pointers for strings?** LLVM doesn't have high-level string types. Strings are memory regions accessed through pointers, allowing for dynamic sizing and efficient passing.

## Basic Types and Literals

### Integer Literals

**Why**: LLVM represents integers as typed constant values. The type system ensures operations are well-defined.

```rust
use inkwell::context::Context;

let context = Context::create();
let i64_type = context.i64_type();
let forty_two = i64_type.const_int(42, false); // false = unsigned
```

**Generated LLVM IR:**
```llvm
%1 = i64 42
```

### Floating Point Literals

**Why**: LLVM uses IEEE 754 representation. The type determines precision and operations available.

```rust
let f64_type = context.f64_type();
let pi = f64_type.const_float(3.14159);
```

**Generated LLVM IR:**
```llvm
%1 = double 3.14159
```

### Boolean Literals

**Why**: Booleans are single bits in LLVM, enabling efficient storage and logical operations.

```rust
let bool_type = context.bool_type();
let true_val = bool_type.const_int(1, false);
let false_val = bool_type.const_int(0, false);
```

**Generated LLVM IR:**
```llvm
%1 = i1 true
%2 = i1 false
```

## Variables and Memory

### Variable Declaration and Storage

**Why**: LLVM uses `alloca` for stack allocation. This creates memory slots that can be loaded from and stored to, enabling mutable variables.

```rust
let builder = context.create_builder();
let i64_type = context.i64_type();

// Allocate stack space for a variable
let var_alloca = builder.build_alloca(i64_type, "my_var").unwrap();

// Store a value
let value = i64_type.const_int(100, false);
builder.build_store(var_alloca, value).unwrap();

// Load the value
let loaded = builder.build_load(i64_type, var_alloca, "loaded_val").unwrap();
```

**Generated LLVM IR:**
```llvm
%my_var = alloca i64
store i64 100, ptr %my_var
%loaded_val = load i64, ptr %my_var
```

**Implementation steps for variables:**
1. **Allocation**: Use `build_alloca` to reserve stack space
2. **Storage**: Use `build_store` to write values
3. **Loading**: Use `build_load` to read values
4. **Scope management**: Track allocations in symbol tables

### Variable Assignment

**Why**: Assignment reuses existing allocation but stores new values. No new memory allocation needed.

```rust
// Assuming var_alloca exists from previous declaration
let new_value = i64_type.const_int(200, false);
builder.build_store(var_alloca, new_value).unwrap();
```

**Generated LLVM IR:**
```llvm
store i64 200, ptr %my_var
```

## Binary Operations

### Arithmetic Operations

**Why**: LLVM provides separate instructions for different numeric types and signedness. This enables optimization and correct overflow behavior.

```rust
// Integer addition
let left = i64_type.const_int(10, false);
let right = i64_type.const_int(20, false);
let sum = builder.build_int_add(left, right, "sum").unwrap();

// Float addition
let left_f = f64_type.const_float(10.5);
let right_f = f64_type.const_float(20.3);
let sum_f = builder.build_float_add(left_f, right_f, "sum_f").unwrap();
```

**Generated LLVM IR:**
```llvm
%sum = add i64 10, 20
%sum_f = fadd double 10.5, 20.3
```

### Comparison Operations

**Why**: Comparisons return `i1` (boolean) values and have different semantics for integers vs floats (NaN handling).

```rust
use inkwell::IntPredicate;

let cmp = builder.build_int_compare(
    IntPredicate::EQ,
    left,
    right,
    "equal"
).unwrap();
```

**Generated LLVM IR:**
```llvm
%equal = icmp eq i64 %left, %right
```

**Implementation steps for binary operations:**
1. **Type checking**: Ensure operands have compatible types
2. **Operation selection**: Choose appropriate LLVM instruction (add vs fadd)
3. **Result naming**: Provide meaningful names for debugging

## Functions

### Function Declaration

**Why**: LLVM functions define entry points and calling conventions. The type signature determines parameter and return handling.

```rust
use inkwell::types::BasicMetadataTypeEnum;

// Define function type: i64 add(i64, i64)
let param_types = vec![
    BasicMetadataTypeEnum::IntType(i64_type),
    BasicMetadataTypeEnum::IntType(i64_type),
];
let fn_type = i64_type.fn_type(&param_types, false); // false = not variadic

// Create function
let function = module.add_function("add", fn_type, None);
```

**Generated LLVM IR:**
```llvm
define i64 @add(i64 %0, i64 %1) {
}
```

### Function Body and Parameters

**Why**: Basic blocks organize control flow. Parameters need allocation for mutability (following Y Lang semantics).

```rust
// Create entry basic block
let entry_block = context.append_basic_block(function, "entry");
builder.position_at_end(entry_block);

// Access parameters and create allocations
let param1 = function.get_nth_param(0).unwrap().into_int_value();
let param2 = function.get_nth_param(1).unwrap().into_int_value();

// Allocate space for parameters (enables mutation if needed)
let param1_alloca = builder.build_alloca(i64_type, "param1").unwrap();
let param2_alloca = builder.build_alloca(i64_type, "param2").unwrap();

builder.build_store(param1_alloca, param1).unwrap();
builder.build_store(param2_alloca, param2).unwrap();

// Function body
let loaded1 = builder.build_load(i64_type, param1_alloca, "a").unwrap();
let loaded2 = builder.build_load(i64_type, param2_alloca, "b").unwrap();
let result = builder.build_int_add(loaded1.into_int_value(), loaded2.into_int_value(), "sum").unwrap();

// Return
builder.build_return(Some(&result)).unwrap();
```

**Generated LLVM IR:**
```llvm
define i64 @add(i64 %0, i64 %1) {
entry:
  %param1 = alloca i64
  %param2 = alloca i64
  store i64 %0, ptr %param1
  store i64 %1, ptr %param2
  %a = load i64, ptr %param1
  %b = load i64, ptr %param2
  %sum = add i64 %a, %b
  ret i64 %sum
}
```

### Function Calls

**Why**: LLVM function calls specify the exact function and argument list. The type system ensures call safety.

```rust
let args = vec![left.into(), right.into()];
let call_result = builder.build_call(function, &args, "call_add").unwrap();
let return_value = call_result.try_as_basic_value().left().unwrap();
```

**Generated LLVM IR:**
```llvm
%call_add = call i64 @add(i64 %left, i64 %right)
```

**Implementation steps for functions:**
1. **Type construction**: Build function signature from parameter and return types
2. **Declaration**: Create function in module with proper linkage
3. **Body generation**: Create entry block and position builder
4. **Parameter handling**: Allocate space for mutable parameters
5. **Return management**: Ensure all paths return appropriate values

## Control Flow

### If Expressions

**Why**: LLVM uses basic blocks for control flow. Conditional branches split execution paths that later merge using PHI nodes.

```rust
// Condition evaluation
let condition = builder.build_int_compare(
    IntPredicate::SGT,
    some_value,
    zero_value,
    "is_positive"
).unwrap();

// Create basic blocks
let then_block = context.append_basic_block(function, "if_then");
let else_block = context.append_basic_block(function, "if_else");
let merge_block = context.append_basic_block(function, "if_merge");

// Conditional branch
builder.build_conditional_branch(condition, then_block, else_block).unwrap();

// Then block
builder.position_at_end(then_block);
let then_value = i64_type.const_int(100, false);
builder.build_unconditional_branch(merge_block).unwrap();

// Else block
builder.position_at_end(else_block);
let else_value = i64_type.const_int(200, false);
builder.build_unconditional_branch(merge_block).unwrap();

// Merge block with PHI
builder.position_at_end(merge_block);
let phi = builder.build_phi(i64_type, "if_result").unwrap();
phi.add_incoming(&[(&then_value, then_block), (&else_value, else_block)]);
```

**Generated LLVM IR:**
```llvm
%is_positive = icmp sgt i64 %some_value, 0
br i1 %is_positive, label %if_then, label %if_else

if_then:
  br label %if_merge

if_else:
  br label %if_merge

if_merge:
  %if_result = phi i64 [ 100, %if_then ], [ 200, %if_else ]
```

### While Loops

**Why**: Loops need condition checking and back-edges. LLVM represents this with conditional branches and block organization.

```rust
let loop_header = context.append_basic_block(function, "loop_header");
let loop_body = context.append_basic_block(function, "loop_body");
let loop_exit = context.append_basic_block(function, "loop_exit");

// Jump to header
builder.build_unconditional_branch(loop_header).unwrap();

// Header: check condition
builder.position_at_end(loop_header);
let condition = /* ... evaluate loop condition ... */;
builder.build_conditional_branch(condition, loop_body, loop_exit).unwrap();

// Body: execute statements and jump back
builder.position_at_end(loop_body);
// ... loop body statements ...
builder.build_unconditional_branch(loop_header).unwrap();

// Exit: continue after loop
builder.position_at_end(loop_exit);
```

**Generated LLVM IR:**
```llvm
br label %loop_header

loop_header:
  %condition = icmp slt i64 %i, 10
  br i1 %condition, label %loop_body, label %loop_exit

loop_body:
  ; loop body instructions
  br label %loop_header

loop_exit:
  ; continue execution
```

**Implementation steps for control flow:**
1. **Basic block creation**: Pre-create all needed blocks
2. **Condition evaluation**: Generate comparison instructions
3. **Branch generation**: Use conditional/unconditional branches
4. **PHI node handling**: Merge values from different execution paths
5. **Builder positioning**: Carefully manage where instructions are inserted

## Data Structures

### Arrays

**Why**: LLVM arrays are fixed-size, homogeneous collections. Access uses GEP (GetElementPtr) for safe pointer arithmetic.

```rust
// Array type [5 x i64]
let array_type = i64_type.array_type(5);
let array_alloca = builder.build_alloca(array_type, "my_array").unwrap();

// Initialize with values
let values = [1, 2, 3, 4, 5].map(|v| i64_type.const_int(v, false));
let array_constant = i64_type.const_array(&values);
builder.build_store(array_alloca, array_constant).unwrap();

// Access element at index 2
let zero = i64_type.const_int(0, false);
let index = i64_type.const_int(2, false);
let element_ptr = unsafe {
    builder.build_gep(
        array_type,
        array_alloca,
        &[zero, index],
        "element_ptr"
    ).unwrap()
};
let element = builder.build_load(i64_type, element_ptr, "element").unwrap();
```

**Generated LLVM IR:**
```llvm
%my_array = alloca [5 x i64]
store [5 x i64] [i64 1, i64 2, i64 3, i64 4, i64 5], ptr %my_array
%element_ptr = getelementptr [5 x i64], ptr %my_array, i64 0, i64 2
%element = load i64, ptr %element_ptr
```

### Structs

**Why**: LLVM structs group heterogeneous data. Field access uses GEP with field indices, providing type safety and optimization opportunities.

```rust
// Define struct type: { i64, f64, i1 }
let field_types = vec![
    i64_type.into(),
    f64_type.into(),
    bool_type.into()
];
let struct_type = context.struct_type(&field_types, false);

// Allocate and initialize
let struct_alloca = builder.build_alloca(struct_type, "my_struct").unwrap();

// Set field 0 (i64)
let zero = i64_type.const_int(0, false);
let field_0_ptr = builder.build_struct_gep(
    struct_type,
    struct_alloca,
    0,
    "field_0_ptr"
).unwrap();
let value_0 = i64_type.const_int(42, false);
builder.build_store(field_0_ptr, value_0).unwrap();

// Set field 1 (f64)
let field_1_ptr = builder.build_struct_gep(
    struct_type,
    struct_alloca,
    1,
    "field_1_ptr"
).unwrap();
let value_1 = f64_type.const_float(3.14);
builder.build_store(field_1_ptr, value_1).unwrap();
```

**Generated LLVM IR:**
```llvm
%my_struct = alloca { i64, double, i1 }
%field_0_ptr = getelementptr { i64, double, i1 }, ptr %my_struct, i32 0, i32 0
store i64 42, ptr %field_0_ptr
%field_1_ptr = getelementptr { i64, double, i1 }, ptr %my_struct, i32 0, i32 1
store double 3.14, ptr %field_1_ptr
```

**Implementation steps for data structures:**
1. **Type construction**: Build aggregate types from component types
2. **Memory allocation**: Use `alloca` for stack allocation
3. **Element access**: Use GEP for safe address calculation
4. **Initialization**: Store values into allocated memory
5. **Bounds checking**: Ensure indices are valid (if dynamic)

## Advanced Constructs

### Lambda Functions (Closures)

**Why**: Lambdas capture variables from their environment. LLVM implements this through closure structs containing captured variables and function pointers.

**Implementation steps for lambdas:**
1. **Capture analysis**: Identify variables referenced from outer scopes
2. **Closure struct creation**: Build struct type containing captured variables
3. **Environment allocation**: Create closure instance on stack/heap
4. **Wrapper function generation**: Create function that unpacks closure and calls code
5. **Function pointer creation**: Return callable reference to wrapper

**Conceptual LLVM IR:**
```llvm
; Closure struct for lambda capturing 'x'
%closure = type { i64 }

; Lambda implementation
define i64 @lambda_impl(ptr %env, i64 %param) {
  %x_ptr = getelementptr %closure, ptr %env, i32 0, i32 0
  %x = load i64, ptr %x_ptr
  %result = add i64 %x, %param
  ret i64 %result
}

; Closure creation
%closure_inst = alloca %closure
%x_val = load i64, ptr %x_var
store i64 %x_val, ptr %closure_inst
```

### String Literals and Operations

**Why**: Strings are complex in LLVM - they're essentially `[N x i8]` arrays. Operations require runtime support or careful memory management.

```rust
// String literal "hello"
let hello_str = "hello\0"; // null-terminated
let hello_global = builder.build_global_string_ptr(hello_str, "hello_str").unwrap();
```

**Generated LLVM IR:**
```llvm
@hello_str = private constant [6 x i8] c"hello\00"
%1 = getelementptr [6 x i8], ptr @hello_str, i64 0, i64 0
```

**Implementation steps for strings:**
1. **Literal creation**: Store string constants in global memory
2. **Runtime support**: Implement length, concatenation, comparison functions
3. **Memory management**: Handle allocation/deallocation for dynamic strings
4. **Encoding handling**: Support UTF-8 or other character encodings

### Error Handling Patterns

**Why**: LLVM doesn't have built-in exception handling for Y Lang's level. Consider result types or error code patterns.

**Common approaches:**
1. **Result tuples**: Return `{success_flag, value, error_code}`
2. **Tagged unions**: Use LLVM's support for variant types
3. **Error globals**: Set global error state (simpler but less composable)

## Best Practices

### Memory Management
- Use `alloca` for local variables (stack allocated)
- Prefer stack allocation over heap when possible
- Consider lifetime and scope when choosing allocation strategy

### Type Safety
- Always verify types before casting LLVM values
- Use the type system to catch errors early
- Provide clear error messages for type mismatches

### Optimization Friendly Code
- Name your values descriptively for debugging
- Avoid unnecessary allocations in hot paths
- Use LLVM's built-in optimization passes

### Debugging Support
- Generate debug information when possible
- Use meaningful names for basic blocks and values
- Structure code to match source-level concepts

This reference provides the conceptual foundation for implementing Y Lang constructs using Inkwell. Each pattern can be extended and optimized based on specific language requirements.
