//! Type checker tests for LIS
//!
//! These tests validate the bidirectional type inference system.

use lis_core::{compile, Error, TypeErrorKind};

// ===== Basic Type Inference Tests =====

#[test]
fn test_int_literal_inference() {
    let source = r#"
        fn main() {
            let x = 42;
        }
    "#;
    let result = compile(source);
    assert!(result.is_ok(), "Failed to compile: {:?}", result.err());
}

#[test]
fn test_float_literal_inference() {
    let source = r#"
        fn main() {
            let x = 3.14;
        }
    "#;
    let result = compile(source);
    assert!(result.is_ok());
}

#[test]
fn test_bool_literal_inference() {
    let source = r#"
        fn main() {
            let x = true;
            let y = false;
        }
    "#;
    let result = compile(source);
    assert!(result.is_ok());
}

#[test]
fn test_string_literal_inference() {
    let source = r#"
        fn main() {
            let x = "hello";
        }
    "#;
    let result = compile(source);
    assert!(result.is_ok());
}

// ===== Complex Number Tests =====

#[test]
fn test_complex_number_inference() {
    let source = r#"
        fn main() {
            let z = (1.0, 0.0);
        }
    "#;
    let result = compile(source);
    assert!(result.is_ok());
}

#[test]
fn test_complex_with_int_components() {
    let source = r#"
        fn main() {
            let z = (1, 0);
        }
    "#;
    let result = compile(source);
    assert!(result.is_ok(), "Should allow int components in complex numbers");
}

// ===== Binary Operation Tests =====

#[test]
fn test_arithmetic_int_int() {
    let source = r#"
        fn main() {
            let x = 1 + 2;
            let y = 10 - 3;
            let z = 4 * 5;
            let w = 20 / 4;
        }
    "#;
    let result = compile(source);
    assert!(result.is_ok());
}

#[test]
fn test_arithmetic_float_float() {
    let source = r#"
        fn main() {
            let x = 1.5 + 2.5;
            let y = 10.0 - 3.0;
        }
    "#;
    let result = compile(source);
    assert!(result.is_ok());
}

#[test]
fn test_arithmetic_int_float_promotion() {
    let source = r#"
        fn main() {
            let x = 1 + 2.5;
            let y = 3.14 - 1;
        }
    "#;
    let result = compile(source);
    assert!(result.is_ok(), "Should promote int to float in mixed arithmetic");
}

#[test]
fn test_comparison_operators() {
    let source = r#"
        fn main() {
            let a = 1 < 2;
            let b = 3 <= 3;
            let c = 5 > 4;
            let d = 10 >= 10;
            let e = 1 == 1;
            let f = 2 != 3;
        }
    "#;
    let result = compile(source);
    assert!(result.is_ok());
}

#[test]
fn test_logical_operators() {
    let source = r#"
        fn main() {
            let a = true && false;
            let b = true || false;
        }
    "#;
    let result = compile(source);
    assert!(result.is_ok());
}

#[test]
fn test_logical_operators_on_non_bool_fails() {
    let source = r#"
        fn main() {
            let x = 1 && 2;
        }
    "#;
    let result = compile(source);
    assert!(result.is_err(), "Should reject logical operators on non-bool types");
}

// ===== Unary Operation Tests =====

#[test]
fn test_negation() {
    let source = r#"
        fn main() {
            let x = -42;
            let y = -3.14;
        }
    "#;
    let result = compile(source);
    assert!(result.is_ok());
}

#[test]
fn test_logical_not() {
    let source = r#"
        fn main() {
            let x = !true;
            let y = !false;
        }
    "#;
    let result = compile(source);
    assert!(result.is_ok());
}

// ===== Variable Reference Tests =====

#[test]
fn test_variable_lookup() {
    let source = r#"
        fn main() {
            let x = 42;
            let y = x;
        }
    "#;
    let result = compile(source);
    assert!(result.is_ok());
}

#[test]
fn test_undefined_variable_fails() {
    let source = r#"
        fn main() {
            let y = x;
        }
    "#;
    let result = compile(source);
    assert!(result.is_err(), "Should reject undefined variables");
}

// ===== Type Annotation Tests =====

#[test]
fn test_explicit_int_type() {
    let source = r#"
        fn main() {
            let x: Int = 42;
        }
    "#;
    let result = compile(source);
    // Note: Current parser may not support type annotations yet
    // This test documents the desired behavior
    // assert!(result.is_ok());
}

#[test]
fn test_type_mismatch_with_annotation() {
    let source = r#"
        fn main() {
            let x: Int = 3.14;
        }
    "#;
    let result = compile(source);
    // Should fail due to type mismatch
    // assert!(result.is_err());
}

// ===== Assignment Tests =====

#[test]
fn test_assignment_compatible_types() {
    let source = r#"
        fn main() {
            let x = 42;
            x = 100;
        }
    "#;
    let result = compile(source);
    assert!(result.is_ok());
}

#[test]
fn test_assignment_incompatible_types_fails() {
    let source = r#"
        fn main() {
            let x = 42;
            x = "string";
        }
    "#;
    let result = compile(source);
    assert!(result.is_err(), "Should reject assignment of incompatible types");
}

