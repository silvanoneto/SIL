//! # Topological Data Analysis
//!
//! Extract topological features from data using persistent homology.
//!
//! ## Whitepaper Reference
//! - §C.45: Topological feature extraction
//!
//! ## Concepts
//!
//! - **Persistence Diagrams**: Birth-death pairs of topological features
//! - **Betti Numbers**: Count of n-dimensional holes
//! - **Wasserstein Distance**: Distance between persistence diagrams

use sil_core::state::{SilState, NUM_LAYERS};
use crate::stdlib::ml::utils::{magnitude, from_mag_phase};

/// A persistence pair (birth, death)
#[derive(Debug, Clone, Copy)]
pub struct PersistencePair {
    pub birth: f64,
    pub death: f64,
    pub dimension: usize,
}

impl PersistencePair {
    pub fn new(birth: f64, death: f64, dimension: usize) -> Self {
        Self { birth, death, dimension }
    }

    /// Persistence (lifetime) of the feature
    pub fn persistence(&self) -> f64 {
        (self.death - self.birth).abs()
    }

    /// Midpoint of the interval
    pub fn midpoint(&self) -> f64 {
        (self.birth + self.death) / 2.0
    }
}

/// Persistence diagram (collection of pairs)
#[derive(Debug, Clone)]
pub struct PersistenceDiagram {
    pub pairs: Vec<PersistencePair>,
}

impl PersistenceDiagram {
    pub fn new() -> Self {
        Self { pairs: Vec::new() }
    }

    pub fn add(&mut self, pair: PersistencePair) {
        self.pairs.push(pair);
    }

    /// Get pairs of a specific dimension
    pub fn dimension(&self, dim: usize) -> Vec<&PersistencePair> {
        self.pairs.iter().filter(|p| p.dimension == dim).collect()
    }

    /// Total persistence (sum of all lifetimes)
    pub fn total_persistence(&self) -> f64 {
        self.pairs.iter().map(|p| p.persistence()).sum()
    }
}

impl Default for PersistenceDiagram {
    fn default() -> Self {
        Self::new()
    }
}

/// Compute 0-dimensional persistence (connected components)
///
/// Uses a simplified Vietoris-Rips filtration.
/// Treats State as a 1D point cloud (magnitudes as values).
///
/// # Arguments
/// * `state` - Input state (magnitudes as point values)
///
/// # Returns
/// Persistence diagram with H0 features
pub fn persistence_h0(state: &SilState) -> PersistenceDiagram {
    let mut diagram = PersistenceDiagram::new();

    // Extract values and sort
    let mut values: Vec<(usize, f64)> = (0..NUM_LAYERS)
        .map(|i| (i, magnitude(&state.get(i))))
        .collect();

    values.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

    // Union-Find structure for component tracking
    let mut parent: Vec<usize> = (0..NUM_LAYERS).collect();
    let mut rank: Vec<usize> = vec![0; NUM_LAYERS];
    let mut birth_time: Vec<f64> = values.iter().map(|(_, v)| *v).collect();

    fn find(parent: &mut [usize], i: usize) -> usize {
        if parent[i] != i {
            parent[i] = find(parent, parent[i]);
        }
        parent[i]
    }

    fn union(parent: &mut [usize], rank: &mut [usize], birth: &mut [f64], i: usize, j: usize) -> Option<f64> {
        let pi = find(parent, i);
        let pj = find(parent, j);

        if pi == pj {
            return None; // Already same component
        }

        // Component with earlier birth survives
        let (survivor, dying) = if birth[pi] <= birth[pj] {
            (pi, pj)
        } else {
            (pj, pi)
        };

        let death_time = birth[dying];

        // Union by rank
        if rank[survivor] < rank[dying] {
            parent[survivor] = dying;
            birth[dying] = birth[survivor];
        } else if rank[survivor] > rank[dying] {
            parent[dying] = survivor;
        } else {
            parent[dying] = survivor;
            rank[survivor] += 1;
        }

        Some(death_time)
    }

    // Process in order of increasing value (sublevel filtration)
    for i in 0..NUM_LAYERS {
        let (idx, val) = values[i];

        // Check neighbors (treating as 1D circular array)
        let left = if idx > 0 { idx - 1 } else { NUM_LAYERS - 1 };
        let right = (idx + 1) % NUM_LAYERS;

        // Only connect to neighbors with smaller values (already in filtration)
        for &neighbor in &[left, right] {
            let neighbor_val = magnitude(&state.get(neighbor));
            if neighbor_val <= val {
                if let Some(death) = union(&mut parent, &mut rank, &mut birth_time, idx, neighbor) {
                    // A component died - record persistence pair
                    let birth_val = values.iter()
                        .find(|(i, _)| find(&mut parent.clone(), *i) == find(&mut parent.clone(), idx))
                        .map(|(_, v)| *v)
                        .unwrap_or(death);

                    if (death - birth_val).abs() > 1e-10 {
                        diagram.add(PersistencePair::new(birth_val, death, 0));
                    }
                }
            }
        }
    }

    // Add the "infinite" component (oldest survivor)
    let max_val = values.last().map(|(_, v)| *v).unwrap_or(0.0);
    let min_val = values.first().map(|(_, v)| *v).unwrap_or(0.0);
    diagram.add(PersistencePair::new(min_val, max_val, 0));

    diagram
}

