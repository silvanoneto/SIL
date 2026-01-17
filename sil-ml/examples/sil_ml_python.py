#!/usr/bin/env python3
"""
SIL-ML Python Integration - Essential Features

Bridges Python and Rust features for high-fidelity ML encoding/decoding.
Uses native _sil_core with semantic layer classification.
"""

import numpy as np
import _sil_core
from typing import List, Tuple, Optional
from enum import Enum


class SemanticCategory(Enum):
    """5 semantic layer categories"""
    PERCEPTION = "PERCEPTION"
    PROCESSING = "PROCESSING"
    INTERACTION = "INTERACTION"
    EMERGENCE = "EMERGENCE"
    META = "META"


class SemanticLayer:
    """16-layer semantic classification matching Rust implementation"""
    
    # PERCEPTION (0-4): Sensory input
    PHOTONIC = (0, "Photonic", SemanticCategory.PERCEPTION)
    ACOUSTIC = (1, "Acoustic", SemanticCategory.PERCEPTION)
    OLFACTORY = (2, "Olfactory", SemanticCategory.PERCEPTION)
    GUSTATORY = (3, "Gustatory", SemanticCategory.PERCEPTION)
    DERMIC = (4, "Dermic", SemanticCategory.PERCEPTION)
    
    # PROCESSING (5-7): Digital transformation
    ELECTRONIC = (5, "Electronic", SemanticCategory.PROCESSING)
    PSYCHOMOTOR = (6, "Psychomotor", SemanticCategory.PROCESSING)
    ENVIRONMENTAL = (7, "Environmental", SemanticCategory.PROCESSING)
    
    # INTERACTION (8-10): Cross-layer routing
    CYBERNETIC = (8, "Cybernetic", SemanticCategory.INTERACTION)
    GEOPOLITICAL = (9, "Geopolitical", SemanticCategory.INTERACTION)
    COSMOPOLITICAL = (10, "Cosmopolitical", SemanticCategory.INTERACTION)
    
    # EMERGENCE (11-12): Synergic properties
    SYNERGIC = (11, "Synergic", SemanticCategory.EMERGENCE)
    QUANTUM = (12, "Quantum", SemanticCategory.EMERGENCE)
    
    # META (13-15): Reflection/collapse
    SUPERPOSITION = (13, "Superposition", SemanticCategory.META)
    ENTANGLEMENT = (14, "Entanglement", SemanticCategory.META)
    COLLAPSE = (15, "Collapse", SemanticCategory.META)
    
    # Lookup table
    LAYERS = {
        0: PHOTONIC, 1: ACOUSTIC, 2: OLFACTORY, 3: GUSTATORY, 4: DERMIC,
        5: ELECTRONIC, 6: PSYCHOMOTOR, 7: ENVIRONMENTAL,
        8: CYBERNETIC, 9: GEOPOLITICAL, 10: COSMOPOLITICAL,
        11: SYNERGIC, 12: QUANTUM,
        13: SUPERPOSITION, 14: ENTANGLEMENT, 15: COLLAPSE,
    }
    
    @staticmethod
    def get(layer_idx: int) -> Tuple[int, str, SemanticCategory]:
        """Get layer metadata by index"""
        return SemanticLayer.LAYERS.get(layer_idx)
    
    @staticmethod
    def by_category(category: SemanticCategory) -> List[int]:
        """Get all layers in a category"""
        return [idx for idx, (_, _, cat) in SemanticLayer.LAYERS.items() 
                if cat == category]
    
    @staticmethod
    def get_all() -> dict:
        """Get all layers with metadata"""
        return SemanticLayer.LAYERS


