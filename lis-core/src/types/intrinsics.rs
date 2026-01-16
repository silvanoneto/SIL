//! Standard library intrinsic function type signatures
//!
//! This module provides type signatures for all stdlib intrinsic functions
//! so they can be registered in the type checker.

use super::inference::TypeContext;
use super::{Span, Type};

/// Registers all stdlib intrinsic function signatures in the type context
pub fn register_stdlib_intrinsics(context: &mut TypeContext) {
    let span = Span::dummy();

    // Helper to create function types
    let func = |params: Vec<Type>, ret: Type| -> Type {
        Type::function(params, ret, span)
    };

    // Type shortcuts
    let int = Type::int(span);
    let float = Type::float(span);
    let bool_ty = Type::bool(span);
    let string = Type::string(span);
    let bytesil = Type::bytesil(span);
    let state = Type::state(span);
    let unit = Type::unit(span);
    let unknown = Type::unknown(0, span);

    // =========================================================================
    // ByteSil operations (28 functions)
    // =========================================================================

    context.bind("bytesil_new".to_string(), func(vec![int.clone(), int.clone()], bytesil.clone()));
    context.bind("bytesil_from_complex".to_string(), func(vec![float.clone(), float.clone()], bytesil.clone()));
    // Note: returns (Float, Float) but we use unknown for now as tuples aren't fully implemented
    context.bind("bytesil_to_complex".to_string(), func(vec![bytesil.clone()], unknown.clone()));
    context.bind("bytesil_null".to_string(), func(vec![], bytesil.clone()));
    context.bind("bytesil_one".to_string(), func(vec![], bytesil.clone()));
    context.bind("bytesil_i".to_string(), func(vec![], bytesil.clone()));
    context.bind("bytesil_neg_one".to_string(), func(vec![], bytesil.clone()));
    context.bind("bytesil_neg_i".to_string(), func(vec![], bytesil.clone()));
    context.bind("bytesil_max".to_string(), func(vec![], bytesil.clone()));
    context.bind("bytesil_mul".to_string(), func(vec![bytesil.clone(), bytesil.clone()], bytesil.clone()));
    context.bind("bytesil_div".to_string(), func(vec![bytesil.clone(), bytesil.clone()], bytesil.clone()));
    context.bind("bytesil_pow".to_string(), func(vec![bytesil.clone(), int.clone()], bytesil.clone()));
    context.bind("bytesil_root".to_string(), func(vec![bytesil.clone(), int.clone()], bytesil.clone()));
    context.bind("bytesil_inv".to_string(), func(vec![bytesil.clone()], bytesil.clone()));
    context.bind("bytesil_conj".to_string(), func(vec![bytesil.clone()], bytesil.clone()));
    context.bind("bytesil_xor".to_string(), func(vec![bytesil.clone(), bytesil.clone()], bytesil.clone()));
    context.bind("bytesil_mix".to_string(), func(vec![bytesil.clone(), bytesil.clone()], bytesil.clone()));
    context.bind("bytesil_rho".to_string(), func(vec![bytesil.clone()], int.clone()));
    context.bind("bytesil_theta".to_string(), func(vec![bytesil.clone()], int.clone()));
    context.bind("bytesil_magnitude".to_string(), func(vec![bytesil.clone()], float.clone()));
    context.bind("bytesil_phase_degrees".to_string(), func(vec![bytesil.clone()], float.clone()));
    context.bind("bytesil_phase_radians".to_string(), func(vec![bytesil.clone()], float.clone()));
    context.bind("bytesil_is_null".to_string(), func(vec![bytesil.clone()], bool_ty.clone()));
    context.bind("bytesil_is_real".to_string(), func(vec![bytesil.clone()], bool_ty.clone()));
    context.bind("bytesil_is_imaginary".to_string(), func(vec![bytesil.clone()], bool_ty.clone()));
    context.bind("bytesil_norm".to_string(), func(vec![bytesil.clone()], int.clone()));
    context.bind("bytesil_from_u8".to_string(), func(vec![int.clone()], bytesil.clone()));
    context.bind("bytesil_to_u8".to_string(), func(vec![bytesil.clone()], int.clone()));

    // Aliases documented in LIS_SIL_DOCUMENTATION.md
    context.bind("bytesil_sqrt".to_string(), func(vec![bytesil.clone()], bytesil.clone()));
    context.bind("bytesil_conjugate".to_string(), func(vec![bytesil.clone()], bytesil.clone()));
    context.bind("from_mag_phase".to_string(), func(vec![float.clone(), float.clone()], bytesil.clone()));
    context.bind("from_cartesian".to_string(), func(vec![float.clone(), float.clone()], bytesil.clone()));

    // =========================================================================
    // Math operations (36 functions)
    // =========================================================================

    // Complex math
    context.bind("complex_add".to_string(), func(vec![bytesil.clone(), bytesil.clone()], bytesil.clone()));
    context.bind("complex_sub".to_string(), func(vec![bytesil.clone(), bytesil.clone()], bytesil.clone()));
    context.bind("complex_scale".to_string(), func(vec![bytesil.clone(), float.clone()], bytesil.clone()));
    context.bind("complex_rotate".to_string(), func(vec![bytesil.clone(), float.clone()], bytesil.clone()));
    context.bind("complex_lerp".to_string(), func(vec![bytesil.clone(), bytesil.clone(), float.clone()], bytesil.clone()));

    // Trigonometric
    context.bind("sin".to_string(), func(vec![float.clone()], float.clone()));
    context.bind("cos".to_string(), func(vec![float.clone()], float.clone()));
    context.bind("tan".to_string(), func(vec![float.clone()], float.clone()));
    context.bind("asin".to_string(), func(vec![float.clone()], float.clone()));
    context.bind("acos".to_string(), func(vec![float.clone()], float.clone()));
    context.bind("atan".to_string(), func(vec![float.clone()], float.clone()));
    context.bind("atan2".to_string(), func(vec![float.clone(), float.clone()], float.clone()));

    // Constants
    context.bind("pi".to_string(), func(vec![], float.clone()));
    context.bind("tau".to_string(), func(vec![], float.clone()));
    context.bind("e".to_string(), func(vec![], float.clone()));
    context.bind("phi".to_string(), func(vec![], float.clone()));

    // Utility functions
    context.bind("abs_int".to_string(), func(vec![int.clone()], int.clone()));
    context.bind("abs_float".to_string(), func(vec![float.clone()], float.clone()));
    context.bind("min_int".to_string(), func(vec![int.clone(), int.clone()], int.clone()));
    context.bind("max_int".to_string(), func(vec![int.clone(), int.clone()], int.clone()));
    context.bind("min_float".to_string(), func(vec![float.clone(), float.clone()], float.clone()));
    context.bind("max_float".to_string(), func(vec![float.clone(), float.clone()], float.clone()));
    context.bind("clamp_int".to_string(), func(vec![int.clone(), int.clone(), int.clone()], int.clone()));
    context.bind("clamp_float".to_string(), func(vec![float.clone(), float.clone(), float.clone()], float.clone()));
    context.bind("sqrt".to_string(), func(vec![float.clone()], float.clone()));
    context.bind("pow_float".to_string(), func(vec![float.clone(), float.clone()], float.clone()));
    context.bind("exp".to_string(), func(vec![float.clone()], float.clone()));
    context.bind("ln".to_string(), func(vec![float.clone()], float.clone()));
    context.bind("log10".to_string(), func(vec![float.clone()], float.clone()));
    context.bind("log2".to_string(), func(vec![float.clone()], float.clone()));
    context.bind("floor".to_string(), func(vec![float.clone()], float.clone()));
    context.bind("ceil".to_string(), func(vec![float.clone()], float.clone()));
    context.bind("round".to_string(), func(vec![float.clone()], float.clone()));
    context.bind("sign_float".to_string(), func(vec![float.clone()], float.clone()));
    context.bind("sign_int".to_string(), func(vec![int.clone()], int.clone()));
    context.bind("degrees_to_radians".to_string(), func(vec![float.clone()], float.clone()));
    context.bind("radians_to_degrees".to_string(), func(vec![float.clone()], float.clone()));

    // Generic math functions documented in LIS_SIL_DOCUMENTATION.md
    context.bind("abs".to_string(), func(vec![float.clone()], float.clone()));
    context.bind("min".to_string(), func(vec![float.clone(), float.clone()], float.clone()));
    context.bind("max".to_string(), func(vec![float.clone(), float.clone()], float.clone()));

    // =========================================================================
    // Console I/O (10 functions)
    // =========================================================================

    // Generic print function that accepts any type
    context.bind("print".to_string(), func(vec![unknown.clone()], unit.clone()));
    context.bind("print_int".to_string(), func(vec![int.clone()], unit.clone()));
    context.bind("print_float".to_string(), func(vec![float.clone()], unit.clone()));
    context.bind("print_string".to_string(), func(vec![string.clone()], unit.clone()));
    context.bind("print_bool".to_string(), func(vec![bool_ty.clone()], unit.clone()));
    context.bind("print_bytesil".to_string(), func(vec![bytesil.clone()], unit.clone()));
    context.bind("print_state".to_string(), func(vec![state.clone()], unit.clone()));
    context.bind("println".to_string(), func(vec![string.clone()], unit.clone()));
    context.bind("read_line".to_string(), func(vec![], string.clone()));
    context.bind("read_int".to_string(), func(vec![], int.clone()));
    context.bind("read_float".to_string(), func(vec![], float.clone()));

    // =========================================================================
    // State operations (30 functions)
    // =========================================================================

    // Constructors
    context.bind("state_vacuum".to_string(), func(vec![], state.clone()));
    context.bind("state_neutral".to_string(), func(vec![], state.clone()));
    context.bind("state_maximum".to_string(), func(vec![], state.clone()));
    context.bind("state_from_bytes".to_string(), func(vec![unknown.clone()], state.clone()));
    context.bind("state_to_bytes".to_string(), func(vec![state.clone()], unknown.clone()));
    context.bind("state_from_layers".to_string(), func(vec![unknown.clone()], state.clone()));

    // Layer access
    context.bind("state_get_layer".to_string(), func(vec![state.clone(), int.clone()], bytesil.clone()));
    context.bind("state_set_layer".to_string(), func(vec![state.clone(), int.clone(), bytesil.clone()], state.clone()));
    context.bind("state_get_perception".to_string(), func(vec![state.clone()], unknown.clone()));
    context.bind("state_get_processing".to_string(), func(vec![state.clone()], unknown.clone()));
    context.bind("state_get_interaction".to_string(), func(vec![state.clone()], unknown.clone()));
    context.bind("state_get_emergence".to_string(), func(vec![state.clone()], unknown.clone()));
    context.bind("state_get_meta".to_string(), func(vec![state.clone()], unknown.clone()));

    // Operations
    context.bind("state_tensor".to_string(), func(vec![state.clone(), state.clone()], state.clone()));
    context.bind("state_xor".to_string(), func(vec![state.clone(), state.clone()], state.clone()));
    context.bind("state_project".to_string(), func(vec![state.clone(), int.clone()], state.clone()));
    context.bind("state_collapse_xor".to_string(), func(vec![state.clone()], bytesil.clone()));
    context.bind("state_collapse_sum".to_string(), func(vec![state.clone()], bytesil.clone()));
    context.bind("state_collapse_first".to_string(), func(vec![state.clone()], bytesil.clone()));
    context.bind("state_collapse_last".to_string(), func(vec![state.clone()], bytesil.clone()));
    context.bind("state_hash".to_string(), func(vec![state.clone()], int.clone()));

    // Masks
    context.bind("perception_mask".to_string(), func(vec![], int.clone()));
    context.bind("processing_mask".to_string(), func(vec![], int.clone()));
    context.bind("interaction_mask".to_string(), func(vec![], int.clone()));
    context.bind("emergence_mask".to_string(), func(vec![], int.clone()));
    context.bind("meta_mask".to_string(), func(vec![], int.clone()));

    // Utility
    context.bind("state_equals".to_string(), func(vec![state.clone(), state.clone()], bool_ty.clone()));
    context.bind("state_is_vacuum".to_string(), func(vec![state.clone()], bool_ty.clone()));
    context.bind("state_is_neutral".to_string(), func(vec![state.clone()], bool_ty.clone()));
    context.bind("state_count_null_layers".to_string(), func(vec![state.clone()], int.clone()));
    context.bind("state_count_active_layers".to_string(), func(vec![state.clone()], int.clone()));

    // State functions documented in LIS_SIL_DOCUMENTATION.md
    context.bind("state_normalize".to_string(), func(vec![state.clone()], state.clone()));

    // =========================================================================
    // Layer operations (7 functions)
    // =========================================================================

    context.bind("fuse_vision_audio".to_string(), func(vec![bytesil.clone(), bytesil.clone()], bytesil.clone()));
    context.bind("fuse_multimodal".to_string(), func(vec![unknown.clone()], bytesil.clone()));
    context.bind("normalize_perception".to_string(), func(vec![state.clone()], state.clone()));
    context.bind("shift_layers_up".to_string(), func(vec![state.clone()], state.clone()));
    context.bind("shift_layers_down".to_string(), func(vec![state.clone()], state.clone()));
    context.bind("rotate_layers".to_string(), func(vec![state.clone(), int.clone()], state.clone()));
    context.bind("spread_to_group".to_string(), func(vec![state.clone(), int.clone(), bytesil.clone(), int.clone()], state.clone()));

    // =========================================================================
    // Transform operations (9 functions)
    // =========================================================================

    context.bind("transform_phase_shift".to_string(), func(vec![state.clone(), int.clone()], state.clone()));
    context.bind("transform_magnitude_scale".to_string(), func(vec![state.clone(), int.clone()], state.clone()));
    context.bind("transform_layer_swap".to_string(), func(vec![state.clone(), int.clone(), int.clone()], state.clone()));
    context.bind("transform_xor_layers".to_string(), func(vec![state.clone(), int.clone(), int.clone(), int.clone()], state.clone()));
    context.bind("transform_identity".to_string(), func(vec![state.clone()], state.clone()));
    context.bind("apply_feedback".to_string(), func(vec![state.clone(), float.clone()], state.clone()));
    context.bind("detect_emergence".to_string(), func(vec![state.clone(), float.clone()], bool_ty.clone()));
    context.bind("emergence_pattern".to_string(), func(vec![state.clone()], bytesil.clone()));
    context.bind("autopoietic_loop".to_string(), func(vec![state.clone(), int.clone()], state.clone()));

    // Transform functions documented in LIS_SIL_DOCUMENTATION.md
    context.bind("relu_state".to_string(), func(vec![state.clone()], state.clone()));

    // =========================================================================
    // Debug utilities (10 functions)
    // =========================================================================

    context.bind("assert".to_string(), func(vec![bool_ty.clone(), string.clone()], unit.clone()));
    context.bind("assert_eq_int".to_string(), func(vec![int.clone(), int.clone(), string.clone()], unit.clone()));
    context.bind("assert_eq_bytesil".to_string(), func(vec![bytesil.clone(), bytesil.clone(), string.clone()], unit.clone()));
    context.bind("assert_eq_state".to_string(), func(vec![state.clone(), state.clone(), string.clone()], unit.clone()));
    context.bind("debug_print".to_string(), func(vec![string.clone(), bytesil.clone()], unit.clone()));
    context.bind("trace_state".to_string(), func(vec![string.clone(), state.clone()], unit.clone()));
    context.bind("timestamp_millis".to_string(), func(vec![], int.clone()));
    context.bind("timestamp_micros".to_string(), func(vec![], int.clone()));
    context.bind("sleep_millis".to_string(), func(vec![int.clone()], unit.clone()));
    context.bind("memory_used".to_string(), func(vec![], int.clone()));

    // =========================================================================
    // String operations (19 functions)
    // =========================================================================

    context.bind("string_length".to_string(), func(vec![string.clone()], int.clone()));
    context.bind("string_concat".to_string(), func(vec![string.clone(), string.clone()], string.clone()));
    context.bind("string_slice".to_string(), func(vec![string.clone(), int.clone(), int.clone()], string.clone()));
    context.bind("string_to_upper".to_string(), func(vec![string.clone()], string.clone()));
    context.bind("string_to_lower".to_string(), func(vec![string.clone()], string.clone()));
    context.bind("string_contains".to_string(), func(vec![string.clone(), string.clone()], bool_ty.clone()));
    context.bind("string_starts_with".to_string(), func(vec![string.clone(), string.clone()], bool_ty.clone()));
    context.bind("string_ends_with".to_string(), func(vec![string.clone(), string.clone()], bool_ty.clone()));
    context.bind("string_equals".to_string(), func(vec![string.clone(), string.clone()], bool_ty.clone()));
    context.bind("int_to_string".to_string(), func(vec![int.clone()], string.clone()));
    context.bind("float_to_string".to_string(), func(vec![float.clone()], string.clone()));
    context.bind("bool_to_string".to_string(), func(vec![bool_ty.clone()], string.clone()));
    context.bind("bytesil_to_string".to_string(), func(vec![bytesil.clone()], string.clone()));
    context.bind("state_to_string".to_string(), func(vec![state.clone()], string.clone()));
    context.bind("string_to_int".to_string(), func(vec![string.clone()], int.clone()));
    context.bind("string_to_float".to_string(), func(vec![string.clone()], float.clone()));
    context.bind("string_trim".to_string(), func(vec![string.clone()], string.clone()));
    context.bind("string_replace".to_string(), func(vec![string.clone(), string.clone(), string.clone()], string.clone()));
    context.bind("string_index_of".to_string(), func(vec![string.clone(), string.clone()], int.clone()));
}
