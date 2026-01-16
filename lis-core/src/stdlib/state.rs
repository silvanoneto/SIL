//! State Construction and Manipulation for LIS
//!
//! Operations on SilState (16-layer architecture) for multimodal state management.

use crate::error::{Error, Result};
use sil_core::state::{ByteSil, SilState};

// =============================================================================
// Constructors
// =============================================================================

/// @stdlib_state fn state_vacuum() -> State
///
/// Creates a vacuum state where all 16 layers are NULL (0, 0).
/// Represents complete absence or initial state.
///
/// # Example
/// ```lis
/// let s = state_vacuum();
/// ```
pub fn state_vacuum() -> Result<SilState> {
    Ok(SilState::vacuum())
}

/// @stdlib_state fn state_neutral() -> State
///
/// Creates a neutral state where all 16 layers are ONE (255, 0).
/// Represents identity or neutral equilibrium.
///
/// # Example
/// ```lis
/// let s = state_neutral();
/// ```
pub fn state_neutral() -> Result<SilState> {
    Ok(SilState::neutral())
}

/// @stdlib_state fn state_maximum() -> State
///
/// Creates a maximum state where all 16 layers are MAX (255, 255).
/// Represents saturation or maximum capacity.
///
/// # Example
/// ```lis
/// let s = state_maximum();
/// ```
pub fn state_maximum() -> Result<SilState> {
    Ok(SilState::maximum())
}

/// @stdlib_state fn state_from_bytes(bytes: [Int; 16]) -> State
///
/// Creates a state from 16 raw byte values (one per layer).
///
/// # Example
/// ```lis
/// let bytes = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];
/// let s = state_from_bytes(bytes);
/// ```
pub fn state_from_bytes(bytes: &[u8; 16]) -> Result<SilState> {
    Ok(SilState::from_bytes(bytes))
}

/// @stdlib_state fn state_to_bytes(s: State) -> [Int; 16]
///
/// Converts a state to 16 raw byte values (one per layer).
///
/// # Example
/// ```lis
/// let bytes = state_to_bytes(s);
/// ```
pub fn state_to_bytes(s: &SilState) -> Result<[u8; 16]> {
    Ok(s.to_bytes())
}

/// @stdlib_state fn state_from_layers(layers: [ByteSil; 16]) -> State
///
/// Creates a state from an array of 16 ByteSil values.
///
/// # Example
/// ```lis
/// let layers = [bytesil_one(); 16];
/// let s = state_from_layers(layers);
/// ```
pub fn state_from_layers(layers: &[ByteSil; 16]) -> Result<SilState> {
    Ok(SilState::from_layers(*layers))
}

// =============================================================================
// Layer Access
// =============================================================================

/// @stdlib_state fn state_get_layer(s: State, idx: Int) -> ByteSil
///
/// Gets the ByteSil value at a specific layer index [0-15].
///
/// # Example
/// ```lis
/// let layer0 = state_get_layer(s, 0);  // Photonic
/// ```
pub fn state_get_layer(s: &SilState, idx: u8) -> Result<ByteSil> {
    if idx >= 16 {
        return Err(Error::SemanticError {
            message: format!("Layer index {} out of bounds [0-15]", idx),
        });
    }
    Ok(s.layer(idx as usize))
}

/// @stdlib_state fn state_set_layer(s: State, idx: Int, value: ByteSil) -> State
///
/// Creates a new state with the specified layer set to a value.
///
/// # Example
/// ```lis
/// let s2 = state_set_layer(s, 0, bytesil_one());
/// ```
pub fn state_set_layer(s: &SilState, idx: u8, value: ByteSil) -> Result<SilState> {
    if idx >= 16 {
        return Err(Error::SemanticError {
            message: format!("Layer index {} out of bounds [0-15]", idx),
        });
    }
    Ok(s.with_layer(idx as usize, value))
}

/// @stdlib_state fn state_get_perception(s: State) -> [ByteSil; 5]
///
/// Gets the perception layer group (L0-L4): Photonic, Acoustic, Olfactory, Gustatory, Dermic.
///
/// # Example
/// ```lis
/// let perception = state_get_perception(s);
/// ```
pub fn state_get_perception(s: &SilState) -> Result<[ByteSil; 5]> {
    Ok(s.perception())
}

