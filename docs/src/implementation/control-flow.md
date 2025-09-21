# Control Flow

This section covers implementing Y Lang's control flow constructs using Inkwell, focusing on conditional expressions, loops, blocks, and advanced control patterns using LLVM's basic block system.

## Conditional Expressions (If-Else)

**Why basic blocks for conditionals**: LLVM represents control flow as graphs of basic blocks connected by branches. Each path through an if-else requires separate basic blocks to maintain proper SSA form.

### Simple If Expression

Y Lang's if expressions evaluate to values, requiring careful handling of result values:

```rust
use inkwell::context::Context;
use inkwell::IntPredicate;

let context = Context::create();
let module = context.create_module("conditionals");
let builder = context.create_builder();

let i64_type = context.i64_type();
let bool_type = context.bool_type();

// Function context
let fn_type = i64_type.fn_type(&[], false);
let function = module.add_function("test_if", fn_type, None);

// Create basic blocks
let entry_block = context.append_basic_block(function, "entry");
let then_block = context.append_basic_block(function, "then");
let else_block = context.append_basic_block(function, "else");
let merge_block = context.append_basic_block(function, "merge");

builder.position_at_end(entry_block);

// Evaluate condition: x > 10
let x = i64_type.const_int(15, false);
let ten = i64_type.const_int(10, false);
let condition = builder.build_int_compare(
    IntPredicate::SGT,
    x,
    ten,
    "x_gt_10"
).unwrap();

// Branch based on condition
builder.build_conditional_branch(condition, then_block, else_block).unwrap();

// Then branch: if x > 10 { 42 }
builder.position_at_end(then_block);
let then_value = i64_type.const_int(42, false);
builder.build_unconditional_branch(merge_block).unwrap();

// Else branch: else { 0 }
builder.position_at_end(else_block);
let else_value = i64_type.const_int(0, false);
builder.build_unconditional_branch(merge_block).unwrap();

// Merge block: combine results with PHI
builder.position_at_end(merge_block);
let phi = builder.build_phi(i64_type, "if_result").unwrap();
phi.add_incoming(&[
    (&then_value, then_block),
    (&else_value, else_block)
]);

builder.build_return(Some(&phi.as_basic_value())).unwrap();
```

**Generated LLVM IR:**
```llvm
define i64 @test_if() {
entry:
  %x_gt_10 = icmp sgt i64 15, 10
  br i1 %x_gt_10, label %then, label %else

then:
  br label %merge

else:
  br label %merge

merge:
  %if_result = phi i64 [ 42, %then ], [ 0, %else ]
  ret i64 %if_result
}
```

**Implementation steps**:
1. Create basic blocks for each control path (then, else, merge)
2. Evaluate condition in entry block
3. Use conditional branch to select path
4. Compute branch-specific values
5. Merge results using PHI node
6. Continue with merged value

### Nested If Expressions

Y Lang supports nested conditionals, requiring additional basic blocks:

