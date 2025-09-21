# Variables and Memory

This section covers variable declaration, memory allocation, scope management, and mutability in Y Lang using Inkwell's memory management primitives.

## Variable Declaration and Storage

**Why variables need memory**: Unlike constants which exist in the IR directly, variables represent mutable storage locations that can change during program execution. LLVM provides stack allocation through `alloca` instructions.

### Basic Variable Declaration

Y Lang variables map to stack-allocated memory slots:

```rust
use inkwell::context::Context;

let context = Context::create();
let module = context.create_module("variables");
let builder = context.create_builder();

// Create a function context for variables
let i64_type = context.i64_type();
let fn_type = context.void_type().fn_type(&[], false);
let function = module.add_function("test_vars", fn_type, None);
let entry_block = context.append_basic_block(function, "entry");
builder.position_at_end(entry_block);

// Declare variable: let x: i64;
let x_alloca = builder.build_alloca(i64_type, "x").unwrap();
```

**Generated LLVM IR:**
```llvm
define void @test_vars() {
entry:
  %x = alloca i64
  ret void
}
```

**Implementation steps**:
1. Position builder in a function's basic block
2. Use `build_alloca` to reserve stack space
3. Store variable metadata (name, type) in symbol table
4. Handle initialization separately

### Variable Initialization

Variables can be initialized at declaration or later:

```rust
// Declare and initialize: let x = 42;
let x_alloca = builder.build_alloca(i64_type, "x").unwrap();
let initial_value = i64_type.const_int(42, false);
builder.build_store(x_alloca, initial_value).unwrap();

// Or initialize from another expression
let computed_value = builder.build_int_add(
    i64_type.const_int(10, false),
    i64_type.const_int(20, false),
    "computed"
).unwrap();
let y_alloca = builder.build_alloca(i64_type, "y").unwrap();
builder.build_store(y_alloca, computed_value).unwrap();
```

**Generated LLVM IR:**
```llvm
%x = alloca i64
store i64 42, ptr %x
%computed = add i64 10, 20
%y = alloca i64
store i64 %computed, ptr %y
```

### Variable Access (Loading)

Reading a variable's value requires loading from memory:

```rust
// Read variable value: x
let x_value = builder.build_load(i64_type, x_alloca, "x_val").unwrap();

// Use in expression: x + 10
let result = builder.build_int_add(
    x_value.into_int_value(),
    i64_type.const_int(10, false),
    "x_plus_10"
).unwrap();
```

**Generated LLVM IR:**
```llvm
%x_val = load i64, ptr %x
%x_plus_10 = add i64 %x_val, 10
```

## Mutability and Assignment

Y Lang distinguishes between immutable and mutable variables, but at the LLVM level, all variables are potentially mutable through their memory allocation.

### Immutable Variables

Even "immutable" variables use `alloca` for consistency, but the type checker prevents reassignment:

```rust
// let x = 42; (immutable)
let x_alloca = builder.build_alloca(i64_type, "x").unwrap();
let value = i64_type.const_int(42, false);
builder.build_store(x_alloca, value).unwrap();

// Immutability enforced by Y Lang type checker, not LLVM
```

### Mutable Variables

Mutable variables allow reassignment through additional store operations:

```rust
// let mut y = 10; (mutable)
let y_alloca = builder.build_alloca(i64_type, "y").unwrap();
let initial = i64_type.const_int(10, false);
builder.build_store(y_alloca, initial).unwrap();

// y = 20; (assignment)
let new_value = i64_type.const_int(20, false);
builder.build_store(y_alloca, new_value).unwrap();
```

**Generated LLVM IR:**
```llvm
%y = alloca i64
store i64 10, ptr %y
store i64 20, ptr %y
```

### Assignment Expressions

Y Lang assignment returns the assigned value:

```rust
// x = y = 42; (chained assignment)
let value = i64_type.const_int(42, false);

// y = 42
builder.build_store(y_alloca, value).unwrap();

// x = y (but we use the immediate value for efficiency)
builder.build_store(x_alloca, value).unwrap();

// The expression evaluates to the assigned value
let assignment_result = value; // Value of the assignment expression
```

