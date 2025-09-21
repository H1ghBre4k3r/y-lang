# Operations

This section covers implementing Y Lang's arithmetic, logical, and comparison operations using Inkwell, focusing on type-specific instruction selection and proper handling of different numeric types.

## Binary Arithmetic Operations

**Why separate instructions for different types**: LLVM uses distinct instructions for integers vs floats to enable proper optimization, overflow handling, and maintain IEEE 754 compliance for floating-point operations.

### Integer Arithmetic

Y Lang's integer operations map directly to LLVM's integer arithmetic instructions:

```rust
use inkwell::context::Context;
use inkwell::IntPredicate;

let context = Context::create();
let builder = context.create_builder();
let i64_type = context.i64_type();

// Basic arithmetic operations
let left = i64_type.const_int(10, false);
let right = i64_type.const_int(3, false);

// Addition: 10 + 3
let add_result = builder.build_int_add(left, right, "add").unwrap();

// Subtraction: 10 - 3
let sub_result = builder.build_int_sub(left, right, "sub").unwrap();

// Multiplication: 10 * 3
let mul_result = builder.build_int_mul(left, right, "mul").unwrap();

// Division: 10 / 3 (signed)
let div_result = builder.build_int_signed_div(left, right, "div").unwrap();

// Remainder: 10 % 3 (signed)
let rem_result = builder.build_int_signed_rem(left, right, "rem").unwrap();
```

**Generated LLVM IR:**
```llvm
%add = add i64 10, 3
%sub = sub i64 10, 3
%mul = mul i64 10, 3
%div = sdiv i64 10, 3
%rem = srem i64 10, 3
```

**Implementation considerations**:
- Use `signed_div` for Y Lang integers to handle negative numbers correctly
- `unsigned_div` would treat negative numbers as large positive values
- Division by zero behavior: LLVM generates undefined behavior, consider runtime checks

### Floating Point Arithmetic

Floating-point operations use separate instructions with IEEE 754 semantics:

```rust
let f64_type = context.f64_type();
let left_f = f64_type.const_float(10.5);
let right_f = f64_type.const_float(3.2);

// Floating-point arithmetic
let fadd_result = builder.build_float_add(left_f, right_f, "fadd").unwrap();
let fsub_result = builder.build_float_sub(left_f, right_f, "fsub").unwrap();
let fmul_result = builder.build_float_mul(left_f, right_f, "fmul").unwrap();
let fdiv_result = builder.build_float_div(left_f, right_f, "fdiv").unwrap();
let frem_result = builder.build_float_rem(left_f, right_f, "frem").unwrap();
```

**Generated LLVM IR:**
```llvm
%fadd = fadd double 10.5, 3.2
%fsub = fsub double 10.5, 3.2
%fmul = fmul double 10.5, 3.2
%fdiv = fdiv double 10.5, 3.2
%frem = frem double 10.5, 3.2
```

**IEEE 754 special cases**:
- Division by zero produces infinity, not undefined behavior
- Operations with NaN propagate NaN
- Overflow produces infinity rather than wrapping

### Mixed-Type Arithmetic

Y Lang requires explicit type conversion for mixed-type operations:

```rust
// Convert integer to float for mixed arithmetic
let int_val = i64_type.const_int(42, false);
let float_val = f64_type.const_float(3.14);

// Convert int to float
let int_as_float = builder.build_signed_int_to_float(int_val, f64_type, "int_to_float").unwrap();

// Now can perform float arithmetic
let mixed_result = builder.build_float_add(int_as_float, float_val, "mixed_add").unwrap();
```

**Generated LLVM IR:**
```llvm
%int_to_float = sitofp i64 42 to double
%mixed_add = fadd double %int_to_float, 3.14
```

## Comparison Operations

**Why comparison predicates matter**: LLVM uses predicates to specify the exact comparison semantics, handling signed vs unsigned integers and NaN behavior for floats.

### Integer Comparisons