```rust
// Y Lang: if x > 0 { if y > 0 { 1 } else { 2 } } else { 3 }

let outer_then_block = context.append_basic_block(function, "outer_then");
let inner_then_block = context.append_basic_block(function, "inner_then");
let inner_else_block = context.append_basic_block(function, "inner_else");
let inner_merge_block = context.append_basic_block(function, "inner_merge");
let outer_else_block = context.append_basic_block(function, "outer_else");
let final_merge_block = context.append_basic_block(function, "final_merge");

// Outer condition
builder.position_at_end(entry_block);
let x = i64_type.const_int(5, false);
let zero = i64_type.const_zero();
let x_gt_0 = builder.build_int_compare(IntPredicate::SGT, x, zero, "x_gt_0").unwrap();
builder.build_conditional_branch(x_gt_0, outer_then_block, outer_else_block).unwrap();

// Outer then: inner conditional
builder.position_at_end(outer_then_block);
let y = i64_type.const_int(-3, false);
let y_gt_0 = builder.build_int_compare(IntPredicate::SGT, y, zero, "y_gt_0").unwrap();
builder.build_conditional_branch(y_gt_0, inner_then_block, inner_else_block).unwrap();

// Inner then
builder.position_at_end(inner_then_block);
let inner_then_val = i64_type.const_int(1, false);
builder.build_unconditional_branch(inner_merge_block).unwrap();

// Inner else
builder.position_at_end(inner_else_block);
let inner_else_val = i64_type.const_int(2, false);
builder.build_unconditional_branch(inner_merge_block).unwrap();

// Inner merge
builder.position_at_end(inner_merge_block);
let inner_phi = builder.build_phi(i64_type, "inner_result").unwrap();
inner_phi.add_incoming(&[
    (&inner_then_val, inner_then_block),
    (&inner_else_val, inner_else_block)
]);
builder.build_unconditional_branch(final_merge_block).unwrap();

// Outer else
builder.position_at_end(outer_else_block);
let outer_else_val = i64_type.const_int(3, false);
builder.build_unconditional_branch(final_merge_block).unwrap();

// Final merge
builder.position_at_end(final_merge_block);
let final_phi = builder.build_phi(i64_type, "final_result").unwrap();
final_phi.add_incoming(&[
    (&inner_phi.as_basic_value(), inner_merge_block),
    (&outer_else_val, outer_else_block)
]);
```

**Generated LLVM IR:**
```llvm
define i64 @nested_if() {
entry:
  %x_gt_0 = icmp sgt i64 5, 0
  br i1 %x_gt_0, label %outer_then, label %outer_else

outer_then:
  %y_gt_0 = icmp sgt i64 -3, 0
  br i1 %y_gt_0, label %inner_then, label %inner_else

inner_then:
  br label %inner_merge

inner_else:
  br label %inner_merge

inner_merge:
  %inner_result = phi i64 [ 1, %inner_then ], [ 2, %inner_else ]
  br label %final_merge

outer_else:
  br label %final_merge

final_merge:
  %final_result = phi i64 [ %inner_result, %inner_merge ], [ 3, %outer_else ]
  ret i64 %final_result
}
```

## While Loops

**Why loops need PHI nodes**: Loop variables change over iterations, requiring PHI nodes to merge values from different loop iterations while maintaining SSA form.

### Basic While Loop

```rust
// Y Lang: while i < 10 { i = i + 1; }

let loop_header = context.append_basic_block(function, "loop_header");
let loop_body = context.append_basic_block(function, "loop_body");
let loop_exit = context.append_basic_block(function, "loop_exit");

// Initialize loop variable
builder.position_at_end(entry_block);
let initial_i = i64_type.const_int(0, false);
builder.build_unconditional_branch(loop_header).unwrap();

// Loop header: check condition
builder.position_at_end(loop_header);
let i_phi = builder.build_phi(i64_type, "i").unwrap();
i_phi.add_incoming(&[(&initial_i, entry_block)]);

let ten = i64_type.const_int(10, false);
let condition = builder.build_int_compare(
    IntPredicate::SLT,
    i_phi.as_basic_value().into_int_value(),
    ten,
    "i_lt_10"
).unwrap();

builder.build_conditional_branch(condition, loop_body, loop_exit).unwrap();

// Loop body: increment i
builder.position_at_end(loop_body);
let current_i = i_phi.as_basic_value().into_int_value();
let one = i64_type.const_int(1, false);
let next_i = builder.build_int_add(current_i, one, "next_i").unwrap();

// Add back-edge to PHI
i_phi.add_incoming(&[(&next_i, loop_body)]);
builder.build_unconditional_branch(loop_header).unwrap();

// Loop exit
builder.position_at_end(loop_exit);
builder.build_return(None).unwrap();
```

