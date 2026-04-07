//! Integration tests for the `uor!` proc macro.
//!
//! Each test verifies that the macro compiles and produces a valid value.

use uor_foundation::uor;

#[test]
fn term_expression_add() {
    let _arena = uor! { add(1, 2) };
}

#[test]
fn term_expression_nested() {
    let _arena = uor! { add(mul(3, 5), 7) };
}

#[test]
fn quantum_literal() {
    let _arena = uor! { 42@Q0 };
}

#[test]
fn type_decl_named_arg_constraint() {
    let _decl = uor! { type Pixel { ResidueConstraint(modulus: 256, residue: 255); } };
}

#[test]
fn type_decl_multiple_constraints() {
    let _decl = uor! {
        type RGB {
            ResidueConstraint(modulus: 256, residue: 0);
            HammingConstraint(hammingBound: 8);
        }
    };
}

#[test]
fn type_decl_with_params() {
    let _decl = uor! { type Pair(T, U) { } };
}

#[test]
fn type_decl_empty() {
    let _decl = uor! { type Unit { } };
}

#[test]
fn ground_assertion_passes() {
    uor! { assert add(1, 2) = 3; };
}

#[test]
fn ground_assertion_mul() {
    uor! { assert mul(3, 5) = 15; };
}

#[test]
fn binding() {
    let _binding = uor! { let origin : Pixel = add(0, 0); };
}

#[test]
fn effect_decl_generic_props() {
    let _effect = uor! {
        effect Blit {
            target: 0;
            delta: 0;
            commutes: true;
        }
    };
}

#[test]
fn source_decl() {
    let _src = uor! { source input : ScalarType via IntegerGroundingMap; };
}

#[test]
fn sink_decl() {
    let _sink = uor! { sink output : ScalarType via IntegerProjectionMap; };
}

#[test]
fn match_without_otherwise() {
    let _m = uor! {
        match x {
            isZero => 0;
        }
    };
}

#[test]
fn match_with_otherwise() {
    let _m = uor! {
        match x {
            isZero => 0;
            otherwise => x;
        }
    };
}

#[test]
fn unfold_stream() {
    let _stream = uor! { unfold nat : Successor from 0 };
}

#[test]
fn any_ident_application() {
    let _arena = uor! { custom_op(1, 2, 3) };
}

#[test]
fn lift_project() {
    let _lift = uor! { lift(0, Q3) };
    let _proj = uor! { project(0, Q0) };
}