**Generated LLVM IR:**
```llvm
store i64 42, ptr %y
store i64 42, ptr %x
; %assignment_result is just 42 (the constant)
```

## Scope Management

Variables have lexical scope that must be tracked during code generation.

### Block Scopes

Y Lang blocks create new variable scopes:

```rust
// Outer scope
let outer_var = builder.build_alloca(i64_type, "outer").unwrap();

// Enter new block scope
// { let inner = 10; ... }
let inner_var = builder.build_alloca(i64_type, "inner").unwrap();
let inner_value = i64_type.const_int(10, false);
builder.build_store(inner_var, inner_value).unwrap();

// Exit block scope - variables still exist in LLVM but become inaccessible
// through symbol table management
```

**Generated LLVM IR:**
```llvm
%outer = alloca i64
%inner = alloca i64
store i64 10, ptr %inner
; Both allocations persist until function return
```

**Implementation pattern for scope management**:

```rust
struct ScopeManager {
    scopes: Vec<HashMap<String, PointerValue<'ctx>>>,
}

impl ScopeManager {
    fn enter_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn exit_scope(&mut self) {
        self.scopes.pop();
    }

    fn declare_variable(&mut self, name: String, alloca: PointerValue<'ctx>) -> Result<(), String> {
        let current_scope = self.scopes.last_mut()
            .ok_or("No active scope")?;

        if current_scope.contains_key(&name) {
            return Err(format!("Variable '{}' already declared in this scope", name));
        }

        current_scope.insert(name, alloca);
        Ok(())
    }

    fn lookup_variable(&self, name: &str) -> Option<PointerValue<'ctx>> {
        // Search scopes from innermost to outermost
        for scope in self.scopes.iter().rev() {
            if let Some(&alloca) = scope.get(name) {
                return Some(alloca);
            }
        }
        None
    }
}
```

### Function Parameter Scope

Function parameters need special handling since they arrive as values, not allocations:

```rust
// Define function: fn add(a: i64, b: i64) -> i64
let param_types = vec![
    BasicMetadataTypeEnum::IntType(i64_type),
    BasicMetadataTypeEnum::IntType(i64_type),
];
let fn_type = i64_type.fn_type(&param_types, false);
let function = module.add_function("add", fn_type, None);

let entry_block = context.append_basic_block(function, "entry");
builder.position_at_end(entry_block);

// Parameters come as values, allocate them for mutability
let param_a = function.get_nth_param(0).unwrap().into_int_value();
let param_b = function.get_nth_param(1).unwrap().into_int_value();

let a_alloca = builder.build_alloca(i64_type, "a").unwrap();
let b_alloca = builder.build_alloca(i64_type, "b").unwrap();

builder.build_store(a_alloca, param_a).unwrap();
builder.build_store(b_alloca, param_b).unwrap();

// Now parameters can be used like local variables
let a_val = builder.build_load(i64_type, a_alloca, "a_val").unwrap();
let b_val = builder.build_load(i64_type, b_alloca, "b_val").unwrap();
let sum = builder.build_int_add(a_val.into_int_value(), b_val.into_int_value(), "sum").unwrap();

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

## Memory Layout and Optimization

### Stack Frame Organization

LLVM automatically manages stack frame layout, but understanding the principles helps with optimization:

```rust
// Multiple variable declarations create stack frame
let var1 = builder.build_alloca(i64_type, "var1").unwrap();    // 8 bytes
let var2 = builder.build_alloca(f64_type, "var2").unwrap();    // 8 bytes
let var3 = builder.build_alloca(bool_type, "var3").unwrap();   // 1 byte
let var4 = builder.build_alloca(i64_type, "var4").unwrap();    // 8 bytes

// LLVM will arrange these optimally in the stack frame
```

**Generated LLVM IR:**
```llvm
%var1 = alloca i64      ; 8-byte aligned
%var2 = alloca double   ; 8-byte aligned
%var3 = alloca i1       ; 1-byte, but may be padded
%var4 = alloca i64      ; 8-byte aligned
```

### Avoiding Unnecessary Allocations

For simple, non-reassigned variables, consider using SSA values directly:

```rust
// Instead of this (allocation-heavy):
let temp_alloca = builder.build_alloca(i64_type, "temp").unwrap();
let computed = builder.build_int_add(x, y, "computed").unwrap();
builder.build_store(temp_alloca, computed).unwrap();
let temp_val = builder.build_load(i64_type, temp_alloca, "temp_val").unwrap();