**Generated LLVM IR:**
```llvm
define void @while_loop() {
entry:
  br label %loop_header

loop_header:
  %i = phi i64 [ 0, %entry ], [ %next_i, %loop_body ]
  %i_lt_10 = icmp slt i64 %i, 10
  br i1 %i_lt_10, label %loop_body, label %loop_exit

loop_body:
  %next_i = add i64 %i, 1
  br label %loop_header

loop_exit:
  ret void
}
```

### While Loop with Complex Body

```rust
// Y Lang: while x > 0 { if x % 2 == 0 { x = x / 2 } else { x = x * 3 + 1 } }

let loop_header = context.append_basic_block(function, "loop_header");
let loop_body = context.append_basic_block(function, "loop_body");
let even_branch = context.append_basic_block(function, "even");
let odd_branch = context.append_basic_block(function, "odd");
let body_merge = context.append_basic_block(function, "body_merge");
let loop_exit = context.append_basic_block(function, "loop_exit");

// Initialize
builder.position_at_end(entry_block);
let initial_x = i64_type.const_int(7, false);
builder.build_unconditional_branch(loop_header).unwrap();

// Loop header
builder.position_at_end(loop_header);
let x_phi = builder.build_phi(i64_type, "x").unwrap();
x_phi.add_incoming(&[(&initial_x, entry_block)]);

let zero = i64_type.const_zero();
let x_gt_0 = builder.build_int_compare(
    IntPredicate::SGT,
    x_phi.as_basic_value().into_int_value(),
    zero,
    "x_gt_0"
).unwrap();
builder.build_conditional_branch(x_gt_0, loop_body, loop_exit).unwrap();

// Loop body: check if even
builder.position_at_end(loop_body);
let current_x = x_phi.as_basic_value().into_int_value();
let two = i64_type.const_int(2, false);
let remainder = builder.build_int_signed_rem(current_x, two, "remainder").unwrap();
let is_even = builder.build_int_compare(
    IntPredicate::EQ,
    remainder,
    zero,
    "is_even"
).unwrap();
builder.build_conditional_branch(is_even, even_branch, odd_branch).unwrap();

// Even branch: x = x / 2
builder.position_at_end(even_branch);
let x_div_2 = builder.build_int_signed_div(current_x, two, "x_div_2").unwrap();
builder.build_unconditional_branch(body_merge).unwrap();

// Odd branch: x = x * 3 + 1
builder.position_at_end(odd_branch);
let three = i64_type.const_int(3, false);
let one = i64_type.const_int(1, false);
let x_mul_3 = builder.build_int_mul(current_x, three, "x_mul_3").unwrap();
let x_mul_3_plus_1 = builder.build_int_add(x_mul_3, one, "x_mul_3_plus_1").unwrap();
builder.build_unconditional_branch(body_merge).unwrap();

// Merge body results
builder.position_at_end(body_merge);
let new_x_phi = builder.build_phi(i64_type, "new_x").unwrap();
new_x_phi.add_incoming(&[
    (&x_div_2, even_branch),
    (&x_mul_3_plus_1, odd_branch)
]);

// Add back-edge
x_phi.add_incoming(&[(&new_x_phi.as_basic_value(), body_merge)]);
builder.build_unconditional_branch(loop_header).unwrap();

// Exit
builder.position_at_end(loop_exit);
builder.build_return(None).unwrap();
```

## Blocks and Scoping

**Why blocks matter**: Y Lang blocks create lexical scopes and can return values. LLVM handles this through careful basic block organization and variable lifetime management.

### Simple Block Expression

