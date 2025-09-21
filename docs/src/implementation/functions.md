# Functions

This section covers implementing Y Lang's function system using Inkwell, including function declaration, parameter handling, calls, returns, and calling conventions.

## Function Declaration and Signatures

**Why function types matter**: LLVM requires explicit function signatures that define parameter types, return type, and calling conventions. This enables type checking, optimization, and proper code generation.

### Basic Function Declaration

Y Lang functions map to LLVM functions with explicit type signatures:

```rust
use inkwell::context::Context;
use inkwell::types::BasicMetadataTypeEnum;

let context = Context::create();
let module = context.create_module("functions");
let builder = context.create_builder();

let i64_type = context.i64_type();
let f64_type = context.f64_type();
let void_type = context.void_type();

// Function: fn add(a: i64, b: i64) -> i64
let param_types = vec![
    BasicMetadataTypeEnum::IntType(i64_type),
    BasicMetadataTypeEnum::IntType(i64_type),
];
let fn_type = i64_type.fn_type(&param_types, false); // false = not variadic
let add_function = module.add_function("add", fn_type, None);
```

**Generated LLVM IR:**
```llvm
declare i64 @add(i64, i64)
```

**Implementation steps**:
1. Collect parameter types from Y Lang function signature
2. Determine return type (void for no return)
3. Create LLVM function type with `fn_type()`
4. Add function to module with unique name

### Function with No Parameters

```rust
// Function: fn get_answer() -> i64
let fn_type = i64_type.fn_type(&[], false); // Empty parameter list
let get_answer = module.add_function("get_answer", fn_type, None);
```

**Generated LLVM IR:**
```llvm
declare i64 @get_answer()
```

### Void Functions

```rust
// Function: fn print_hello() -> ()
let void_fn_type = void_type.fn_type(&[], false);
let print_hello = module.add_function("print_hello", void_fn_type, None);
```

**Generated LLVM IR:**
```llvm
declare void @print_hello()
```

## Function Implementation

**Why basic blocks are required**: LLVM functions must contain at least one basic block with a terminator instruction. The entry block is where execution begins.

### Complete Function Implementation

```rust
// Implement: fn add(a: i64, b: i64) -> i64 { a + b }

// Create entry basic block
let entry_block = context.append_basic_block(add_function, "entry");
builder.position_at_end(entry_block);

// Access function parameters
let param_a = add_function.get_nth_param(0).unwrap().into_int_value();
let param_b = add_function.get_nth_param(1).unwrap().into_int_value();

// Allocate parameters for potential mutation (Y Lang semantics)
let a_alloca = builder.build_alloca(i64_type, "a").unwrap();
let b_alloca = builder.build_alloca(i64_type, "b").unwrap();

builder.build_store(a_alloca, param_a).unwrap();
builder.build_store(b_alloca, param_b).unwrap();

// Function body: a + b
let a_val = builder.build_load(i64_type, a_alloca, "a_val").unwrap().into_int_value();
let b_val = builder.build_load(i64_type, b_alloca, "b_val").unwrap().into_int_value();
let sum = builder.build_int_add(a_val, b_val, "sum").unwrap();

// Return result
builder.build_return(Some(&sum)).unwrap();
```

**Generated LLVM IR:**
```llvm
define i64 @add(i64 %0, i64 %1) {
entry:
  %a = alloca i64
  %b = alloca i64
  store i64 %0, ptr %a
  store i64 %1, ptr %b
  %a_val = load i64, ptr %a
  %b_val = load i64, ptr %b
  %sum = add i64 %a_val, %b_val
  ret i64 %sum
}
```

### Void Function Implementation

```rust
// Implement: fn print_number(n: i64) { /* side effects only */ }
let param_types = vec![BasicMetadataTypeEnum::IntType(i64_type)];
let fn_type = void_type.fn_type(&param_types, false);
let print_number = module.add_function("print_number", fn_type, None);

let entry_block = context.append_basic_block(print_number, "entry");
builder.position_at_end(entry_block);

// Access parameter
let param_n = print_number.get_nth_param(0).unwrap().into_int_value();

// Function body (side effects, I/O, etc.)
// ... implementation details ...

// Void return
builder.build_return(None).unwrap();
```

**Generated LLVM IR:**
```llvm
define void @print_number(i64 %0) {
entry:
  ; function body
  ret void
}
```

## Parameter Handling Strategies

### Immutable Parameters (Default)

For parameters that won't be reassigned, direct use without allocation:

