#!/usr/bin/env python3
"""
Enhanced ByteSilMapper using new SIL-ML Python module.

Integrates semantic layer classification with high-fidelity encoding.
Replaces the inline ByteSilMapper class with module-based implementation.
"""

import numpy as np
import _sil_core
from sil_ml_python import (
    LinearEncoder,
    TransformPipeline,
    MlPipeline,
    SemanticLayer,
    SemanticCategory,
)


class EnhancedByteSilMapper:
    """
    Enhanced semantic mapper using SIL-ML Python module.
    
    Features:
    - HIGH-FIDELITY linear encoding (< 0.01 error)
    - Semantic layer classification (16-layer topology)
    - Optional post-encoding transforms
    - Unified interface for all configurations
    """
    
    def __init__(self, pipeline_config: str = "pure"):
        """
        Initialize mapper with pipeline configuration.
        
        Args:
            pipeline_config: "pure", "with_processing", or "full_semantic"
        """
        self.pipeline = MlPipeline(config=pipeline_config)
        self.encoder = LinearEncoder()
    
    def to_sil_state(self, feature_vector: np.ndarray) -> _sil_core.SilState:
        """
        Convert feature vector to SilState with high fidelity.
        
        Args:
            feature_vector: Input features (16 values max)
            
        Returns:
            SilState with encoded features
        """
        return self.pipeline.encode(feature_vector)
    
    def from_sil_state(self, state: _sil_core.SilState) -> np.ndarray:
        """
        Convert SilState back to feature vector.
        
        Args:
            state: SilState with encoded features
            
        Returns:
            Recovered feature vector (16 values)
        """
        return self.pipeline.decode(state)
    
    def get_layer_info(self, layer_idx: int) -> dict:
        """Get semantic metadata for a layer"""
        return self.pipeline.get_layer_info(layer_idx)
    
    def measure_fidelity(self, features: np.ndarray) -> dict:
        """Measure encoding fidelity"""
        return self.pipeline.measure_fidelity(features)
    
    @staticmethod
    def fidelity_summary(features: np.ndarray) -> str:
        """Get a string summary of fidelity metrics"""
        encoder = LinearEncoder()
        mean_err, max_err = encoder.measure_fidelity(features)
        
        status = "✅ ALTA FIDELIDADE!" if mean_err < 0.01 else "⚠️  Acceptable"
        return f"{status} (Mean: {mean_err:.6f}, Max: {max_err:.6f})"
    
    @staticmethod
    def get_semantic_layers() -> dict:
        """Get all semantic layers with metadata"""
        return SemanticLayer.get_all()
    
    @staticmethod
    def get_layers_by_category(category: str) -> list:
        """Get layers for a specific category"""
        try:
            cat_enum = SemanticCategory[category.upper()]
            return SemanticLayer.by_category(cat_enum)
        except (KeyError, AttributeError):
            return []


# ============================================================================
# Example Usage
# ============================================================================

if __name__ == "__main__":
    print("\n✨ Enhanced ByteSilMapper Demo")
    print("=" * 60)
    
    # Create mapper with pure LINEAR encoding
    mapper = EnhancedByteSilMapper(pipeline_config="pure")
    
    # Test features
    features = np.array([0.5, -0.3, 1.2, -0.8, 0.0, 0.7, -0.5, 0.3,
                        0.9, -0.2, 0.4, -0.6, 0.1, -0.9, 0.6, -0.1])
    
    print("\n1. Encode features:")
    print(f"   Input: {features[:4]}...")
    state = mapper.to_sil_state(features)
    print(f"   State: SilState with 16 layers encoded ✓")
    
    print("\n2. Decode back:")
    recovered = mapper.from_sil_state(state)
    print(f"   Output: {recovered[:4]}...")
    
    print("\n3. Measure fidelity:")
    fidelity = mapper.measure_fidelity(features)
    print(f"   Mean error: {fidelity['mean_error']:.6f}")
    print(f"   Max error: {fidelity['max_error']:.6f}")
    print(f"   Status: {fidelity['round_trip_error']}")
    print(f"   ML ready: {fidelity['fidelity_ok']}")
    
    print("\n4. Semantic layer info:")
    for i in [0, 5, 8, 11, 15]:
        info = mapper.get_layer_info(i)
        print(f"   L{i:X}: {info['name']} ({info['category']})")
    
    print("\n5. Layers by category:")
    perception = mapper.get_layers_by_category("perception")
    meta = mapper.get_layers_by_category("meta")
    print(f"   PERCEPTION: {perception}")
    print(f"   META: {meta}")
    
    print("\n✅ Enhanced mapper ready for ML models")
    print()