```rust
// Y Lang: { let x = 10; let y = 20; x + y }

let block_entry = context.append_basic_block(function, "block_entry");
let block_exit = context.append_basic_block(function, "block_exit");

builder.position_at_end(entry_block);
builder.build_unconditional_branch(block_entry).unwrap();

// Block body
builder.position_at_end(block_entry);

// let x = 10;
let x_alloca = builder.build_alloca(i64_type, "x").unwrap();
let ten = i64_type.const_int(10, false);
builder.build_store(x_alloca, ten).unwrap();

// let y = 20;
let y_alloca = builder.build_alloca(i64_type, "y").unwrap();
let twenty = i64_type.const_int(20, false);
builder.build_store(y_alloca, twenty).unwrap();

// x + y (block result)
let x_val = builder.build_load(i64_type, x_alloca, "x_val").unwrap();
let y_val = builder.build_load(i64_type, y_alloca, "y_val").unwrap();
let block_result = builder.build_int_add(
    x_val.into_int_value(),
    y_val.into_int_value(),
    "block_result"
).unwrap();

builder.build_unconditional_branch(block_exit).unwrap();

// Block exit: return result
builder.position_at_end(block_exit);
builder.build_return(Some(&block_result)).unwrap();
```

**Generated LLVM IR:**
```llvm
define i64 @block_expression() {
entry:
  br label %block_entry

block_entry:
  %x = alloca i64
  store i64 10, ptr %x
  %y = alloca i64
  store i64 20, ptr %y
  %x_val = load i64, ptr %x
  %y_val = load i64, ptr %y
  %block_result = add i64 %x_val, %y_val
  br label %block_exit

block_exit:
  ret i64 %block_result
}
```

### Nested Blocks with Shadowing

```rust
// Y Lang: { let x = 1; { let x = 2; x } }

let outer_block = context.append_basic_block(function, "outer_block");
let inner_block = context.append_basic_block(function, "inner_block");
let inner_exit = context.append_basic_block(function, "inner_exit");
let outer_exit = context.append_basic_block(function, "outer_exit");

builder.position_at_end(entry_block);
builder.build_unconditional_branch(outer_block).unwrap();

// Outer block
builder.position_at_end(outer_block);
let outer_x_alloca = builder.build_alloca(i64_type, "outer_x").unwrap();
let one = i64_type.const_int(1, false);
builder.build_store(outer_x_alloca, one).unwrap();

builder.build_unconditional_branch(inner_block).unwrap();

// Inner block (shadows outer x)
builder.position_at_end(inner_block);
let inner_x_alloca = builder.build_alloca(i64_type, "inner_x").unwrap();
let two = i64_type.const_int(2, false);
builder.build_store(inner_x_alloca, two).unwrap();

// Inner block result: inner x
let inner_x_val = builder.build_load(i64_type, inner_x_alloca, "inner_x_val").unwrap();
builder.build_unconditional_branch(inner_exit).unwrap();

// Inner exit
builder.position_at_end(inner_exit);
builder.build_unconditional_branch(outer_exit).unwrap();

// Outer exit: return inner block result
builder.position_at_end(outer_exit);
builder.build_return(Some(&inner_x_val)).unwrap();
```

**Generated LLVM IR:**
```llvm
define i64 @nested_blocks() {
entry:
  br label %outer_block

outer_block:
  %outer_x = alloca i64
  store i64 1, ptr %outer_x
  br label %inner_block

inner_block:
  %inner_x = alloca i64
  store i64 2, ptr %inner_x
  %inner_x_val = load i64, ptr %inner_x
  br label %inner_exit

inner_exit:
  br label %outer_exit

outer_exit:
  ret i64 %inner_x_val
}
```

## Advanced Control Flow Patterns

### Early Return from Blocks

Y Lang allows early returns from nested contexts:

