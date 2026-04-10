//! Proc macro crate for the UOR Foundation.
//!
//! Provides the `uor!` macro that parses the EBNF term grammar at compile time,
//! producing typed `Term` ASTs in the `uor_foundation::enforcement` module.
//!
//! # Usage
//!
//! ```rust,ignore
//! use uor_foundation::uor;
//!
//! // Type declaration (named-argument constraint syntax)
//! let pixel = uor! { type Pixel { ResidueConstraint(modulus: 256, residue: 255); } };
//!
//! // Term expression
//! let sum = uor! { add(mul(3, 5), 7) };
//!
//! // Assertion (ground — checked at compile time)
//! uor! { assert add(1, 2) = 3; };
//! ```

#![deny(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    missing_docs,
    clippy::missing_errors_doc
)]

mod address;
mod codegen;
mod lexer;
mod parser;
mod surface;

use proc_macro::TokenStream;

/// The UOR term language DSL macro.
///
/// Parses EBNF surface syntax at compile time and produces typed `Term` ASTs
/// in the `uor_foundation::enforcement` module. The macro handles:
///
/// - **Term expressions**: `add(mul(3, 5), 7)` — operation applications
/// - **Type declarations**: `type Pixel { ResidueConstraint(modulus: 256, residue: 255); }` — constrained types
/// - **Bindings**: `let x : Pixel = add(0, 0);` — named term bindings
/// - **Assertions**: `assert lhs = rhs;` — ground assertions checked at compile time
/// - **Effect declarations**: `effect Name { target: {0,1}; delta: 0; commutes: true; }` — generic props
/// - **Boundary declarations**: `source name : Type via grounding;`
/// - **Quantum literals**: `42@Q7` — level-annotated integers
/// - **Lift/Project**: `lift(x, Q3)`, `project(y, Q0)` — level transitions
/// - **Match**: `match x { pred => expr; otherwise => expr; }`
/// - **Recursion**: `recurse f(n) measure n base is_zero => 1 step => mul(n, f(pred(n)))`
/// - **Streams**: `unfold nat : Successor from 0`
///
/// # Examples
///
/// ```rust,ignore
/// use uor_foundation::uor;
///
/// // Term expressions produce a TermArena with the expression tree.
/// let sum = uor! { add(mul(3, 5), 7) };
///
/// // Quantum-annotated literals tag a value at a specific ring width.
/// let wide = uor! { 144115188075855617@Q7 };
///
/// // Type declarations define constrained types.
/// let pixel = uor! {
///     type Pixel {
///         ResidueConstraint(modulus: 256, residue: 255);
///         HammingConstraint(hammingBound: 8);
///         DepthConstraint(minDepth: 0, maxDepth: 1);
///     }
/// };
///
/// // Bindings carry surface syntax and content addresses.
/// let origin = uor! { let origin : Pixel = add(0, 0); };
///
/// // Ground assertions are checked at COMPILE TIME.
/// uor! { assert add(1, 2) = 3; };
/// uor! { assert mul(3, 5) = 15; };
///
/// // Effect declarations register fiber-targeted effects.
/// let blit = uor! {
///     effect Blit {
///         target: {0, 1, 2, 3};
///         delta: 0;
///         commutes: true;
///     }
/// };
///
/// // Boundary declarations define data sources and sinks.
/// uor! { source pixel_in  : Pixel via sRGB; };
/// uor! { sink   pixel_out : Pixel via DisplayP3; };
///
/// // Lift/Project handle level transitions explicitly.
/// let widened  = uor! { lift(x, Q3) };
/// let narrowed = uor! { project(y, Q0) };
///
/// // Match expressions with pattern arms and a required otherwise arm.
/// let clamped = uor! {
///     match x {
///         is_negative => 0;
///         exceeds_max => 255;
///         otherwise => x;
///     }
/// };
///
/// // Bounded recursion with a descent measure and base case.
/// let factorial = uor! {
///     recurse fact(n)
///         measure n
///         base is_zero => 1
///         step => mul(n, fact(pred(n)))
/// };
///
/// // Stream construction via unfold (coinductive).
/// let naturals = uor! { unfold nat : Successor from 0 };
/// ```
///
/// # Errors
///
/// Parse errors produce `compile_error!()` at the macro call site.
/// The error message includes the unexpected token:
///
/// ```text
/// uor! { add(1, ) }
/// // error: uor! parse error: Unexpected token in expression: RParen
/// ```
///
/// Ground assertions that fail ring evaluation also produce compile errors:
///
/// ```text
/// uor! { assert add(1, 2) = 4; }
/// // error: assertion failed at compile time
/// ```
#[proc_macro]
pub fn uor(input: TokenStream) -> TokenStream {
    let input_str = input.to_string();
    match parser::parse(&input_str) {
        Ok(parsed) => codegen::emit(&parsed),
        Err(err) => {
            let msg = format!("uor! parse error: {err}");
            quote::quote! { compile_error!(#msg); }.into()
        }
    }
}