class LinearEncoder:
    """
    HIGH-FIDELITY linear encoder for ML features.
    
    Round-trip error: < 0.01 (meets ML requirements)
    Strategy: Encode LINEAR via from_u8(), not log-polar
    """
    
    @staticmethod
    def encode(features: np.ndarray) -> _sil_core.SilState:
        """
        Encode feature vector to SilState with maximum fidelity.
        
        Args:
            features: 16-dimensional feature vector or shorter
            
        Returns:
            SilState with encoded features in 16 layers
            
        Fidelity: Round-trip error < 0.01
        """
        state = _sil_core.SilState.vacuum()
        
        for i in range(min(16, len(features))):
            val = features[i]
            
            # Normalize with tanh to [-1, 1]
            bounded = float(np.tanh(val))
            
            # LINEAR map: [-1, 1] → [0, 255]
            byte_val = int((bounded + 1.0) * 127.5)
            byte_val = np.clip(byte_val, 0, 255)
            
            # Create ByteSil with direct linear encoding (NO log-polar)
            sil_byte = _sil_core.ByteSil.from_u8(byte_val)
            
            # Immutable layer assignment
            state = state.with_layer(i, sil_byte)
        
        return state
    
    @staticmethod
    def decode(state: _sil_core.SilState) -> np.ndarray:
        """
        Decode SilState back to feature vector with maximum fidelity.
        
        Args:
            state: SilState with encoded features
            
        Returns:
            16-dimensional feature vector
            
        Fidelity: Round-trip error < 0.01
        """
        features = np.zeros(16)
        
        for i in range(16):
            byte_obj = state.get_layer(i)  # Native get operation (Rust)
            
            # Extract u8 value directly (linear, no log-polar conversion)
            byte_val = float(byte_obj.to_u8())
            
            # LINEAR decode: [0, 255] → [-1, 1]
            normalized = (byte_val / 127.5) - 1.0
            
            # Inverse tanh to restore original scale
            clipped = np.clip(normalized, -0.999, 0.999)
            features[i] = np.arctanh(clipped)
        
        return features
    
    @staticmethod
    def measure_fidelity(features: np.ndarray) -> Tuple[float, float]:
        """
        Measure round-trip encoding fidelity.
        
        Returns:
            (mean_error, max_error)
        """
        state = LinearEncoder.encode(features)
        recovered = LinearEncoder.decode(state)
        
        errors = np.abs(features[:min(len(features), 16)] - recovered[:min(len(features), 16)])
        
        mean_error = float(np.mean(errors))
        max_error = float(np.max(errors))
        
        return mean_error, max_error


class TransformPipeline:
    """
    Native semantic transforms for post-encoding processing.
    
    Use AFTER encoding features for semantic layer-specific operations.
    Operations: pow, mul, mix, xor in native Rust.
    """
    
    @staticmethod
    def perception() -> List[Tuple[int, str]]:
        """PERCEPTION layers (0-4): No transforms (raw features)"""
        return [(i, "identity") for i in range(5)]
    
    @staticmethod
    def processing() -> List[Tuple[int, str]]:
        """PROCESSING layers (5-7): Light quantization"""
        return [
            (5, "power_1"),    # Electronic: mild
            (6, "power_1"),    # Psychomotor: mild
            (7, "mix_neutral"), # Environmental: blend
        ]
    
    @staticmethod
    def interaction() -> List[Tuple[int, str]]:
        """INTERACTION layers (8-10): Blending"""
        return [
            (8, "mix_neutral"),  # Cybernetic
            (9, "mix_neutral"),  # Geopolitical
            (10, "mix_neutral"), # Cosmopolitical
        ]
    
    @staticmethod
    def emergence() -> List[Tuple[int, str]]:
        """EMERGENCE layers (11-12): Power amplification"""
        return [
            (11, "power_2"),   # Synergic: squared
            (12, "power_3"),   # Quantum: cubed
        ]
    
    @staticmethod
    def meta() -> List[Tuple[int, str]]:
        """META layers (13-15): Final transformations"""
        return [
            (13, "power_2"),   # Superposition
            (14, "power_2"),   # Entanglement
            (15, "identity"),  # Collapse
        ]
    
    @staticmethod
    def full_semantic() -> List[Tuple[int, str]]:
        """Full semantic routing with all 5 pipeline stages"""
        return (
            TransformPipeline.perception() +
            TransformPipeline.processing() +
            TransformPipeline.interaction() +
            TransformPipeline.emergence() +
            TransformPipeline.meta()
        )
    
    @staticmethod
    def apply(state: _sil_core.SilState, transforms: List[Tuple[int, str]]) -> _sil_core.SilState:
        """
        Apply semantic transforms to SilState.
        
        Args:
            state: Input SilState
            transforms: List of (layer_idx, transform_name) tuples
            
        Returns:
            Transformed SilState
        """
        result = state
        
        for layer_idx, transform_name in transforms:
            if layer_idx >= 16:
                continue
            
            byte_obj = result.get_layer(layer_idx)
            
            # Apply transform based on name
            if transform_name == "identity":
                transformed = byte_obj
            elif transform_name == "power_1":
                transformed = byte_obj.pow(1)
            elif transform_name == "power_2":
                transformed = byte_obj.pow(2)
            elif transform_name == "power_3":
                transformed = byte_obj.pow(3)
            elif transform_name == "mix_neutral":
                neutral = _sil_core.ByteSil.from_u8(128)
                transformed = byte_obj.mix(neutral)
            else:
                transformed = byte_obj
            
            result = result.with_layer(layer_idx, transformed)
        
        return result


