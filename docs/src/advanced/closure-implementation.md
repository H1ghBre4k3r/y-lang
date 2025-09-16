# Closure Implementation

This document provides a comprehensive technical overview of how closures and variable capture are implemented in the Y language compiler.

## Overview

Y implements closures using a unified representation where all function-typed values are represented as closure structs containing a function pointer and environment pointer. This enables seamless interoperability between named functions and capturing lambdas while maintaining performance for non-capturing cases.

## Architecture

### Closure Representation

All function values in Y use a consistent representation:

```llvm
{ ptr, ptr }  ; {function_pointer, environment_pointer}
```

- **Function Pointer**: Points to the actual executable code
- **Environment Pointer**: Points to captured variables (null for non-capturing functions)

### Function Signatures

There are two types of function signatures in the generated LLVM IR:

1. **Named Functions**: `(params...) -> return_type`
2. **Closure Implementation**: `(ptr env, params...) -> return_type`

The environment pointer is always the first parameter in closure implementations.

## Implementation Flow

### 1. Capture Analysis (Typechecker Phase)

**File**: `crates/why_lib/src/typechecker/typed_ast/expression/lambda.rs`

#### Key Components:

- **`CaptureInfo`**: Stores captured variables and their types
- **`IdentifierCollector`**: AST visitor that identifies free variables
- **Global Storage**: Uses `once_cell::Lazy` for storing capture information

#### Algorithm:

1. **Collect Identifiers**: Visit lambda body and collect all identifier references
2. **Filter Parameters**: Remove lambda parameters from the set
3. **Determine Captures**: Remaining identifiers are captured variables
4. **Store Metadata**: Associate capture info with lambda using position-based ID

```rust
// Simplified capture analysis
fn analyze_captures(lambda_body: &Expression, parameters: &[Parameter]) -> CaptureInfo {
    let mut collector = IdentifierCollector::new();
    lambda_body.visit(&mut collector);

    let captures: Vec<(String, Type)> = collector.identifiers
        .into_iter()
        .filter(|id| !parameters.contains(id))
        .map(|id| (id.name, id.inferred_type))
        .collect();

    CaptureInfo { captures }
}
```

### 2. Code Generation (LLVM IR Generation)

**File**: `crates/why_lib/src/codegen/expressions/lambda.rs`

#### Closure Struct Helpers

**File**: `crates/why_lib/src/codegen/mod.rs`

Key helper methods in `CodegenContext`:

```rust
// Get canonical closure type {ptr, ptr}
fn get_closure_struct_type() -> StructType<'ctx>

// Build closure value from function and environment pointers
fn build_closure_value(fn_ptr: PointerValue, env_ptr: PointerValue) -> StructValue

// Extract function pointer and cast to target type
fn extract_closure_fn_ptr(closure: StructValue, target_type: FunctionType) -> PointerValue

// Extract environment pointer
fn extract_closure_env_ptr(closure: StructValue) -> PointerValue
```

#### Lambda Code Generation

The lambda codegen follows different paths based on capture analysis:

##### Non-Capturing Lambdas

1. Generate standard function with original signature
2. Create closure struct with `env = null`
3. Return closure value

```rust
fn codegen_non_capturing_lambda() {
    // 1. Create function: (params...) -> return_type
    let lambda_fn = ctx.module.add_function(name, standard_fn_type, None);

    // 2. Generate function body
    generate_lambda_body(ctx, lambda_fn, parameters, expression);

    // 3. Create closure with null environment
    let fn_ptr = lambda_fn.as_global_value().as_pointer_value();
    let null_env = ctx.context.ptr_type().const_null();
    let closure = ctx.build_closure_value(fn_ptr, null_env);

    closure
}
```

##### Capturing Lambdas

1. Generate closure implementation function with env parameter
2. Create and populate environment struct on heap
3. Return closure value with environment