```rust
// Y Lang: { if condition { return 42; } other_computation() }

let block_start = context.append_basic_block(function, "block_start");
let check_condition = context.append_basic_block(function, "check_condition");
let early_return = context.append_basic_block(function, "early_return");
let continue_block = context.append_basic_block(function, "continue_block");
let block_end = context.append_basic_block(function, "block_end");

builder.position_at_end(entry_block);
builder.build_unconditional_branch(block_start).unwrap();

builder.position_at_end(block_start);
builder.build_unconditional_branch(check_condition).unwrap();

// Check condition for early return
builder.position_at_end(check_condition);
let condition = bool_type.const_int(1, false); // true for example
builder.build_conditional_branch(condition, early_return, continue_block).unwrap();

// Early return path
builder.position_at_end(early_return);
let early_value = i64_type.const_int(42, false);
builder.build_return(Some(&early_value)).unwrap();

// Continue with normal computation
builder.position_at_end(continue_block);
let other_result = i64_type.const_int(100, false);
builder.build_unconditional_branch(block_end).unwrap();

// Block end
builder.position_at_end(block_end);
builder.build_return(Some(&other_result)).unwrap();
```

### Break and Continue (for future loop constructs)

Pattern for implementing break/continue in loops:

```rust
// Y Lang: while condition { if should_break { break; } if should_continue { continue; } body; }

let loop_header = context.append_basic_block(function, "loop_header");
let loop_body = context.append_basic_block(function, "loop_body");
let check_break = context.append_basic_block(function, "check_break");
let check_continue = context.append_basic_block(function, "check_continue");
let loop_body_end = context.append_basic_block(function, "loop_body_end");
let loop_exit = context.append_basic_block(function, "loop_exit");

// Loop header with condition check
builder.position_at_end(loop_header);
let condition = bool_type.const_int(1, false); // Placeholder condition
builder.build_conditional_branch(condition, loop_body, loop_exit).unwrap();

// Loop body start
builder.position_at_end(loop_body);
builder.build_unconditional_branch(check_break).unwrap();

// Check for break
builder.position_at_end(check_break);
let should_break = bool_type.const_int(0, false); // false for example
builder.build_conditional_branch(should_break, loop_exit, check_continue).unwrap();

// Check for continue
builder.position_at_end(check_continue);
let should_continue = bool_type.const_int(0, false); // false for example
builder.build_conditional_branch(should_continue, loop_header, loop_body_end).unwrap();

// Rest of loop body
builder.position_at_end(loop_body_end);
// ... other loop body operations ...
builder.build_unconditional_branch(loop_header).unwrap();

// Loop exit
builder.position_at_end(loop_exit);
builder.build_return(None).unwrap();
```

## Control Flow with Variables

### Loop Variables and Mutation

```rust
// Y Lang: let mut sum = 0; let mut i = 1; while i <= 10 { sum = sum + i; i = i + 1; } sum

let loop_header = context.append_basic_block(function, "loop_header");
let loop_body = context.append_basic_block(function, "loop_body");
let loop_exit = context.append_basic_block(function, "loop_exit");

// Initialize variables
builder.position_at_end(entry_block);
let sum_alloca = builder.build_alloca(i64_type, "sum").unwrap();
let i_alloca = builder.build_alloca(i64_type, "i").unwrap();

let zero = i64_type.const_zero();
let one = i64_type.const_int(1, false);
builder.build_store(sum_alloca, zero).unwrap();
builder.build_store(i_alloca, one).unwrap();

builder.build_unconditional_branch(loop_header).unwrap();

// Loop condition: i <= 10
builder.position_at_end(loop_header);
let current_i = builder.build_load(i64_type, i_alloca, "current_i").unwrap();
let ten = i64_type.const_int(10, false);
let i_le_10 = builder.build_int_compare(
    IntPredicate::SLE,
    current_i.into_int_value(),
    ten,
    "i_le_10"
).unwrap();
builder.build_conditional_branch(i_le_10, loop_body, loop_exit).unwrap();

// Loop body: sum = sum + i; i = i + 1;
builder.position_at_end(loop_body);
let current_sum = builder.build_load(i64_type, sum_alloca, "current_sum").unwrap();
let current_i_body = builder.build_load(i64_type, i_alloca, "current_i_body").unwrap();

// sum = sum + i
let new_sum = builder.build_int_add(
    current_sum.into_int_value(),
    current_i_body.into_int_value(),
    "new_sum"
).unwrap();
builder.build_store(sum_alloca, new_sum).unwrap();

// i = i + 1
let new_i = builder.build_int_add(
    current_i_body.into_int_value(),
    one,
    "new_i"
).unwrap();
builder.build_store(i_alloca, new_i).unwrap();

builder.build_unconditional_branch(loop_header).unwrap();

// Loop exit: return sum
builder.position_at_end(loop_exit);
let final_sum = builder.build_load(i64_type, sum_alloca, "final_sum").unwrap();
builder.build_return(Some(&final_sum)).unwrap();
```