/// Compute Betti numbers (simplified)
///
/// β₀ = number of connected components
/// β₁ = number of 1-dimensional holes (loops)
///
/// # Arguments
/// * `state` - Input state
/// * `threshold` - Connectivity threshold
///
/// # Returns
/// (β₀, β₁) as State with magnitudes
pub fn betti_numbers(state: &SilState, threshold: f64) -> (usize, usize) {
    // Build adjacency based on threshold
    let mut adj = [[false; NUM_LAYERS]; NUM_LAYERS];

    for i in 0..NUM_LAYERS {
        for j in i + 1..NUM_LAYERS {
            let vi = magnitude(&state.get(i));
            let vj = magnitude(&state.get(j));
            let dist = (vi - vj).abs();

            if dist < threshold {
                adj[i][j] = true;
                adj[j][i] = true;
            }
        }
    }

    // Count connected components (β₀) using DFS
    let mut visited = [false; NUM_LAYERS];
    let mut beta_0 = 0;

    fn dfs(node: usize, adj: &[[bool; NUM_LAYERS]; NUM_LAYERS], visited: &mut [bool; NUM_LAYERS]) {
        visited[node] = true;
        for (j, &connected) in adj[node].iter().enumerate() {
            if connected && !visited[j] {
                dfs(j, adj, visited);
            }
        }
    }

    for i in 0..NUM_LAYERS {
        if !visited[i] {
            beta_0 += 1;
            dfs(i, &adj, &mut visited);
        }
    }

    // Count edges and estimate β₁ using Euler characteristic
    // χ = V - E + F = β₀ - β₁ + β₂
    // For 1D complex (graph): β₂ = 0, so β₁ = E - V + β₀
    let mut edges = 0;
    for i in 0..NUM_LAYERS {
        for j in i + 1..NUM_LAYERS {
            if adj[i][j] {
                edges += 1;
            }
        }
    }

    let beta_1 = if edges >= NUM_LAYERS { edges - NUM_LAYERS + beta_0 } else { 0 };

    (beta_0, beta_1)
}

/// Betti curve: Betti numbers as function of filtration parameter
///
/// # Arguments
/// * `state` - Input state
/// * `num_samples` - Number of threshold samples
///
/// # Returns
/// State where layer i = β₀ at threshold i/num_samples
pub fn betti_curve(state: &SilState, num_samples: usize) -> SilState {
    let num_samples = num_samples.min(NUM_LAYERS);

    // Find value range
    let min_val = (0..NUM_LAYERS)
        .map(|i| magnitude(&state.get(i)))
        .fold(f64::INFINITY, |a, b| a.min(b));
    let max_val = (0..NUM_LAYERS)
        .map(|i| magnitude(&state.get(i)))
        .fold(f64::NEG_INFINITY, |a, b| a.max(b));

    let range = max_val - min_val;

    let mut result = SilState::vacuum();
    for i in 0..num_samples {
        let threshold = min_val + range * (i as f64 + 1.0) / num_samples as f64;
        let (beta_0, _) = betti_numbers(state, threshold);
        result = result.with_layer(i, from_mag_phase(beta_0 as f64, 0.0));
    }

    result
}

