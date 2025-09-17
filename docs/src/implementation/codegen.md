# Code Generation

This chapter describes the LLVM-based code generation backend for Y, focusing on the uniform function/closure ABI and how expressions lower to IR.

- Uniform closure type: `{ ptr, ptr }` where field0=erased function pointer, field1=environment pointer (nullable)
- Capturing lambda impl signature: `(ptr env, params...) -> ret`
- Non‑capturing lambda signature: `(params...) -> ret` and env=null in closure
- Named functions stored as raw pointers; wrapped lazily into closure structs when used as values

Key modules and functions:
- crates/why_lib/src/codegen/mod.rs
  - get_closure_struct_type
  - create_closure_impl_fn_type
  - build_closure_value
  - extract_closure_fn_ptr
  - extract_closure_env_ptr
  - store_function, store_lambda
- crates/why_lib/src/codegen/expressions/lambda.rs
  - create_and_populate_environment (heap alloc via malloc(i64))
  - generate_lambda_body (bind captures/params, return)
- crates/why_lib/src/codegen/expressions/postfix.rs
  - codegen_call (direct/method/closure dispatch, indirect call typing)
- crates/why_lib/src/codegen/expressions/id.rs
  - identifier resolution; loading closure structs and special-casing strings/functions

ABI invariants:
- Closure struct has exactly two pointers
- Fn ptr non‑null; env ptr may be null
- Capturing impls take env first; non‑capturing impls do not
- Indirect calls type the call via FunctionType; fn pointer remains generic

Minimal IR sketches:
```llvm
; Construct non‑capturing closure
%closure = insertvalue { ptr, ptr } undef, ptr bitcast (void (...)* @lambda_impl to ptr), 0
%closure1 = insertvalue { ptr, ptr } %closure, ptr null, 1

; Indirect call (capturing)
%fn_i8 = extractvalue { ptr, ptr } %closure, 0
%env   = extractvalue { ptr, ptr } %closure, 1
%res   = call i64 %fn_typed(ptr %env, i64 %arg0)
```

Memory management:
- Environments allocated with malloc; no deallocation yet (leak semantics)
- Future: reference counting/arena; escape analysis for stack promotion

See also: [Closure Implementation](./closures.md) for end‑to‑end details and examples.