```rust
// Optimized version for immutable parameters
let entry_block = context.append_basic_block(add_function, "entry");
builder.position_at_end(entry_block);

let param_a = add_function.get_nth_param(0).unwrap().into_int_value();
let param_b = add_function.get_nth_param(1).unwrap().into_int_value();

// Direct use without allocation
let sum = builder.build_int_add(param_a, param_b, "sum").unwrap();
builder.build_return(Some(&sum)).unwrap();
```

**Generated LLVM IR:**
```llvm
define i64 @add(i64 %0, i64 %1) {
entry:
  %sum = add i64 %0, %1
  ret i64 %sum
}
```

### Mutable Parameters

For parameters that may be reassigned within the function:

```rust
// Function: fn increment_and_add(mut a: i64, b: i64) -> i64
let entry_block = context.append_basic_block(function, "entry");
builder.position_at_end(entry_block);

let param_a = function.get_nth_param(0).unwrap().into_int_value();
let param_b = function.get_nth_param(1).unwrap().into_int_value();

// Allocate only mutable parameter
let a_alloca = builder.build_alloca(i64_type, "a").unwrap();
builder.build_store(a_alloca, param_a).unwrap();

// Increment a
let a_val = builder.build_load(i64_type, a_alloca, "a_val").unwrap().into_int_value();
let incremented = builder.build_int_add(a_val, i64_type.const_int(1, false), "incremented").unwrap();
builder.build_store(a_alloca, incremented).unwrap();

// Add to b
let final_a = builder.build_load(i64_type, a_alloca, "final_a").unwrap().into_int_value();
let result = builder.build_int_add(final_a, param_b, "result").unwrap();

builder.build_return(Some(&result)).unwrap();
```

### Reference Parameters

Y Lang references are passed as pointers:

```rust
// Function: fn increment_ref(ref: &mut i64)
let ptr_type = context.ptr_type(Default::default());
let param_types = vec![BasicMetadataTypeEnum::PointerType(ptr_type)];
let fn_type = void_type.fn_type(&param_types, false);
let increment_ref = module.add_function("increment_ref", fn_type, None);

let entry_block = context.append_basic_block(increment_ref, "entry");
builder.position_at_end(entry_block);

let ref_param = increment_ref.get_nth_param(0).unwrap().into_pointer_value();

// Load current value
let current = builder.build_load(i64_type, ref_param, "current").unwrap().into_int_value();

// Increment
let incremented = builder.build_int_add(current, i64_type.const_int(1, false), "incremented").unwrap();

// Store back
builder.build_store(ref_param, incremented).unwrap();
builder.build_return(None).unwrap();
```

**Generated LLVM IR:**
```llvm
define void @increment_ref(ptr %0) {
entry:
  %current = load i64, ptr %0
  %incremented = add i64 %current, 1
  store i64 %incremented, ptr %0
  ret void
}
```

## Function Calls

**Why call instructions matter**: LLVM call instructions handle argument passing, stack management, and return value handling according to the target's calling convention.

### Basic Function Calls

```rust
// Call: add(10, 20)
let arg1 = i64_type.const_int(10, false);
let arg2 = i64_type.const_int(20, false);
let args = vec![arg1.into(), arg2.into()];

let call_result = builder.build_call(add_function, &args, "call_add").unwrap();
let return_value = call_result.try_as_basic_value().left().unwrap().into_int_value();

// Use return value
let doubled = builder.build_int_mul(return_value, i64_type.const_int(2, false), "doubled").unwrap();
```

**Generated LLVM IR:**
```llvm
%call_add = call i64 @add(i64 10, i64 20)
%doubled = mul i64 %call_add, 2
```

### Void Function Calls

```rust
// Call: print_number(42)
let arg = i64_type.const_int(42, false);
let args = vec![arg.into()];

builder.build_call(print_number, &args, "call_print").unwrap();
// No return value to handle
```

**Generated LLVM IR:**
```llvm
call void @print_number(i64 42)
```

### Nested Function Calls

```rust
// Call: add(add(1, 2), add(3, 4))
let inner1_args = vec![
    i64_type.const_int(1, false).into(),
    i64_type.const_int(2, false).into()
];
let inner1_result = builder.build_call(add_function, &inner1_args, "inner1").unwrap()
    .try_as_basic_value().left().unwrap();

let inner2_args = vec![
    i64_type.const_int(3, false).into(),
    i64_type.const_int(4, false).into()
];
let inner2_result = builder.build_call(add_function, &inner2_args, "inner2").unwrap()
    .try_as_basic_value().left().unwrap();

let outer_args = vec![inner1_result.into(), inner2_result.into()];
let final_result = builder.build_call(add_function, &outer_args, "outer").unwrap();
```