/// Wasserstein distance (p=2) between persistence diagrams
///
/// Simplified implementation using greedy matching.
///
/// # Arguments
/// * `d1` - First diagram
/// * `d2` - Second diagram
///
/// # Returns
/// Wasserstein distance
pub fn wasserstein_distance(d1: &PersistenceDiagram, d2: &PersistenceDiagram) -> f64 {
    if d1.pairs.is_empty() && d2.pairs.is_empty() {
        return 0.0;
    }

    // Greedy matching (not optimal, but O(n²))
    let pairs1: Vec<_> = d1.pairs.iter().collect();
    let pairs2: Vec<_> = d2.pairs.iter().collect();

    // Add diagonal points (birth = death) for unmatched pairs
    let _n1 = pairs1.len();
    let n2 = pairs2.len();

    let mut total_cost = 0.0;
    let mut matched2 = vec![false; n2];

    for p1 in &pairs1 {
        let mut best_cost = f64::INFINITY;
        let mut best_idx = None;

        // Find best match in pairs2
        for (j, p2) in pairs2.iter().enumerate() {
            if matched2[j] {
                continue;
            }

            let cost = ((p1.birth - p2.birth).powi(2) + (p1.death - p2.death).powi(2)).sqrt();
            if cost < best_cost {
                best_cost = cost;
                best_idx = Some(j);
            }
        }

        // Also consider matching to diagonal
        let diagonal_cost = p1.persistence() / 2.0_f64.sqrt();
        if diagonal_cost < best_cost {
            total_cost += diagonal_cost.powi(2);
        } else if let Some(idx) = best_idx {
            matched2[idx] = true;
            total_cost += best_cost.powi(2);
        }
    }

    // Unmatched pairs in d2 go to diagonal
    for (j, p2) in pairs2.iter().enumerate() {
        if !matched2[j] {
            let diagonal_cost = p2.persistence() / 2.0_f64.sqrt();
            total_cost += diagonal_cost.powi(2);
        }
    }

    total_cost.sqrt()
}

/// Bottleneck distance between persistence diagrams
///
/// Maximum cost in optimal matching.
///
/// # Arguments
/// * `d1` - First diagram
/// * `d2` - Second diagram
///
/// # Returns
/// Bottleneck distance
pub fn bottleneck_distance(d1: &PersistenceDiagram, d2: &PersistenceDiagram) -> f64 {
    if d1.pairs.is_empty() && d2.pairs.is_empty() {
        return 0.0;
    }

    let mut max_cost = 0.0f64;

    // For each pair in d1, find minimum cost to d2 or diagonal
    for p1 in &d1.pairs {
        let mut min_cost = p1.persistence() / 2.0_f64.sqrt(); // Diagonal cost

        for p2 in &d2.pairs {
            let cost = ((p1.birth - p2.birth).powi(2) + (p1.death - p2.death).powi(2)).sqrt();
            min_cost = min_cost.min(cost);
        }

        max_cost = max_cost.max(min_cost);
    }

    // Same for d2
    for p2 in &d2.pairs {
        let mut min_cost = p2.persistence() / 2.0_f64.sqrt();

        for p1 in &d1.pairs {
            let cost = ((p1.birth - p2.birth).powi(2) + (p1.death - p2.death).powi(2)).sqrt();
            min_cost = min_cost.min(cost);
        }

        max_cost = max_cost.max(min_cost);
    }

    max_cost
}

/// Persistence landscape (statistical summary)
///
/// Converts diagram to a function for machine learning.
///
/// # Arguments
/// * `diagram` - Persistence diagram
/// * `num_samples` - Number of sample points
///
/// # Returns
/// State representing the landscape
pub fn persistence_landscape(diagram: &PersistenceDiagram, num_samples: usize) -> SilState {
    let num_samples = num_samples.min(NUM_LAYERS);

    if diagram.pairs.is_empty() {
        return SilState::vacuum();
    }

    // Find range
    let min_t = diagram.pairs.iter().map(|p| p.birth).fold(f64::INFINITY, |a, b| a.min(b));
    let max_t = diagram.pairs.iter().map(|p| p.death).fold(f64::NEG_INFINITY, |a, b| a.max(b));
    let range = max_t - min_t;

    let mut result = SilState::vacuum();

    for i in 0..num_samples {
        let t = min_t + range * i as f64 / (num_samples - 1).max(1) as f64;

        // Landscape value at t: max of tent functions
        let mut landscape_val: f64 = 0.0;
        for p in &diagram.pairs {
            if t >= p.birth && t <= p.death {
                // Tent function: min(t - birth, death - t)
                let tent = (t - p.birth).min(p.death - t);
                landscape_val = landscape_val.max(tent);
            }
        }

        result = result.with_layer(i, from_mag_phase(landscape_val, 0.0));
    }

    result
}

/// Persistence entropy
///
/// H = -Σ pᵢ log(pᵢ) where pᵢ = persistence(i) / total_persistence
///
/// # Arguments
/// * `diagram` - Persistence diagram
///
/// # Returns
/// Entropy value
pub fn persistence_entropy(diagram: &PersistenceDiagram) -> f64 {
    let total = diagram.total_persistence();

    if total < 1e-10 {
        return 0.0;
    }

    let mut entropy = 0.0;
    for p in &diagram.pairs {
        let prob = p.persistence() / total;
        if prob > 1e-10 {
            entropy -= prob * prob.ln();
        }
    }

    entropy
}

