# Closures Roadmap

This document tracks the design and phased implementation plan to add proper closure support to Y’s compiler (why_lib).

## Goals
- Represent first-class functions uniformly so both named functions and lambdas can carry environments.
- Capture outer variables by value at lambda-creation time (heap-allocated env), so returned lambdas remain valid.
- Keep direct calls to named functions efficient and backward-compatible.

## Representation
- Closure value layout: a 2-field struct `{ i8* fn_ptr, i8* env_ptr }` used at all function-typed expression boundaries (params, returns, locals, struct fields, arrays of functions).
- Lambda implementation function signature: `(i8* env, params...) -> ret`.
- Named functions retain their existing ABI `(params...) -> ret` for direct calls. When used as values, they are wrapped into a closure with `env_ptr = null`.

## Call Convention
- Indirect calls (calling a function-typed expression):
  - Unpack `{fn_ptr, env_ptr}`.
  - Bitcast `fn_ptr` to the closure-impl function type `(i8*, params...) -> ret`.
  - Call with `env_ptr` as the hidden first argument, followed by the user arguments.
- Direct calls (to known module functions) stay as is and do not pass an env.

## Lambda Codegen
- At lambda expression site:
  - Compute capture set (free variables minus parameters and inner bindings).
  - Build an env struct type with fields matching captured variable types.
  - Allocate env on the heap and copy captured values by value into it. Pointer-like values (strings, arrays, function values) copy the pointer value.
  - Emit the lambda implementation function `(i8* env, params...) -> ret`.
    - Inside, bitcast `env` to the env struct pointer and bind captured fields into the lambda scope so regular identifier codegen works unchanged.
  - Return a closure pair `{fn_ptr_as_i8*, env_ptr_as_i8*}`.

## Named Functions as Values
- Continue emitting named functions with their current ABI.
- When a named function is used as a value (e.g., stored to a variable or passed as an argument), treat it as a closure with `env_ptr = null`.
- For indirect calls that receive such a closure, the call site still goes through the closure convention; for direct calls we keep the current fast path.

## Phases

### Phase 1: IR & Codegen Adaptation (MVP)
- Add helpers in codegen to:
  - Get the canonical closure struct type `{i8*, i8*}`.
  - Build closure-impl function types `(i8*, params...) -> ret`.
  - Construct closure values from a function pointer and env pointer.
- Change `Type::Function` lowering to the closure struct type.
- Update lambda codegen to:
  - Compute captures (codegen-side), allocate and populate env by value.
  - Emit impl function with `(i8*, params...) -> ret`.
  - Return a closure value.
- Update indirect-call sites to:
  - Unpack closure, bitcast fn pointer, prepend env, call.
- Update storage of named functions in the variable scope to store a closure value with `env = null`.
- Validate: `examples/closure.why` returns 43; `examples/lambda.why` still works.

### Phase 2: Typechecker Integration & Tests
- Move capture analysis to typechecker and attach capture metadata to lambdas.
- Extend tests:
  - Non-capturing vs capturing lambdas
  - Nested closures
  - Shadowing
  - Functions as values
  - Closures in structs/arrays and as returns/params

### Phase 3: Optimizations & Cleanup
- Optimize non-capturing lambdas and named functions to elide env allocation (keep `env = null`).
- Consider stack env for provably non-escaping closures.
- Explore refcounting or ownership for pointer-like captures if/when strings or heap aggregates need ownership semantics.

## Pitfalls & Notes
- Lifetime safety: Use heap for env; do not capture stack addresses.
- Mutability semantics: Captures are by value; outer mutations after creation are not reflected in the closure (future work may add by-ref captures).
- Type and IR safety: Always bitcast `fn_ptr` to `(i8*, params...) -> ret` before indirect calls.
- Keep direct vs indirect paths distinct to avoid ABI mismatches.