/// @stdlib_state fn state_get_processing(s: State) -> [ByteSil; 3]
///
/// Gets the processing layer group (L5-L7): Electronic, Psychomotor, Environmental.
///
/// # Example
/// ```lis
/// let processing = state_get_processing(s);
/// ```
pub fn state_get_processing(s: &SilState) -> Result<[ByteSil; 3]> {
    Ok(s.processing())
}

/// @stdlib_state fn state_get_interaction(s: State) -> [ByteSil; 3]
///
/// Gets the interaction layer group (L8-LA): Cybernetic, Geopolitical, Cosmopolitical.
///
/// # Example
/// ```lis
/// let interaction = state_get_interaction(s);
/// ```
pub fn state_get_interaction(s: &SilState) -> Result<[ByteSil; 3]> {
    Ok(s.interaction())
}

/// @stdlib_state fn state_get_emergence(s: State) -> [ByteSil; 2]
///
/// Gets the emergence layer group (LB-LC): Synergic, Quantum.
///
/// # Example
/// ```lis
/// let emergence = state_get_emergence(s);
/// ```
pub fn state_get_emergence(s: &SilState) -> Result<[ByteSil; 2]> {
    Ok(s.emergence())
}

/// @stdlib_state fn state_get_meta(s: State) -> [ByteSil; 3]
///
/// Gets the meta layer group (LD-LF): Superposition, Entanglement, Collapse.
///
/// # Example
/// ```lis
/// let meta = state_get_meta(s);
/// ```
pub fn state_get_meta(s: &SilState) -> Result<[ByteSil; 3]> {
    Ok(s.meta())
}

// =============================================================================
// State Operations
// =============================================================================

/// @stdlib_state fn state_tensor(a: State, b: State) -> State
///
/// Tensor product of two states (layer-wise multiplication).
///
/// # Example
/// ```lis
/// let product = state_tensor(a, b);
/// ```
pub fn state_tensor(a: &SilState, b: &SilState) -> Result<SilState> {
    Ok(a.tensor(b))
}

/// @stdlib_state fn state_xor(a: State, b: State) -> State
///
/// XOR of two states (layer-wise XOR operation).
///
/// # Example
/// ```lis
/// let xored = state_xor(a, b);
/// ```
pub fn state_xor(a: &SilState, b: &SilState) -> Result<SilState> {
    Ok(a.xor(b))
}

/// @stdlib_state fn state_project(s: State, mask: Int) -> State
///
/// Projects a state using a 16-bit mask (selects specific layers).
///
/// # Example
/// ```lis
/// let perception_only = state_project(s, 0x001F);  // Mask L0-L4
/// ```
pub fn state_project(s: &SilState, mask: u16) -> Result<SilState> {
    Ok(s.project(mask))
}

/// @stdlib_state fn state_collapse_xor(s: State) -> ByteSil
///
/// Collapses a state to a single ByteSil by XORing all layers.
///
/// # Example
/// ```lis
/// let collapsed = state_collapse_xor(s);
/// ```
pub fn state_collapse_xor(s: &SilState) -> Result<ByteSil> {
    use sil_core::state::CollapseStrategy;
    Ok(s.collapse(CollapseStrategy::Xor))
}

/// @stdlib_state fn state_collapse_sum(s: State) -> ByteSil
///
/// Collapses a state to a single ByteSil by summing all layer magnitudes.
///
/// # Example
/// ```lis
/// let collapsed = state_collapse_sum(s);
/// ```
pub fn state_collapse_sum(s: &SilState) -> Result<ByteSil> {
    use sil_core::state::CollapseStrategy;
    Ok(s.collapse(CollapseStrategy::Sum))
}

/// @stdlib_state fn state_collapse_first(s: State) -> ByteSil
///
/// Collapses a state by returning the first non-null layer.
///
/// # Example
/// ```lis
/// let collapsed = state_collapse_first(s);
/// ```
pub fn state_collapse_first(s: &SilState) -> Result<ByteSil> {
    use sil_core::state::CollapseStrategy;
    Ok(s.collapse(CollapseStrategy::First))
}

/// @stdlib_state fn state_collapse_last(s: State) -> ByteSil
///
/// Collapses a state by returning the last non-null layer.
///
/// # Example
/// ```lis
/// let collapsed = state_collapse_last(s);
/// ```
pub fn state_collapse_last(s: &SilState) -> Result<ByteSil> {
    use sil_core::state::CollapseStrategy;
    Ok(s.collapse(CollapseStrategy::Last))
}