/// Persistence image (vectorization for ML)
///
/// Discretizes diagram onto a grid with Gaussian smoothing.
///
/// # Arguments
/// * `diagram` - Persistence diagram
///
/// # Returns
/// State representing 4x4 persistence image (flattened)
pub fn persistence_image(diagram: &PersistenceDiagram) -> SilState {
    let mut result = SilState::vacuum();

    if diagram.pairs.is_empty() {
        return result;
    }

    // Find bounds
    let min_b = diagram.pairs.iter().map(|p| p.birth).fold(f64::INFINITY, |a, b| a.min(b));
    let max_b = diagram.pairs.iter().map(|p| p.birth).fold(f64::NEG_INFINITY, |a, b| a.max(b));
    let max_p = diagram.pairs.iter().map(|p| p.persistence()).fold(f64::NEG_INFINITY, |a, b| a.max(b));

    let b_range = (max_b - min_b).max(1e-10);
    let p_range = max_p.max(1e-10);

    // 4x4 grid
    let sigma = 0.2; // Gaussian width

    for i in 0..4 {
        for j in 0..4 {
            let grid_b = min_b + b_range * (i as f64 + 0.5) / 4.0;
            let grid_p = p_range * (j as f64 + 0.5) / 4.0;

            // Sum Gaussian contributions
            let mut val = 0.0;
            for p in &diagram.pairs {
                let db = (p.birth - grid_b) / b_range;
                let dp = (p.persistence() - grid_p) / p_range;
                let dist_sq = db * db + dp * dp;

                // Weight by persistence (more persistent = more important)
                let weight = p.persistence() / p_range;
                val += weight * (-dist_sq / (2.0 * sigma * sigma)).exp();
            }

            result = result.with_layer(i * 4 + j, from_mag_phase(val, 0.0));
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use crate::stdlib::ml::utils::{magnitude, phase, from_mag_phase};
    use sil_core::state::NUM_LAYERS;
    use super::*;

    #[test]
    fn test_persistence_h0() {
        // Simple increasing sequence
        let mut state = SilState::vacuum();
        for i in 0..NUM_LAYERS {
            state = state.with_layer(i, from_mag_phase(i as f64, 0.0));
        }

        let diagram = persistence_h0(&state);

        assert!(!diagram.pairs.is_empty());
    }

    #[test]
    fn test_betti_numbers() {
        // All values equal = one component
        let mut state = SilState::vacuum();
        for i in 0..NUM_LAYERS {
            state = state.with_layer(i, from_mag_phase(1.0, 0.0));
        }

        let (beta_0, _) = betti_numbers(&state, 0.5);
        assert_eq!(beta_0, 1); // One connected component
    }

    #[test]
    fn test_wasserstein_distance() {
        let mut d1 = PersistenceDiagram::new();
        d1.add(PersistencePair::new(0.0, 1.0, 0));

        let mut d2 = PersistenceDiagram::new();
        d2.add(PersistencePair::new(0.0, 1.0, 0));

        let dist = wasserstein_distance(&d1, &d2);
        assert!(dist < 1e-10); // Same diagrams = zero distance
    }

    #[test]
    fn test_persistence_entropy() {
        let mut diagram = PersistenceDiagram::new();
        diagram.add(PersistencePair::new(0.0, 1.0, 0));
        diagram.add(PersistencePair::new(0.0, 1.0, 0));

        let entropy = persistence_entropy(&diagram);
        assert!(entropy > 0.0); // Non-trivial entropy
    }

    #[test]
    fn test_persistence_landscape() {
        let mut diagram = PersistenceDiagram::new();
        diagram.add(PersistencePair::new(0.0, 2.0, 0));

        let landscape = persistence_landscape(&diagram, 16);

        // Landscape should peak in the middle
        let mid = magnitude(&landscape.get(8));
        let start = magnitude(&landscape.get(0));
        assert!(mid > start);
    }

    #[test]
    fn test_persistence_image() {
        let mut diagram = PersistenceDiagram::new();
        diagram.add(PersistencePair::new(0.0, 1.0, 0));
        diagram.add(PersistencePair::new(0.5, 1.5, 0));

        let image = persistence_image(&diagram);

        // Should have non-zero values
        let sum: f64 = (0..NUM_LAYERS)
            .map(|i| magnitude(&image.get(i)))
            .sum();
        assert!(sum > 0.0);
    }
}