```rust
fn codegen_capturing_lambda() {
    // 1. Create function: (ptr env, params...) -> return_type
    let closure_fn_type = ctx.create_closure_impl_fn_type(return_type, params);
    let closure_fn = ctx.module.add_function(name, closure_fn_type, None);

    // 2. Create environment struct and allocate on heap
    let (env_struct_type, env_ptr) = create_and_populate_environment(ctx, captures);

    // 3. Generate function body with environment access
    generate_lambda_body(ctx, closure_fn, parameters, expression, Some(env_info));

    // 4. Create closure value
    let fn_ptr = closure_fn.as_global_value().as_pointer_value();
    let closure = ctx.build_closure_value(fn_ptr, env_ptr);

    closure
}
```

### 3. Environment Management

#### Environment Struct Creation

For capturing lambdas, an environment struct is created with fields for each captured variable:

```rust
fn create_and_populate_environment(ctx: &CodegenContext, captures: &CaptureInfo) {
    // 1. Build struct type from captured variable types
    let field_types: Vec<BasicTypeEnum> = captures.iter()
        .map(|(_, var_type)| ctx.get_llvm_type(var_type))
        .collect();
    let env_struct_type = ctx.context.struct_type(&field_types, false);

    // 2. Allocate on heap using malloc
    let env_size = env_struct_type.size_of().unwrap();
    let env_ptr = ctx.builder.build_call(malloc_fn, &[env_size], "env_malloc");

    // 3. Copy captured values into environment
    for (i, (var_name, _)) in captures.iter().enumerate() {
        let captured_value = ctx.find_variable(var_name);
        let field_ptr = ctx.builder.build_struct_gep(
            env_struct_type, env_ptr, i as u32, "env_field"
        );
        ctx.builder.build_store(field_ptr, captured_value);
    }

    (env_struct_type, env_ptr)
}
```

#### Environment Access in Lambda Body

Within the lambda body, captured variables are accessed by loading from the environment struct:

```rust
fn generate_lambda_body_with_env(ctx: &CodegenContext, env_info: EnvInfo) {
    // Cast environment pointer to struct type
    let env_param = lambda_fn.get_nth_param(0).into_pointer_value();
    let env_struct_ptr = ctx.builder.build_bit_cast(env_param, struct_ptr_type);

    // Bind captured variables into lambda scope
    for (i, (var_name, _)) in captures.iter().enumerate() {
        let field_ptr = ctx.builder.build_struct_gep(
            env_struct_type, env_struct_ptr, i as u32, "capture_ptr"
        );
        let field_value = ctx.builder.build_load(field_type, field_ptr, var_name);
        ctx.store_variable(var_name, field_value);
    }

    // Generate lambda body (captured variables now accessible)
    expression.codegen(ctx)
}
```

### 4. Function Calls

**File**: `crates/why_lib/src/codegen/expressions/postfix.rs`

Function calls are handled differently based on the call type:

#### Direct Calls

Named functions called directly use their original ABI:

```rust
if let Some(llvm_function) = ctx.module.get_function(function_name) {
    // Direct call: function(args...)
    ctx.builder.build_call(llvm_function, &args, "")
}
```

#### Indirect Calls

Function-typed expressions are called through closure unpacking:

```rust
fn call_closure(closure_struct: StructValue) {
    // Extract components
    let env_ptr = ctx.extract_closure_env_ptr(closure_struct);
    let fn_ptr = ctx.extract_closure_fn_ptr(closure_struct, target_type);

    // Check if capturing (env_ptr != null)
    if env_ptr != null_env {
        // Capturing closure: call with environment
        let mut call_args = vec![env_ptr.into()];
        call_args.extend(args);
        ctx.builder.build_indirect_call(closure_fn_type, fn_ptr, &call_args, "")
    } else {
        // Non-capturing: call without environment
        ctx.builder.build_indirect_call(standard_fn_type, fn_ptr, &args, "")
    }
}
```

### 5. Variable Access

**File**: `crates/why_lib/src/codegen/expressions/id.rs`

When accessing function-typed variables, the system properly loads closure structs:

```rust
impl CodeGen for Id<ValidatedTypeInformation> {
    fn codegen(&self, ctx: &CodegenContext) -> BasicValueEnum {
        let variable = ctx.find_variable(name);

        match variable {
            BasicValueEnum::PointerValue(ptr) if is_function_type => {
                // Load closure struct from memory
                let closure_type = ctx.get_closure_struct_type();
                ctx.builder.build_load(closure_type, ptr, "").unwrap()
            }
            // ... other cases
        }
    }
}
```