/// @stdlib_state fn state_hash(s: State) -> Int
///
/// Computes a 64-bit hash of the state.
///
/// # Example
/// ```lis
/// let h = state_hash(s);
/// ```
pub fn state_hash(s: &SilState) -> Result<u128> {
    Ok(s.hash())
}

// =============================================================================
// Layer Group Masks
// =============================================================================

/// @stdlib_state fn perception_mask() -> Int
///
/// Returns the mask for perception layers (L0-L4): 0x001F
pub fn perception_mask() -> Result<u16> {
    Ok(0x001F)
}

/// @stdlib_state fn processing_mask() -> Int
///
/// Returns the mask for processing layers (L5-L7): 0x00E0
pub fn processing_mask() -> Result<u16> {
    Ok(0x00E0)
}

/// @stdlib_state fn interaction_mask() -> Int
///
/// Returns the mask for interaction layers (L8-LA): 0x0700
pub fn interaction_mask() -> Result<u16> {
    Ok(0x0700)
}

/// @stdlib_state fn emergence_mask() -> Int
///
/// Returns the mask for emergence layers (LB-LC): 0x1800
pub fn emergence_mask() -> Result<u16> {
    Ok(0x1800)
}

/// @stdlib_state fn meta_mask() -> Int
///
/// Returns the mask for meta layers (LD-LF): 0xE000
pub fn meta_mask() -> Result<u16> {
    Ok(0xE000)
}

// =============================================================================
// Utility Functions
// =============================================================================

/// @stdlib_state fn state_equals(a: State, b: State) -> Bool
///
/// Checks if two states are equal (all layers match).
///
/// # Example
/// ```lis
/// if state_equals(a, b) {
///     // States are identical
/// }
/// ```
pub fn state_equals(a: &SilState, b: &SilState) -> Result<bool> {
    Ok(a == b)
}

/// @stdlib_state fn state_is_vacuum(s: State) -> Bool
///
/// Checks if the state is a vacuum state (all layers NULL).
///
/// # Example
/// ```lis
/// if state_is_vacuum(s) {
///     // State is empty
/// }
/// ```
pub fn state_is_vacuum(s: &SilState) -> Result<bool> {
    Ok(*s == SilState::vacuum())
}

/// @stdlib_state fn state_is_neutral(s: State) -> Bool
///
/// Checks if the state is a neutral state (all layers ONE).
///
/// # Example
/// ```lis
/// if state_is_neutral(s) {
///     // State is at identity
/// }
/// ```
pub fn state_is_neutral(s: &SilState) -> Result<bool> {
    Ok(*s == SilState::neutral())
}

/// @stdlib_state fn state_count_null_layers(s: State) -> Int
///
/// Counts how many layers are NULL in the state.
///
/// # Example
/// ```lis
/// let null_count = state_count_null_layers(s);
/// ```
pub fn state_count_null_layers(s: &SilState) -> Result<u8> {
    let mut count = 0;
    for i in 0..16 {
        if s.layer(i) == ByteSil::NULL {
            count += 1;
        }
    }
    Ok(count)
}