**Generated LLVM IR:**
```llvm
%inner1 = call i64 @add(i64 1, i64 2)
%inner2 = call i64 @add(i64 3, i64 4)
%outer = call i64 @add(i64 %inner1, i64 %inner2)
```

## Return Value Handling

### Early Returns

Y Lang functions can have multiple return points:

```rust
// Function: fn abs(x: i64) -> i64
let fn_type = i64_type.fn_type(&[BasicMetadataTypeEnum::IntType(i64_type)], false);
let abs_function = module.add_function("abs", fn_type, None);

let entry_block = context.append_basic_block(abs_function, "entry");
let negative_block = context.append_basic_block(abs_function, "negative");
let positive_block = context.append_basic_block(abs_function, "positive");

builder.position_at_end(entry_block);

let param_x = abs_function.get_nth_param(0).unwrap().into_int_value();
let zero = i64_type.const_zero();

// Check if negative
let is_negative = builder.build_int_compare(
    IntPredicate::SLT,
    param_x,
    zero,
    "is_negative"
).unwrap();

builder.build_conditional_branch(is_negative, negative_block, positive_block).unwrap();

// Negative case: return -x
builder.position_at_end(negative_block);
let negated = builder.build_int_neg(param_x, "negated").unwrap();
builder.build_return(Some(&negated)).unwrap();

// Positive case: return x
builder.position_at_end(positive_block);
builder.build_return(Some(&param_x)).unwrap();
```

**Generated LLVM IR:**
```llvm
define i64 @abs(i64 %0) {
entry:
  %is_negative = icmp slt i64 %0, 0
  br i1 %is_negative, label %negative, label %positive

negative:
  %negated = sub i64 0, %0
  ret i64 %negated

positive:
  ret i64 %0
}
```

### Expression-Based Returns

Y Lang functions return the value of their last expression:

```rust
// Function body is a single expression
let entry_block = context.append_basic_block(function, "entry");
builder.position_at_end(entry_block);

// Function body expression evaluation
let result = /* ... evaluate expression ... */;

// Return the expression result
builder.build_return(Some(&result)).unwrap();
```

## Function Overloading and Name Mangling

Y Lang may support function overloading, requiring name mangling:

```rust
// Original: fn add(a: i64, b: i64) -> i64
// Mangled: add_i64_i64_i64

fn mangle_function_name(name: &str, param_types: &[Type], return_type: &Type) -> String {
    let mut mangled = name.to_string();

    for param_type in param_types {
        mangled.push('_');
        mangled.push_str(&type_to_string(param_type));
    }

    mangled.push('_');
    mangled.push_str(&type_to_string(return_type));

    mangled
}

// Usage
let mangled_name = mangle_function_name("add", &[Type::I64, Type::I64], &Type::I64);
let function = module.add_function(&mangled_name, fn_type, None);
```

## Recursive Functions

Recursive functions work naturally in LLVM due to function declarations:

```rust
// Function: fn factorial(n: i64) -> i64
let fn_type = i64_type.fn_type(&[BasicMetadataTypeEnum::IntType(i64_type)], false);
let factorial = module.add_function("factorial", fn_type, None);

let entry_block = context.append_basic_block(factorial, "entry");
let base_case_block = context.append_basic_block(factorial, "base_case");
let recursive_case_block = context.append_basic_block(factorial, "recursive_case");

builder.position_at_end(entry_block);

let param_n = factorial.get_nth_param(0).unwrap().into_int_value();
let one = i64_type.const_int(1, false);

// Check base case: n <= 1
let is_base_case = builder.build_int_compare(
    IntPredicate::SLE,
    param_n,
    one,
    "is_base_case"
).unwrap();

builder.build_conditional_branch(is_base_case, base_case_block, recursive_case_block).unwrap();

// Base case: return 1
builder.position_at_end(base_case_block);
builder.build_return(Some(&one)).unwrap();

// Recursive case: return n * factorial(n - 1)
builder.position_at_end(recursive_case_block);
let n_minus_1 = builder.build_int_sub(param_n, one, "n_minus_1").unwrap();

// Recursive call
let recursive_args = vec![n_minus_1.into()];
let recursive_result = builder.build_call(factorial, &recursive_args, "factorial_recursive").unwrap()
    .try_as_basic_value().left().unwrap().into_int_value();

let result = builder.build_int_mul(param_n, recursive_result, "result").unwrap();
builder.build_return(Some(&result)).unwrap();
```

**Generated LLVM IR:**
```llvm
define i64 @factorial(i64 %0) {
entry:
  %is_base_case = icmp sle i64 %0, 1
  br i1 %is_base_case, label %base_case, label %recursive_case

base_case:
  ret i64 1

recursive_case:
  %n_minus_1 = sub i64 %0, 1
  %factorial_recursive = call i64 @factorial(i64 %n_minus_1)
  %result = mul i64 %0, %factorial_recursive
  ret i64 %result
}
```