**Generated LLVM IR:**
```llvm
define i64 @sum_loop() {
entry:
  %sum = alloca i64
  %i = alloca i64
  store i64 0, ptr %sum
  store i64 1, ptr %i
  br label %loop_header

loop_header:
  %current_i = load i64, ptr %i
  %i_le_10 = icmp sle i64 %current_i, 10
  br i1 %i_le_10, label %loop_body, label %loop_exit

loop_body:
  %current_sum = load i64, ptr %sum
  %current_i_body = load i64, ptr %i
  %new_sum = add i64 %current_sum, %current_i_body
  store i64 %new_sum, ptr %sum
  %new_i = add i64 %current_i_body, 1
  store i64 %new_i, ptr %i
  br label %loop_header

loop_exit:
  %final_sum = load i64, ptr %sum
  ret i64 %final_sum
}
```

## Error Handling in Control Flow

### Safe Condition Evaluation

```rust
fn safe_conditional_branch<'ctx>(
    builder: &Builder<'ctx>,
    condition: IntValue<'ctx>,
    then_block: BasicBlock<'ctx>,
    else_block: BasicBlock<'ctx>
) -> Result<(), String> {
    if condition.get_type().get_bit_width() != 1 {
        return Err(format!(
            "Condition must be i1, got i{}",
            condition.get_type().get_bit_width()
        ));
    }

    builder.build_conditional_branch(condition, then_block, else_block)
        .map_err(|e| format!("Failed to build conditional branch: {}", e))?;

    Ok(())
}
```

### PHI Node Validation

```rust
fn validate_phi_node<'ctx>(
    phi: PhiValue<'ctx>,
    expected_type: BasicTypeEnum<'ctx>
) -> Result<(), String> {
    if phi.as_basic_value().get_type() != expected_type {
        return Err(format!(
            "PHI type mismatch: expected {:?}, got {:?}",
            expected_type,
            phi.as_basic_value().get_type()
        ));
    }

    if phi.count_incoming() == 0 {
        return Err("PHI node has no incoming values".to_string());
    }

    Ok(())
}
```

## Optimization Considerations

### Minimizing Basic Blocks

```rust
// Prefer this: direct value computation when possible
let condition = bool_type.const_int(1, false);
let result = builder.build_select(
    condition,
    i64_type.const_int(42, false),
    i64_type.const_int(0, false),
    "conditional_result"
).unwrap();

// Over this: creating basic blocks for simple conditionals
// (Only use basic blocks when necessary for complex control flow)
```

### Loop Optimization Hints

```rust
// Mark loop headers for optimization
use inkwell::attributes::{Attribute, AttributeLoc};

let loop_header = context.append_basic_block(function, "loop_header");
// LLVM can automatically detect loops, but explicit marking helps
```

### Dead Code Elimination

```rust
// Ensure all basic blocks are reachable
fn validate_cfg<'ctx>(function: FunctionValue<'ctx>) -> Result<(), String> {
    for block in function.get_basic_blocks() {
        if block.get_terminator().is_none() {
            return Err(format!(
                "Basic block '{}' has no terminator",
                block.get_name().to_string_lossy()
            ));
        }
    }
    Ok(())
}
```

This comprehensive coverage of control flow provides the foundation for implementing Y Lang's conditional expressions, loops, and blocks in LLVM, emphasizing the proper use of basic blocks, PHI nodes, and SSA form maintenance.