/// @stdlib_state fn state_count_active_layers(s: State) -> Int
///
/// Counts how many layers are non-NULL in the state.
///
/// # Example
/// ```lis
/// let active_count = state_count_active_layers(s);
/// ```
pub fn state_count_active_layers(s: &SilState) -> Result<u8> {
    let mut count = 0;
    for i in 0..16 {
        if s.layer(i) != ByteSil::NULL {
            count += 1;
        }
    }
    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constructors() {
        let vacuum = state_vacuum().unwrap();
        assert!(state_is_vacuum(&vacuum).unwrap());

        let neutral = state_neutral().unwrap();
        assert!(state_is_neutral(&neutral).unwrap());

        let maximum = state_maximum().unwrap();
        for i in 0..16 {
            assert_eq!(maximum.layer(i), ByteSil::MAX);
        }
    }

    #[test]
    fn test_layer_access() {
        let mut s = state_vacuum().unwrap();
        s = state_set_layer(&s, 0, ByteSil::ONE).unwrap();

        let layer0 = state_get_layer(&s, 0).unwrap();
        assert_eq!(layer0, ByteSil::ONE);
    }

    #[test]
    fn test_layer_groups() {
        let s = state_neutral().unwrap();

        let perception = state_get_perception(&s).unwrap();
        assert_eq!(perception.len(), 5);

        let processing = state_get_processing(&s).unwrap();
        assert_eq!(processing.len(), 3);

        let interaction = state_get_interaction(&s).unwrap();
        assert_eq!(interaction.len(), 3);

        let emergence = state_get_emergence(&s).unwrap();
        assert_eq!(emergence.len(), 2);

        let meta = state_get_meta(&s).unwrap();
        assert_eq!(meta.len(), 3);
    }

    #[test]
    fn test_tensor_product() {
        let a = state_neutral().unwrap();
        let b = state_neutral().unwrap();
        let result = state_tensor(&a, &b).unwrap();

        // Identity tensor identity should be identity
        assert!(state_is_neutral(&result).unwrap());
    }

    #[test]
    fn test_xor_operation() {
        let a = state_neutral().unwrap();
        let b = state_neutral().unwrap();
        let xored = state_xor(&a, &b).unwrap();

        // XOR of identical states: rho ^ rho = 0, theta ^ theta = 0
        // This gives ByteSil { rho: 0, theta: 0 } = ONE, not NULL (rho=-8)
        // So XOR of neutral with itself gives all-ones state, not vacuum
        for i in 0..16 {
            let layer = xored.get(i);
            assert_eq!(layer.rho, 0, "Layer {} rho should be 0 (XOR of same)", i);
            assert_eq!(layer.theta, 0, "Layer {} theta should be 0 (XOR of same)", i);
        }
    }

    #[test]
    fn test_projection() {
        let s = state_neutral().unwrap();
        let mask = perception_mask().unwrap();
        let projected = state_project(&s, mask).unwrap();

        // First 5 layers should be neutral, rest should be vacuum
        for i in 0..5 {
            assert_eq!(projected.layer(i), ByteSil::ONE);
        }
        for i in 5..16 {
            assert_eq!(projected.layer(i), ByteSil::NULL);
        }
    }

    #[test]
    fn test_collapse_strategies() {
        let s = state_neutral().unwrap();

        let _xor_collapse = state_collapse_xor(&s).unwrap();
        let _sum_collapse = state_collapse_sum(&s).unwrap();
        let first_collapse = state_collapse_first(&s).unwrap();
        let last_collapse = state_collapse_last(&s).unwrap();

        // First and last of neutral state should be ONE
        assert_eq!(first_collapse, ByteSil::ONE);
        assert_eq!(last_collapse, ByteSil::ONE);
    }

    #[test]
    fn test_bytes_conversion() {
        let bytes: [u8; 16] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];
        let s = state_from_bytes(&bytes).unwrap();
        let bytes2 = state_to_bytes(&s).unwrap();

        assert_eq!(bytes, bytes2);
    }

    #[test]
    fn test_count_layers() {
        let vacuum = state_vacuum().unwrap();
        assert_eq!(state_count_null_layers(&vacuum).unwrap(), 16);
        assert_eq!(state_count_active_layers(&vacuum).unwrap(), 0);

        let neutral = state_neutral().unwrap();
        assert_eq!(state_count_null_layers(&neutral).unwrap(), 0);
        assert_eq!(state_count_active_layers(&neutral).unwrap(), 16);
    }

    #[test]
    fn test_layer_bounds_checking() {
        let s = state_vacuum().unwrap();
        let result = state_get_layer(&s, 16);
        assert!(result.is_err());

        let result = state_set_layer(&s, 16, ByteSil::ONE);
        assert!(result.is_err());
    }

    #[test]
    fn test_hash() {
        let a = state_neutral().unwrap();
        let b = state_neutral().unwrap();
        let c = state_vacuum().unwrap();

        let hash_a = state_hash(&a).unwrap();
        let hash_b = state_hash(&b).unwrap();
        let hash_c = state_hash(&c).unwrap();

        // Same states should have same hash
        assert_eq!(hash_a, hash_b);

        // Different states should (probably) have different hashes
        assert_ne!(hash_a, hash_c);
    }

    #[test]
    fn test_masks() {
        assert_eq!(perception_mask().unwrap(), 0x001F);
        assert_eq!(processing_mask().unwrap(), 0x00E0);
        assert_eq!(interaction_mask().unwrap(), 0x0700);
        assert_eq!(emergence_mask().unwrap(), 0x1800);
        assert_eq!(meta_mask().unwrap(), 0xE000);
    }
}
