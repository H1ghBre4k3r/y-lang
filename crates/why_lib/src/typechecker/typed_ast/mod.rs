//! Typed AST layering.
//!
//! This module tree mirrors the parser AST structure but each node now
//! carries either [`TypeInformation`] (during / after inference) or
//! [`ValidatedTypeInformation`] (post validation). The separation into
//! `expression` and (private) `statement` submodules intentionally hides
//! statement internals while exposing expression forms required by
//! downstream stages (e.g. code generation, optimisation passes, tools).
//!
//! The design keeps statement typing logic encapsulated—only the
//! transformed top level statements are returned outward—while allowing
//! fine‑grained handling of expression forms such as lambdas and blocks.
//! Lambda specific capture analysis lives in
//! `expression::lambda` (see exported `get_lambda_captures`).
pub mod expression;
/// Internal typed statement forms (kept private to restrict surface area)
mod statement;
