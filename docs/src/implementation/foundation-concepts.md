# Foundation Concepts

Understanding LLVM's core abstractions and design philosophy is essential for implementing Y Lang constructs effectively. This section covers the fundamental concepts that underpin all code generation.

## LLVM Architecture Overview

LLVM separates compilation into distinct phases, each with clear responsibilities:

1. **Frontend** (Y Lang parser/type checker) → **LLVM IR**
2. **Optimization passes** → **Optimized LLVM IR**
3. **Backend** → **Target assembly/machine code**

Y Lang's code generator focuses on the first step: producing clean, correct LLVM IR that can be optimized and compiled to any target.

## Core Abstractions

### Context: The Global State Container

**Why Context exists**: LLVM types and constants are interned and cached globally. The Context ensures type identity and memory management across the entire compilation unit.

```rust
use inkwell::context::Context;

let context = Context::create();
// All types created from this context are compatible
let i64_type_1 = context.i64_type();
let i64_type_2 = context.i64_type();
assert_eq!(i64_type_1, i64_type_2); // Same type instance
```

**Key principles**:
- One Context per compilation unit
- All types from the same Context are compatible
- Context owns the memory for types and constants
- Thread-safe but not designed for concurrent modification

### Module: The Compilation Unit

**Why Modules exist**: LLVM organizes code into modules, which represent single compilation units (like .c files). Modules contain functions, global variables, and metadata.

```rust
let module = context.create_module("my_program");

// Modules can contain:
// - Function declarations and definitions
// - Global variables and constants
// - Type definitions
// - Metadata (debug info, etc.)
```

**Module organization in Y Lang**:
- One module per Y Lang source file
- Global functions and constants declared at module level
- Module name typically matches source file name

### Builder: The Instruction Generator

**Why Builder exists**: LLVM instructions must be generated in sequence within basic blocks. The Builder manages this positioning and provides the API for instruction generation.

```rust
let builder = context.create_builder();

// Builder is always positioned within a basic block
// Instructions are inserted at the current position
let add_result = builder.build_int_add(left, right, "sum").unwrap();
// Next instruction will be inserted after the add
```

**Builder positioning patterns**:
```rust
// Position at end of basic block (most common)
builder.position_at_end(basic_block);

// Position before specific instruction (rare)
builder.position_before(&some_instruction);

// Always check current position when debugging
let current_block = builder.get_insert_block().unwrap();
```

## Type System Mapping

Y Lang's type system maps to LLVM types with specific design decisions:

### Primitive Type Mapping

| Y Lang Type | LLVM Type | Size | Reasoning |
|-------------|-----------|------|-----------|
| `i64` | `i64` | 64 bits | Direct mapping, platform independent |
| `f64` | `double` | 64 bits | IEEE 754 double precision |
| `bool` | `i1` | 1 bit | Minimal storage, efficient operations |
| `char` | `i8` | 8 bits | UTF-8 byte, composable into strings |
| `()` (void) | `void` | 0 bits | Represents no value |

```rust
// Type creation examples
let i64_type = context.i64_type();
let f64_type = context.f64_type();
let bool_type = context.bool_type();
let void_type = context.void_type();
let ptr_type = context.ptr_type(Default::default()); // Opaque pointer
```

### Reference and Pointer Types

**Why pointers**: Y Lang references map to LLVM pointers because LLVM doesn't have high-level reference semantics.

```rust
// Y Lang: &i64
// LLVM: ptr (opaque pointer to i64-sized memory)
let ptr_to_i64 = context.ptr_type(Default::default());

// All pointers are opaque in modern LLVM
// Type safety comes from how you load/store
```

### Aggregate Types

**Structs**: Y Lang structs become LLVM struct types with named fields mapped to indices.

```rust
// Y Lang: struct Point { x: i64, y: i64 }
// LLVM: { i64, i64 }
let point_type = context.struct_type(&[
    i64_type.into(),  // field 0: x
    i64_type.into(),  // field 1: y
], false); // false = not packed
```

**Arrays**: Fixed-size homogeneous collections.