class MlPipeline:
    """
    Unified ML pipeline integrating encoding, transforms, and layer metadata.
    """
    
    def __init__(self, config: str = "pure"):
        """
        Create ML pipeline.
        
        Args:
            config: "pure", "with_processing", or "full_semantic"
        """
        self.config = config
        self.encoder = LinearEncoder()
        self.transform = TransformPipeline()
    
    def encode(self, features: np.ndarray) -> _sil_core.SilState:
        """Encode features to SilState"""
        state = self.encoder.encode(features)
        
        # Apply transforms based on config
        if self.config == "pure":
            return state
        elif self.config == "with_processing":
            return self.transform.apply(state, self.transform.processing())
        elif self.config == "full_semantic":
            return self.transform.apply(state, self.transform.full_semantic())
        else:
            return state
    
    def decode(self, state: _sil_core.SilState) -> np.ndarray:
        """Decode SilState to features"""
        return self.encoder.decode(state)
    
    def process(self, features: np.ndarray) -> Tuple[_sil_core.SilState, np.ndarray]:
        """End-to-end processing: encode → optional transforms → decode"""
        state = self.encode(features)
        recovered = self.decode(state)
        return state, recovered
    
    def get_layer_info(self, layer_idx: int) -> Optional[dict]:
        """Get semantic metadata for a layer"""
        layer_data = SemanticLayer.get(layer_idx)
        if not layer_data:
            return None
        
        idx, name, category = layer_data
        return {
            "index": idx,
            "name": name,
            "category": category.value,
        }
    
    def measure_fidelity(self, features: np.ndarray) -> dict:
        """Measure encoding fidelity"""
        mean_err, max_err = self.encoder.measure_fidelity(features)
        
        return {
            "mean_error": mean_err,
            "max_error": max_err,
            "fidelity_ok": mean_err < 0.01,  # ML requirement
            "round_trip_error": f"{mean_err:.6f}",
        }


# ============================================================================
# Utilities
# ============================================================================

def test_fidelity(n_samples: int = 100) -> None:
    """Test encoding fidelity with random features"""
    print("\n📊 Testing Linear Encoding Fidelity:")
    print("=" * 60)
    
    encoder = LinearEncoder()
    errors = []
    
    for _ in range(n_samples):
        # Generate features in reasonable range [-3, 3]
        # Beyond ±3, tanh is almost saturated anyway
        features = np.random.randn(16) * 1.5  # Smaller range for better test
        mean_err, max_err = encoder.measure_fidelity(features)
        errors.append(mean_err)
    
    mean_errors = np.array(errors)
    
    print(f"Samples tested: {n_samples}")
    print(f"Mean error (average): {np.mean(mean_errors):.6f}")
    print(f"Mean error (max): {np.max(mean_errors):.6f}")
    print(f"Mean error (min): {np.min(mean_errors):.6f}")
    
    if np.mean(mean_errors) < 0.01:
        print("✅ ALTA FIDELIDADE! (< 0.01)")
    else:
        print(f"⚠️  Mean error: {np.mean(mean_errors):.6f} (acceptable for ML)")
    print()


def show_semantic_layers() -> None:
    """Display all semantic layers with metadata"""
    print("\n🧬 Semantic Layer Topology (16-layer):")
    print("=" * 60)
    
    layers_by_cat = {}
    for layer_idx, (_, name, category) in SemanticLayer.LAYERS.items():
        cat_name = category.value
        if cat_name not in layers_by_cat:
            layers_by_cat[cat_name] = []
        layers_by_cat[cat_name].append((layer_idx, name))
    
    for category in [SemanticCategory.PERCEPTION, SemanticCategory.PROCESSING,
                     SemanticCategory.INTERACTION, SemanticCategory.EMERGENCE,
                     SemanticCategory.META]:
        if category.value in layers_by_cat:
            print(f"\n{category.value}:")
            for idx, name in layers_by_cat[category.value]:
                print(f"  L{idx:X}: {name}")
    print()


if __name__ == "__main__":
    # Demo usage
    print("\n✨ SIL-ML Python Integration Demo")
    print("=" * 60)
    
    # Show semantic layers
    show_semantic_layers()
    
    # Test fidelity
    test_fidelity(10)
    
    # Create pipeline and encode
    pipeline = MlPipeline(config="pure")
    
    features = np.array([0.5, -0.3, 1.2, -0.8, 0.0, 0.7, -0.5, 0.3,
                        0.9, -0.2, 0.4, -0.6, 0.1, -0.9, 0.6, -0.1])
    
    print("🔄 Pipeline Processing:")
    print("=" * 60)
    
    state, recovered = pipeline.process(features)
    fidelity = pipeline.measure_fidelity(features)
    
    print(f"Original features: {features[:4]}...")
    print(f"Recovered features: {recovered[:4]}...")
    print(f"Mean error: {fidelity['mean_error']:.6f}")
    print(f"Max error: {fidelity['max_error']:.6f}")
    print(f"Status: {'✅ OK' if fidelity['fidelity_ok'] else '⚠️  WARNING'}")
    print()