```rust
let left = i64_type.const_int(10, false);
let right = i64_type.const_int(20, false);

// Equality: 10 == 20
let eq = builder.build_int_compare(IntPredicate::EQ, left, right, "eq").unwrap();

// Inequality: 10 != 20
let ne = builder.build_int_compare(IntPredicate::NE, left, right, "ne").unwrap();

// Signed comparisons
let slt = builder.build_int_compare(IntPredicate::SLT, left, right, "slt").unwrap(); // <
let sle = builder.build_int_compare(IntPredicate::SLE, left, right, "sle").unwrap(); // <=
let sgt = builder.build_int_compare(IntPredicate::SGT, left, right, "sgt").unwrap(); // >
let sge = builder.build_int_compare(IntPredicate::SGE, left, right, "sge").unwrap(); // >=

// Unsigned comparisons (if needed)
let ult = builder.build_int_compare(IntPredicate::ULT, left, right, "ult").unwrap();
```

**Generated LLVM IR:**
```llvm
%eq = icmp eq i64 10, 20
%ne = icmp ne i64 10, 20
%slt = icmp slt i64 10, 20
%sle = icmp sle i64 10, 20
%sgt = icmp sgt i64 10, 20
%sge = icmp sge i64 10, 20
%ult = icmp ult i64 10, 20
```

### Floating Point Comparisons

Float comparisons need special handling for NaN values:

```rust
use inkwell::FloatPredicate;

let left_f = f64_type.const_float(10.5);
let right_f = f64_type.const_float(20.3);

// Ordered comparisons (false if either operand is NaN)
let oeq = builder.build_float_compare(FloatPredicate::OEQ, left_f, right_f, "oeq").unwrap();
let olt = builder.build_float_compare(FloatPredicate::OLT, left_f, right_f, "olt").unwrap();
let ole = builder.build_float_compare(FloatPredicate::OLE, left_f, right_f, "ole").unwrap();
let ogt = builder.build_float_compare(FloatPredicate::OGT, left_f, right_f, "ogt").unwrap();
let oge = builder.build_float_compare(FloatPredicate::OGE, left_f, right_f, "oge").unwrap();

// Unordered comparisons (true if either operand is NaN)
let ueq = builder.build_float_compare(FloatPredicate::UEQ, left_f, right_f, "ueq").unwrap();
let une = builder.build_float_compare(FloatPredicate::UNE, left_f, right_f, "une").unwrap();
```

**Generated LLVM IR:**
```llvm
%oeq = fcmp oeq double 10.5, 20.3
%olt = fcmp olt double 10.5, 20.3
%ole = fcmp ole double 10.5, 20.3
%ogt = fcmp ogt double 10.5, 20.3
%oge = fcmp oge double 10.5, 20.3
%ueq = fcmp ueq double 10.5, 20.3
%une = fcmp une double 10.5, 20.3
```

**NaN handling choice**: Most languages use ordered comparisons (O-prefixed) as the default, making `NaN == NaN` false.

## Logical Operations

**Why bitwise vs boolean logic**: Y Lang distinguishes between bitwise operations on integers and logical operations on booleans.

### Boolean Logic

```rust
let bool_type = context.bool_type();
let true_val = bool_type.const_int(1, false);
let false_val = bool_type.const_int(0, false);

// Logical AND: true && false
let and_result = builder.build_and(true_val, false_val, "and").unwrap();

// Logical OR: true || false
let or_result = builder.build_or(true_val, false_val, "or").unwrap();

// Logical NOT: !true
let not_result = builder.build_not(true_val, "not").unwrap();
```

**Generated LLVM IR:**
```llvm
%and = and i1 true, false
%or = or i1 true, false
%not = xor i1 true, true  ; NOT implemented as XOR with all-ones
```

### Short-Circuit Evaluation

Y Lang's `&&` and `||` operators use short-circuit evaluation, requiring control flow:

```rust
// Implementing: a && b (short-circuit)
let a_cond = /* evaluate condition a */;

let and_true_block = context.append_basic_block(function, "and_true");
let and_merge_block = context.append_basic_block(function, "and_merge");

// If a is false, skip evaluating b
builder.build_conditional_branch(a_cond, and_true_block, and_merge_block).unwrap();

// Evaluate b only if a was true
builder.position_at_end(and_true_block);
let b_cond = /* evaluate condition b */;
builder.build_unconditional_branch(and_merge_block).unwrap();

// Merge results with PHI
builder.position_at_end(and_merge_block);
let phi = builder.build_phi(bool_type, "and_result").unwrap();
phi.add_incoming(&[
    (&bool_type.const_int(0, false), /* block where a was false */),
    (&b_cond, and_true_block)
]);
```