// Use this (direct SSA):
let computed = builder.build_int_add(x, y, "computed").unwrap();
// Use 'computed' directly in subsequent operations
```

### Memory Access Patterns

Efficient memory access requires understanding when to load vs. reuse values:

```rust
// Inefficient: repeated loads
let x_val1 = builder.build_load(i64_type, x_alloca, "x1").unwrap();
let y_val1 = builder.build_load(i64_type, y_alloca, "y1").unwrap();
let result1 = builder.build_int_add(x_val1.into_int_value(), y_val1.into_int_value(), "r1").unwrap();

let x_val2 = builder.build_load(i64_type, x_alloca, "x2").unwrap(); // Redundant load
let result2 = builder.build_int_mul(x_val2.into_int_value(), i64_type.const_int(2, false), "r2").unwrap();

// Efficient: load once, reuse values
let x_val = builder.build_load(i64_type, x_alloca, "x").unwrap().into_int_value();
let y_val = builder.build_load(i64_type, y_alloca, "y").unwrap().into_int_value();
let result1 = builder.build_int_add(x_val, y_val, "r1").unwrap();
let result2 = builder.build_int_mul(x_val, i64_type.const_int(2, false), "r2").unwrap();
```

## Advanced Memory Concepts

### Composite Type Variables

Structs and arrays require more complex allocation patterns:

```rust
// Struct variable: let point = Point { x: 10, y: 20 };
let field_types = vec![i64_type.into(), i64_type.into()];
let point_type = context.struct_type(&field_types, false);
let point_alloca = builder.build_alloca(point_type, "point").unwrap();

// Initialize fields
let x_ptr = builder.build_struct_gep(point_type, point_alloca, 0, "x_ptr").unwrap();
let y_ptr = builder.build_struct_gep(point_type, point_alloca, 1, "y_ptr").unwrap();

builder.build_store(x_ptr, i64_type.const_int(10, false)).unwrap();
builder.build_store(y_ptr, i64_type.const_int(20, false)).unwrap();
```

**Generated LLVM IR:**
```llvm
%point = alloca { i64, i64 }
%x_ptr = getelementptr { i64, i64 }, ptr %point, i32 0, i32 0
%y_ptr = getelementptr { i64, i64 }, ptr %point, i32 0, i32 1
store i64 10, ptr %x_ptr
store i64 20, ptr %y_ptr
```

### Array Variables

Arrays use similar patterns with index-based access:

```rust
// Array variable: let arr = [1, 2, 3, 4, 5];
let array_type = i64_type.array_type(5);
let array_alloca = builder.build_alloca(array_type, "arr").unwrap();

// Initialize with constant array
let values = [1, 2, 3, 4, 5].map(|v| i64_type.const_int(v, false));
let array_constant = i64_type.const_array(&values);
builder.build_store(array_alloca, array_constant).unwrap();

// Access element: arr[2]
let zero = i64_type.const_int(0, false);
let index = i64_type.const_int(2, false);
let element_ptr = unsafe {
    builder.build_gep(array_type, array_alloca, &[zero, index], "elem_ptr").unwrap()
};
let element_val = builder.build_load(i64_type, element_ptr, "elem").unwrap();
```

**Generated LLVM IR:**
```llvm
%arr = alloca [5 x i64]
store [5 x i64] [i64 1, i64 2, i64 3, i64 4, i64 5], ptr %arr
%elem_ptr = getelementptr [5 x i64], ptr %arr, i64 0, i64 2
%elem = load i64, ptr %elem_ptr
```

### Reference Variables

Y Lang references map to pointers in LLVM:

```rust
// Reference variable: let ref_x = &x;
let ref_x_alloca = builder.build_alloca(ptr_type, "ref_x").unwrap();
builder.build_store(ref_x_alloca, x_alloca).unwrap();