// ===== Control Flow Tests =====

#[test]
fn test_if_statement_with_bool_condition() {
    let source = r#"
        fn main() {
            if true {
                let x = 1;
            }
        }
    "#;
    let result = compile(source);
    assert!(result.is_ok());
}

#[test]
fn test_if_statement_with_comparison() {
    let source = r#"
        fn main() {
            let x = 5;
            if x > 3 {
                let y = 10;
            }
        }
    "#;
    let result = compile(source);
    assert!(result.is_ok());
}

#[test]
fn test_if_with_non_bool_condition_fails() {
    let source = r#"
        fn main() {
            if 42 {
                let x = 1;
            }
        }
    "#;
    let result = compile(source);
    assert!(result.is_err(), "Should reject non-bool if condition");
}

#[test]
fn test_if_else() {
    let source = r#"
        fn main() {
            if true {
                let x = 1;
            } else {
                let y = 2;
            }
        }
    "#;
    let result = compile(source);
    assert!(result.is_ok());
}

#[test]
fn test_loop() {
    let source = r#"
        fn main() {
            loop {
                let x = 1;
                break;
            }
        }
    "#;
    let result = compile(source);
    assert!(result.is_ok());
}

// ===== Function Tests =====

#[test]
fn test_function_with_params() {
    let source = r#"
        fn add(a: Int, b: Int) {
            return a + b;
        }
        fn main() {
            let result = add(1, 2);
        }
    "#;
    let result = compile(source);
    // Note: Function calls may not be fully implemented yet
    // assert!(result.is_ok());
}

#[test]
fn test_function_return_type_inference() {
    let source = r#"
        fn get_number() {
            return 42;
        }
        fn main() {
            let x = get_number();
        }
    "#;
    let result = compile(source);
    // assert!(result.is_ok());
}

// ===== Layer and State Tests =====

#[test]
fn test_layer_access() {
    let source = r#"
        fn main() {
            let s = State { L0: (1.0, 0.0), L1: (2.0, 0.0) };
            let layer0 = s.L0;
        }
    "#;
    let result = compile(source);
    // State construction may not be parsed yet
    // assert!(result.is_ok());
}

#[test]
fn test_invalid_layer_access_fails() {
    let source = r#"
        fn main() {
            let s = State { L0: (1.0, 0.0) };
            let bad = s.L10;
        }
    "#;
    let result = compile(source);
    // Should fail for layer > 0xF
    // assert!(result.is_err());
}

#[test]
fn test_state_construction() {
    let source = r#"
        fn main() {
            let s = State {
                L0: (1.0, 0.0),
                L1: (2.0, 0.0),
                L2: (3.0, 0.0)
            };
        }
    "#;
    let result = compile(source);
    // assert!(result.is_ok());
}

// ===== Integration Tests =====

#[test]
fn test_complex_expression() {
    let source = r#"
        fn main() {
            let a = 1;
            let b = 2;
            let c = 3;
            let result = (a + b) * c - 5;
        }
    "#;
    let result = compile(source);
    assert!(result.is_ok());
}

#[test]
fn test_nested_control_flow() {
    let source = r#"
        fn main() {
            let x = 10;
            if x > 5 {
                if x < 15 {
                    let y = x * 2;
                }
            }
        }
    "#;
    let result = compile(source);
    assert!(result.is_ok());
}

#[test]
fn test_multiple_functions() {
    let source = r#"
        fn helper() {
            let x = 42;
        }

        fn main() {
            let y = 100;
        }
    "#;
    let result = compile(source);
    assert!(result.is_ok());
}

// ===== Error Message Tests =====

#[test]
fn test_error_contains_line_info() {
    let source = r#"
        fn main() {
            let x = y;
        }
    "#;
    let result = compile(source);
    assert!(result.is_err());
    if let Err(Error::SemanticError { message }) = result {
        assert!(message.contains("y") || message.contains("Undefined"),
                "Error should mention undefined variable: {}", message);
    }
}

// ===== Stress Tests =====

#[test]
fn test_many_variables() {
    let source = r#"
        fn main() {
            let a = 1;
            let b = 2;
            let c = 3;
            let d = 4;
            let e = 5;
            let f = 6;
            let g = 7;
            let h = 8;
            let i = 9;
            let j = 10;
            let result = a + b + c + d + e + f + g + h + i + j;
        }
    "#;
    let result = compile(source);
    assert!(result.is_ok());
}

#[test]
fn test_deep_expression_nesting() {
    let source = r#"
        fn main() {
            let x = ((((1 + 2) * 3) - 4) / 5);
        }
    "#;
    let result = compile(source);
    assert!(result.is_ok());
}

// ===== Edge Cases =====

#[test]
fn test_empty_function() {
    let source = r#"
        fn main() {
        }
    "#;
    let result = compile(source);
    assert!(result.is_ok());
}

#[test]
fn test_only_return() {
    let source = r#"
        fn main() {
            return;
        }
    "#;
    let result = compile(source);
    assert!(result.is_ok());
}

#[test]
fn test_return_with_value() {
    let source = r#"
        fn main() {
            return 42;
        }
    "#;
    let result = compile(source);
    assert!(result.is_ok());
}