**Generated LLVM IR:**
```llvm
br i1 %a_cond, label %and_true, label %and_merge

and_true:
  ; evaluate b_cond
  br label %and_merge

and_merge:
  %and_result = phi i1 [ false, %entry ], [ %b_cond, %and_true ]
```

### Bitwise Operations

Integer bitwise operations for bit manipulation:

```rust
let left = i64_type.const_int(0b1010, false);  // 10 in binary
let right = i64_type.const_int(0b1100, false); // 12 in binary

// Bitwise AND: 1010 & 1100 = 1000
let bit_and = builder.build_and(left, right, "bit_and").unwrap();

// Bitwise OR: 1010 | 1100 = 1110
let bit_or = builder.build_or(left, right, "bit_or").unwrap();

// Bitwise XOR: 1010 ^ 1100 = 0110
let bit_xor = builder.build_xor(left, right, "bit_xor").unwrap();

// Bitwise NOT: ~1010 = ...11110101 (two's complement)
let bit_not = builder.build_not(left, "bit_not").unwrap();

// Left shift: 1010 << 2 = 101000
let shift_left = builder.build_left_shift(left, i64_type.const_int(2, false), "shl").unwrap();

// Right shift (arithmetic): 1010 >> 1 = 101
let shift_right = builder.build_right_shift(left, i64_type.const_int(1, false), true, "shr").unwrap();
```

**Generated LLVM IR:**
```llvm
%bit_and = and i64 10, 12
%bit_or = or i64 10, 12
%bit_xor = xor i64 10, 12
%bit_not = xor i64 10, -1
%shl = shl i64 10, 2
%shr = ashr i64 10, 1  ; arithmetic right shift (preserves sign)
```

## Unary Operations

### Arithmetic Unary Operations

```rust
let value = i64_type.const_int(42, false);
let float_val = f64_type.const_float(3.14);

// Unary minus (negation)
let neg_int = builder.build_int_neg(value, "neg_int").unwrap();
let neg_float = builder.build_float_neg(float_val, "neg_float").unwrap();

// Unary plus (identity - no operation needed)
let pos_int = value; // Just use the value directly
```

**Generated LLVM IR:**
```llvm
%neg_int = sub i64 0, 42      ; Negation as subtraction from zero
%neg_float = fneg double 3.14 ; Direct float negation
```

### Type-Specific Considerations

**Integer overflow behavior**: LLVM integer operations wrap on overflow by default:

```rust
// This will wrap around for large values
let max_val = i64_type.const_int(i64::MAX as u64, false);
let one = i64_type.const_int(1, false);
let overflow = builder.build_int_add(max_val, one, "overflow").unwrap();
// Result wraps to i64::MIN
```

**Overflow detection** (if Y Lang needs it):

```rust
use inkwell::intrinsics::Intrinsic;

// Get overflow-checking intrinsic
let intrinsic = Intrinsic::find("llvm.sadd.with.overflow.i64").unwrap();
let intrinsic_fn = intrinsic.get_declaration(&module, &[i64_type.into()]).unwrap();

// Call with overflow detection
let args = vec![left.into(), right.into()];
let result = builder.build_call(intrinsic_fn, &args, "add_overflow").unwrap();

// Extract result and overflow flag
let sum = builder.build_extract_value(result.try_as_basic_value().left().unwrap().into_struct_value(), 0, "sum").unwrap();
let overflow_flag = builder.build_extract_value(result.try_as_basic_value().left().unwrap().into_struct_value(), 1, "overflow").unwrap();
```

## Operator Precedence Implementation

Y Lang's operator precedence needs careful handling during parsing, but at the LLVM level, operations are explicit:

```rust
// Y Lang: a + b * c
// Parser ensures this becomes: a + (b * c)

let a = i64_type.const_int(5, false);
let b = i64_type.const_int(3, false);
let c = i64_type.const_int(2, false);

// First: b * c
let mul_result = builder.build_int_mul(b, c, "mul").unwrap();

// Then: a + (result)
let final_result = builder.build_int_add(a, mul_result, "add").unwrap();
```

**Generated LLVM IR:**
```llvm
%mul = mul i64 3, 2
%add = add i64 5, %mul
```