// Dereferencing: *ref_x
let ptr_val = builder.build_load(ptr_type, ref_x_alloca, "ptr").unwrap();
let deref_val = builder.build_load(i64_type, ptr_val.into_pointer_value(), "deref").unwrap();
```

**Generated LLVM IR:**
```llvm
%ref_x = alloca ptr
store ptr %x, ptr %ref_x
%ptr = load ptr, ptr %ref_x
%deref = load i64, ptr %ptr
```

## Error Handling and Validation

### Variable Lifecycle Validation

Track variable states to prevent common errors:

```rust
#[derive(Debug, Clone, Copy)]
enum VariableState {
    Declared,      // Allocated but not initialized
    Initialized,   // Has a value
    Moved,         // Value has been moved (for move semantics)
}

struct VariableInfo<'ctx> {
    alloca: PointerValue<'ctx>,
    var_type: BasicTypeEnum<'ctx>,
    state: VariableState,
    is_mutable: bool,
}

impl VariableInfo<'_> {
    fn can_read(&self) -> bool {
        matches!(self.state, VariableState::Initialized)
    }

    fn can_assign(&self) -> bool {
        self.is_mutable && !matches!(self.state, VariableState::Moved)
    }
}
```

### Type Safety in Variable Operations

Ensure type compatibility before memory operations:

```rust
fn safe_store<'ctx>(
    builder: &Builder<'ctx>,
    alloca: PointerValue<'ctx>,
    value: BasicValueEnum<'ctx>,
    expected_type: BasicTypeEnum<'ctx>
) -> Result<(), String> {
    if value.get_type() != expected_type {
        return Err(format!(
            "Type mismatch: expected {:?}, got {:?}",
            expected_type, value.get_type()
        ));
    }

    builder.build_store(alloca, value)
        .map_err(|e| format!("Store failed: {}", e))?;

    Ok(())
}
```

### Memory Safety Considerations

While LLVM doesn't enforce memory safety automatically, implement patterns to prevent common issues:

```rust
// Bounds checking for array access
fn safe_array_access<'ctx>(
    builder: &Builder<'ctx>,
    array_alloca: PointerValue<'ctx>,
    array_type: ArrayType<'ctx>,
    index: IntValue<'ctx>,
    element_type: BasicTypeEnum<'ctx>
) -> Result<BasicValueEnum<'ctx>, String> {
    let array_len = array_type.len();
    let index_val = index.get_zero_extended_constant()
        .ok_or("Dynamic array access requires runtime bounds checking")?;

    if index_val >= array_len as u64 {
        return Err(format!("Array index {} out of bounds (length {})", index_val, array_len));
    }

    let zero = element_type.into_int_type().const_int(0, false);
    let element_ptr = unsafe {
        builder.build_gep(array_type, array_alloca, &[zero, index], "elem_ptr")
            .map_err(|e| format!("GEP failed: {}", e))?
    };

    builder.build_load(element_type, element_ptr, "element")
        .map_err(|e| format!("Load failed: {}", e))
}
```

## Performance Optimization Strategies

### Minimize Allocations

Use SSA form for temporary values:

```rust
// Good: Direct SSA computation
let a = builder.build_load(i64_type, a_alloca, "a").unwrap().into_int_value();
let b = builder.build_load(i64_type, b_alloca, "b").unwrap().into_int_value();
let temp1 = builder.build_int_add(a, b, "temp1").unwrap();
let temp2 = builder.build_int_mul(temp1, i64_type.const_int(2, false), "temp2").unwrap();
let result = builder.build_int_sub(temp2, i64_type.const_int(1, false), "result").unwrap();

// Avoid: Unnecessary allocations for temporaries
```

### Leverage LLVM Optimizations

LLVM's optimization passes can eliminate redundant loads and stores:

```rust
// This pattern:
let var = builder.build_alloca(i64_type, "var").unwrap();
builder.build_store(var, i64_type.const_int(42, false)).unwrap();
let val = builder.build_load(i64_type, var, "val").unwrap();

// May be optimized to just:
// %val = i64 42
```

This comprehensive coverage of variables and memory management provides the foundation for implementing Y Lang's variable system in LLVM, emphasizing both correctness and performance considerations.