```rust
// Y Lang: &[i64; 5]
// LLVM: [5 x i64]
let array_type = i64_type.array_type(5);
```

**Tuples**: Anonymous structs with positional access.

```rust
// Y Lang: (i64, f64, bool)
// LLVM: { i64, double, i1 }
let tuple_type = context.struct_type(&[
    i64_type.into(),
    f64_type.into(),
    bool_type.into(),
], false);
```

## Memory Model

### Stack vs Heap Allocation

**Stack allocation with `alloca`**:
```rust
// Allocates on the current function's stack frame
let var_alloca = builder.build_alloca(i64_type, "local_var").unwrap();

// Properties:
// - Automatically deallocated when function returns
// - Fast allocation (just stack pointer adjustment)
// - Limited by stack size
// - Address is stable within function
```

**Heap allocation patterns**:
```rust
// Y Lang doesn't expose heap allocation directly
// But internal runtime functions might use malloc/free
// Example: dynamic strings, closures with captures
```

### Memory Access Patterns

**Load and Store**:
```rust
let ptr = builder.build_alloca(i64_type, "var").unwrap();

// Store: memory[ptr] = value
let value = i64_type.const_int(42, false);
builder.build_store(ptr, value).unwrap();

// Load: value = memory[ptr]
let loaded = builder.build_load(i64_type, ptr, "loaded").unwrap();
```

**GetElementPtr (GEP) for safe addressing**:
```rust
// Access array element
let array_ptr = builder.build_alloca(array_type, "arr").unwrap();
let index = i64_type.const_int(2, false);
let element_ptr = unsafe {
    builder.build_gep(
        array_type,
        array_ptr,
        &[i64_type.const_int(0, false), index], // [base_offset, element_index]
        "element_ptr"
    ).unwrap()
};

// GEP calculates: base_ptr + (0 * sizeof(array)) + (2 * sizeof(i64))
// Result: pointer to array[2]
```

## Value System

LLVM distinguishes between different value categories:

### Constants vs Variables

**Constants**: Compile-time known values
```rust
let const_42 = i64_type.const_int(42, false);
let const_pi = f64_type.const_float(3.14159);
let const_true = bool_type.const_int(1, false);

// Constants can be used directly in operations
let const_sum = const_42.const_add(i64_type.const_int(8, false));
```

**Variables**: Runtime values that may change
```rust
// Variables require memory allocation and load/store
let var_alloca = builder.build_alloca(i64_type, "var").unwrap();
builder.build_store(var_alloca, const_42).unwrap();
let runtime_value = builder.build_load(i64_type, var_alloca, "value").unwrap();
```

### Value Naming and SSA Form

**Single Static Assignment (SSA)**: Each value is assigned exactly once
```rust
// Good: Each result has a unique name
let a = builder.build_int_add(x, y, "a").unwrap();
let b = builder.build_int_mul(a, z, "b").unwrap();
let c = builder.build_int_sub(b, w, "c").unwrap();

// LLVM IR:
// %a = add i64 %x, %y
// %b = mul i64 %a, %z
// %c = sub i64 %b, %w
```

**Naming conventions**:
- Use descriptive names for debugging: `"user_age"`, `"total_cost"`
- Include operation context: `"array_length"`, `"loop_counter"`
- Temporary values can use generic names: `"tmp"`, `"result"`

## Basic Blocks and Control Flow

### Basic Block Structure

**What is a basic block**: A sequence of instructions with:
- Single entry point (at the beginning)
- Single exit point (terminator instruction)
- No jumps into the middle

```rust
let function = module.add_function("example", void_type.fn_type(&[], false), None);
let entry_block = context.append_basic_block(function, "entry");

builder.position_at_end(entry_block);
// Add instructions here...

// Every basic block must end with a terminator
builder.build_return(None).unwrap(); // ret void
```

### Terminator Instructions

All basic blocks must end with exactly one terminator:

```rust
// Unconditional branch
builder.build_unconditional_branch(target_block).unwrap();

// Conditional branch
builder.build_conditional_branch(condition, then_block, else_block).unwrap();

// Return
builder.build_return(Some(&return_value)).unwrap();
builder.build_return(None).unwrap(); // void return

// Unreachable (for impossible code paths)
builder.build_unreachable().unwrap();
```

### Control Flow Patterns

**Sequential flow**:
```rust
let block1 = context.append_basic_block(function, "block1");
let block2 = context.append_basic_block(function, "block2");

builder.position_at_end(block1);
// ... instructions ...
builder.build_unconditional_branch(block2).unwrap();

builder.position_at_end(block2);
// ... more instructions ...
builder.build_return(None).unwrap();
```

**Conditional flow with merge**:
```rust
let condition_block = context.append_basic_block(function, "condition");
let then_block = context.append_basic_block(function, "then");
let else_block = context.append_basic_block(function, "else");
let merge_block = context.append_basic_block(function, "merge");

// Condition evaluation
builder.position_at_end(condition_block);
let cond = /* ... evaluate condition ... */;
builder.build_conditional_branch(cond, then_block, else_block).unwrap();

// Then path
builder.position_at_end(then_block);
let then_value = /* ... compute then value ... */;
builder.build_unconditional_branch(merge_block).unwrap();

// Else path
builder.position_at_end(else_block);
let else_value = /* ... compute else value ... */;
builder.build_unconditional_branch(merge_block).unwrap();

// Merge point with PHI
builder.position_at_end(merge_block);
let phi = builder.build_phi(i64_type, "merged_value").unwrap();
phi.add_incoming(&[(&then_value, then_block), (&else_value, else_block)]);
```

## Error Handling in Code Generation

### Inkwell Error Patterns

Most Inkwell operations return `Result` types:

```rust
// Handle errors explicitly
match builder.build_int_add(left, right, "sum") {
    Ok(result) => result,
    Err(e) => panic!("Failed to build add instruction: {}", e),
}

// Or use unwrap() for prototype code
let result = builder.build_int_add(left, right, "sum").unwrap();

// Use expect() for better error messages
let result = builder.build_int_add(left, right, "sum")
    .expect("Integer addition should never fail with valid operands");
```

### Common Error Conditions

1. **Type mismatches**: Using incompatible types in operations
2. **Missing terminators**: Basic blocks without terminator instructions
3. **Invalid positioning**: Builder not positioned in a basic block
4. **Name conflicts**: Reusing names in the same scope

### Defensive Programming

```rust
// Verify builder is positioned
assert!(builder.get_insert_block().is_some(), "Builder must be positioned in a basic block");

// Verify types before operations
assert_eq!(left.get_type(), right.get_type(), "Operand types must match");

// Check for existing terminators
let current_block = builder.get_insert_block().unwrap();
if current_block.get_terminator().is_some() {
    panic!("Cannot add instructions after terminator");
}
```

## Performance Considerations

### Compile-Time Performance

- **Type interning**: Context automatically interns types, so type creation is fast
- **Instruction building**: Builder operations are relatively cheap
- **Memory usage**: LLVM IR uses significant memory; avoid creating unnecessary instructions

### Runtime Performance

- **Stack allocation**: Prefer `alloca` over heap allocation when possible
- **Constant folding**: Use LLVM constants for compile-time known values
- **Optimization passes**: Generate simple IR and let LLVM optimize

### Debugging and Development

```rust
// Use descriptive names for values and blocks
let user_age = builder.build_load(i64_type, age_ptr, "user_age").unwrap();
let is_adult = builder.build_int_compare(
    IntPredicate::SGE,
    user_age,
    i64_type.const_int(18, false),
    "is_adult"
).unwrap();

// Name basic blocks meaningfully
let check_age_block = context.append_basic_block(function, "check_age");
let adult_path_block = context.append_basic_block(function, "adult_path");
let minor_path_block = context.append_basic_block(function, "minor_path");
```

This foundation provides the conceptual framework for all Y Lang code generation. Understanding these patterns enables implementing any language construct effectively.