## Type Coercion and Promotion

Y Lang may need automatic type promotion in mixed operations:

```rust
// Promoting smaller integers to i64
let i32_type = context.i32_type();
let small_val = i32_type.const_int(100, false);
let large_val = i64_type.const_int(200, false);

// Promote i32 to i64
let promoted = builder.build_int_s_extend(small_val, i64_type, "promoted").unwrap();

// Now can operate
let result = builder.build_int_add(promoted, large_val, "result").unwrap();
```

**Generated LLVM IR:**
```llvm
%promoted = sext i32 100 to i64
%result = add i64 %promoted, 200
```

## Advanced Operation Patterns

### Conditional Operations (Ternary-like)

LLVM's `select` instruction provides conditional value selection:

```rust
let condition = bool_type.const_int(1, false); // true
let true_val = i64_type.const_int(42, false);
let false_val = i64_type.const_int(24, false);

// condition ? true_val : false_val
let selected = builder.build_select(condition, true_val, false_val, "select").unwrap();
```

**Generated LLVM IR:**
```llvm
%select = select i1 true, i64 42, i64 24
```

### Pointer Arithmetic

For array indexing and memory operations:

```rust
// Array element access using GEP
let array_type = i64_type.array_type(10);
let array_ptr = builder.build_alloca(array_type, "array").unwrap();

let zero = i64_type.const_int(0, false);
let index = i64_type.const_int(5, false);

let element_ptr = unsafe {
    builder.build_gep(array_type, array_ptr, &[zero, index], "elem_ptr").unwrap()
};
```

**Generated LLVM IR:**
```llvm
%array = alloca [10 x i64]
%elem_ptr = getelementptr [10 x i64], ptr %array, i64 0, i64 5
```

## Error Handling and Validation

### Runtime Division by Zero Checks

```rust
fn safe_divide<'ctx>(
    builder: &Builder<'ctx>,
    left: IntValue<'ctx>,
    right: IntValue<'ctx>,
    context: &'ctx Context
) -> IntValue<'ctx> {
    let i64_type = context.i64_type();
    let zero = i64_type.const_zero();

    // Check if divisor is zero
    let is_zero = builder.build_int_compare(
        IntPredicate::EQ,
        right,
        zero,
        "is_zero"
    ).unwrap();

    // Use select to avoid division by zero
    let safe_divisor = builder.build_select(
        is_zero,
        i64_type.const_int(1, false), // Use 1 if zero (or handle error differently)
        right,
        "safe_divisor"
    ).unwrap();

    builder.build_int_signed_div(left, safe_divisor.into_int_value(), "safe_div").unwrap()
}
```

### Type Validation for Operations

```rust
fn validate_arithmetic_types<'ctx>(
    left_type: BasicTypeEnum<'ctx>,
    right_type: BasicTypeEnum<'ctx>
) -> Result<BasicTypeEnum<'ctx>, String> {
    match (left_type, right_type) {
        (BasicTypeEnum::IntType(l), BasicTypeEnum::IntType(r)) if l == r => Ok(left_type),
        (BasicTypeEnum::FloatType(l), BasicTypeEnum::FloatType(r)) if l == r => Ok(left_type),
        _ => Err(format!("Type mismatch in arithmetic: {:?} vs {:?}", left_type, right_type))
    }
}
```

## Performance Optimization

### Constant Folding

LLVM automatically folds constants, but be aware of the pattern:

```rust
// This gets computed at compile time
let a = i64_type.const_int(10, false);
let b = i64_type.const_int(20, false);
let c = a.const_add(b); // Immediate result: 30

// This requires runtime computation
let runtime_a = builder.build_load(i64_type, some_ptr, "a").unwrap().into_int_value();
let runtime_result = builder.build_int_add(runtime_a, b, "result").unwrap();
```

### Strength Reduction

Some operations can be optimized by LLVM:

```rust
// Multiplication by power of 2 -> shift
let val = i64_type.const_int(42, false);
let mul_by_8 = builder.build_int_mul(val, i64_type.const_int(8, false), "mul8").unwrap();
// LLVM may optimize this to: shl i64 %val, 3
```

This comprehensive coverage of operations provides the foundation for implementing Y Lang's expression evaluation in LLVM, handling type safety, proper instruction selection, and performance considerations.