## Memory Management

### Heap Allocation

Captured environments are allocated on the heap using malloc:

```llvm
%env_size = ptrtoint ptr getelementptr (%env_struct_type, ptr null, i32 1) to i64
%env_ptr = call ptr @malloc(i64 %env_size)
```

### Memory Cleanup

**Current Limitation**: The implementation doesn't include automatic memory deallocation. Captured environments remain allocated for the program's lifetime.

**Future Improvement**: Add reference counting or explicit cleanup mechanisms.

## Type System Integration

### Function Type Representation

**File**: `crates/why_lib/src/codegen/statements/function.rs`

Function types in signatures are converted to closure struct types:

```rust
fn build_llvm_function_type_from_own_types(return_type: &Type) -> FunctionType {
    match return_type {
        Type::Function { .. } => {
            // Functions return closure structs
            let closure_struct_type = ctx.get_closure_struct_type();
            closure_struct_type.fn_type(&param_types, false)
        }
        // ... other types
    }
}
```

### Type Conversion Pipeline

1. **Parser**: `(i64) -> i64` syntax
2. **Typechecker**: `Type::Function { params, return_value }`
3. **Codegen**: `{ ptr, ptr }` closure struct in LLVM IR

## Testing and Validation

### Test Cases

The implementation is validated against several test scenarios:

1. **Simple Variable Capture**:
   ```why
   fn get(x: i64): (i64) -> i64 { \(y) => x + y }
   get(1)(42)  // Should return 43
   ```

2. **Struct Field Capture**:
   ```why
   struct P { x: i64; }
   fn foo(p: P): (i64) -> i64 { \(y) => p.x + y }
   ```

3. **Non-Capturing Lambdas**:
   ```why
   let f: (i64) -> i64 = \(x) => x * 2;
   ```

### Generated LLVM IR Example

For the simple capture case, the generated IR includes:

```llvm
; Environment allocation and population
%env_malloc = call ptr @malloc(i64 8)
%env_field_0 = getelementptr inbounds { i64 }, ptr %env_malloc, i32 0, i32 0
store i64 %x, ptr %env_field_0, align 4

; Closure creation
%closure_complete = insertvalue { ptr, ptr } { ptr @lambda_impl_0, ptr undef }, ptr %env_malloc, 1

; Lambda implementation function
define i64 @lambda_impl_0(ptr %env, i64 %y) {
    %capture_x_ptr = getelementptr inbounds { i64 }, ptr %env, i32 0, i32 0
    %capture_x = load i64, ptr %capture_x_ptr, align 4
    %result = add i64 %capture_x, %y
    ret i64 %result
}
```

## Performance Characteristics

### Runtime Overhead

- **Non-capturing lambdas**: Minimal overhead (single indirection)
- **Capturing lambdas**: Heap allocation + environment access overhead
- **Direct function calls**: No overhead (optimized path)

### Memory Usage

- Environment structs: Size equals sum of captured variable sizes
- Closure values: Fixed 16 bytes (two pointers) on 64-bit systems

### Optimization Opportunities

1. **Stack allocation** for non-escaping closures
2. **Environment sharing** for closures with identical captures
3. **Specialization** for common capture patterns

## Debugging and Troubleshooting

### Common Issues

1. **Type Mismatches**: Ensure closure structs are used consistently
2. **Memory Errors**: Verify environment pointer validity
3. **ABI Mismatches**: Check direct vs indirect call paths

### Debugging Tools

- **LLVM IR Inspection**: Examine generated IR for correctness
- **Capture Analysis Logging**: Enable debug output for capture detection
- **Runtime Debugging**: Use debugger to inspect environment structures

### Debug Output

The compiler outputs capture information during compilation:

```
Lambda lambda_7_4_7_17 captures: [("x", Integer)]
```

This helps verify that capture analysis is working correctly.

---

This implementation provides a solid foundation for functional programming patterns in Y while maintaining the performance characteristics expected of a systems programming language.