## Higher-Order Functions

Functions that take other functions as parameters:

```rust
// Function type for operation: fn(i64, i64) -> i64
let op_fn_type = i64_type.fn_type(&[
    BasicMetadataTypeEnum::IntType(i64_type),
    BasicMetadataTypeEnum::IntType(i64_type)
], false);

// Function: fn apply_op(op: fn(i64, i64) -> i64, a: i64, b: i64) -> i64
let fn_ptr_type = op_fn_type.ptr_type(Default::default());
let apply_op_type = i64_type.fn_type(&[
    BasicMetadataTypeEnum::PointerType(fn_ptr_type),
    BasicMetadataTypeEnum::IntType(i64_type),
    BasicMetadataTypeEnum::IntType(i64_type)
], false);

let apply_op = module.add_function("apply_op", apply_op_type, None);

let entry_block = context.append_basic_block(apply_op, "entry");
builder.position_at_end(entry_block);

let fn_param = apply_op.get_nth_param(0).unwrap().into_pointer_value();
let a_param = apply_op.get_nth_param(1).unwrap().into_int_value();
let b_param = apply_op.get_nth_param(2).unwrap().into_int_value();

// Call the function pointer
let args = vec![a_param.into(), b_param.into()];
let result = builder.build_indirect_call(op_fn_type, fn_param, &args, "indirect_call").unwrap()
    .try_as_basic_value().left().unwrap();

builder.build_return(Some(&result)).unwrap();
```

**Generated LLVM IR:**
```llvm
define i64 @apply_op(ptr %0, i64 %1, i64 %2) {
entry:
  %indirect_call = call i64 %0(i64 %1, i64 %2)
  ret i64 %indirect_call
}
```

## Error Handling in Functions

### Validation and Assertions

```rust
fn validate_function_signature(
    name: &str,
    param_types: &[BasicTypeEnum],
    return_type: Option<BasicTypeEnum>
) -> Result<(), String> {
    if name.is_empty() {
        return Err("Function name cannot be empty".to_string());
    }

    if param_types.len() > 255 {
        return Err("Too many parameters (max 255)".to_string());
    }

    // Additional validation logic
    Ok(())
}
```

### Safe Function Calls

```rust
fn safe_function_call<'ctx>(
    builder: &Builder<'ctx>,
    function: FunctionValue<'ctx>,
    args: &[BasicValueEnum<'ctx>],
    name: &str
) -> Result<Option<BasicValueEnum<'ctx>>, String> {
    let fn_type = function.get_type();
    let param_types = fn_type.get_param_types();

    if args.len() != param_types.len() {
        return Err(format!(
            "Argument count mismatch: expected {}, got {}",
            param_types.len(),
            args.len()
        ));
    }

    // Type checking
    for (i, (arg, expected_type)) in args.iter().zip(param_types.iter()).enumerate() {
        if arg.get_type() != *expected_type {
            return Err(format!(
                "Argument {} type mismatch: expected {:?}, got {:?}",
                i, expected_type, arg.get_type()
            ));
        }
    }

    let call_site = builder.build_call(function, args, name)
        .map_err(|e| format!("Call failed: {}", e))?;

    Ok(call_site.try_as_basic_value().left())
}
```

## Optimization Considerations

### Inlining Hints

```rust
use inkwell::attributes::{Attribute, AttributeLoc};

// Mark function for inlining
let inline_attr = context.create_enum_attribute(Attribute::get_named_enum_kind_id("alwaysinline"), 0);
function.add_attribute(AttributeLoc::Function, inline_attr);
```

### Tail Call Optimization

```rust
// Enable tail call optimization for recursive functions
let call_site = builder.build_call(function, args, "tail_call").unwrap();
call_site.set_tail_call(true);
```

### Function Attributes

```rust
// Mark function as pure (no side effects)
let readonly_attr = context.create_enum_attribute(Attribute::get_named_enum_kind_id("readonly"), 0);
function.add_attribute(AttributeLoc::Function, readonly_attr);

// Mark function as not throwing exceptions
let nounwind_attr = context.create_enum_attribute(Attribute::get_named_enum_kind_id("nounwind"), 0);
function.add_attribute(AttributeLoc::Function, nounwind_attr);
```

This comprehensive coverage of functions provides the foundation for implementing Y Lang's function system in LLVM, handling declaration, implementation, calls, and advanced patterns like recursion and higher-order functions.
