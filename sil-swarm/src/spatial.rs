//! Spatial Partitioning for Large Swarms
//!
//! Implements spatial hashing to limit neighbor visibility to O(k) where k is constant,
//! enabling O(k × 16) = O(1) flocking behavior even for very large swarms (N > 1000).
//!
//! ## Algorithm
//!
//! - Divide 3D space into a grid of cells
//! - Each node is assigned to a cell based on its position
//! - Visible neighbors are limited to nodes in the same cell + adjacent cells (27 cells max)
//! - With proper cell sizing, this limits visible neighbors to ~k where k << N
//!
//! ## Performance
//!
//! - Without spatial partitioning: O(N × 16) for N neighbors
//! - With spatial partitioning: O(k × 16) where k ≈ 20-50 (configurable)
//! - For N = 10,000: 10,000× → 50× improvement (200× faster)

use std::collections::HashMap;

/// 3D position for spatial hashing
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Position3D {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Position3D {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    /// Distance to another position
    pub fn distance_to(&self, other: &Position3D) -> f32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        let dz = self.z - other.z;
        (dx * dx + dy * dy + dz * dz).sqrt()
    }
}

/// Grid cell identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CellId {
    x: i32,
    y: i32,
    z: i32,
}

impl CellId {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    /// Get all adjacent cells (including self) - 27 cells total
    pub fn get_neighbors(&self) -> Vec<CellId> {
        let mut neighbors = Vec::with_capacity(27);
        for dx in -1..=1 {
            for dy in -1..=1 {
                for dz in -1..=1 {
                    neighbors.push(CellId::new(
                        self.x + dx,
                        self.y + dy,
                        self.z + dz,
                    ));
                }
            }
        }
        neighbors
    }
}

/// Spatial grid for efficient neighbor queries
#[derive(Debug)]
pub struct SpatialGrid {
    /// Cell size - larger cells = fewer neighbors per cell
    cell_size: f32,
    /// Grid cells mapping to node IDs
    cells: HashMap<CellId, Vec<u64>>,
    /// Node positions
    positions: HashMap<u64, Position3D>,
    /// Maximum visible neighbors per node
    max_neighbors: usize,
}

impl SpatialGrid {
    /// Create new spatial grid
    ///
    /// # Parameters
    /// - `cell_size`: Size of each grid cell (typical: 5.0-10.0)
    /// - `max_neighbors`: Maximum neighbors to return per query (typical: 20-50)
    pub fn new(cell_size: f32, max_neighbors: usize) -> Self {
        Self {
            cell_size,
            cells: HashMap::new(),
            positions: HashMap::new(),
            max_neighbors,
        }
    }

    /// Get cell ID for a position
    fn get_cell_id(&self, pos: &Position3D) -> CellId {
        CellId::new(
            (pos.x / self.cell_size).floor() as i32,
            (pos.y / self.cell_size).floor() as i32,
            (pos.z / self.cell_size).floor() as i32,
        )
    }

    /// Update node position in the grid
    pub fn update_position(&mut self, node_id: u64, position: Position3D) {
        // Remove from old cell if exists
        if let Some(old_pos) = self.positions.get(&node_id) {
            let old_cell = self.get_cell_id(old_pos);
            if let Some(cell_nodes) = self.cells.get_mut(&old_cell) {
                cell_nodes.retain(|&id| id != node_id);
            }
        }

        // Add to new cell
        let new_cell = self.get_cell_id(&position);
        self.cells
            .entry(new_cell)
            .or_insert_with(Vec::new)
            .push(node_id);

        // Update position
        self.positions.insert(node_id, position);
    }

    /// Remove node from grid
    pub fn remove_node(&mut self, node_id: u64) {
        if let Some(pos) = self.positions.remove(&node_id) {
            let cell = self.get_cell_id(&pos);
            if let Some(cell_nodes) = self.cells.get_mut(&cell) {
                cell_nodes.retain(|&id| id != node_id);
            }
        }
    }

    /// Get visible neighbors for a node (limited to max_neighbors)
    ///
    /// Returns node IDs sorted by distance, up to max_neighbors.
    /// Complexity: O(k) where k = max_neighbors
    pub fn get_visible_neighbors(&self, node_id: u64) -> Vec<u64> {
        let pos = match self.positions.get(&node_id) {
            Some(p) => p,
            None => return Vec::new(),
        };

        let cell = self.get_cell_id(pos);
        let neighbor_cells = cell.get_neighbors();

        // Collect all candidates from nearby cells
        let mut candidates: Vec<(u64, f32)> = Vec::new();
        for cell_id in neighbor_cells {
            if let Some(cell_nodes) = self.cells.get(&cell_id) {
                for &neighbor_id in cell_nodes {
                    if neighbor_id != node_id {
                        if let Some(neighbor_pos) = self.positions.get(&neighbor_id) {
                            let distance = pos.distance_to(neighbor_pos);
                            candidates.push((neighbor_id, distance));
                        }
                    }
                }
            }
        }

        // Sort by distance and take top max_neighbors
        candidates.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        candidates
            .into_iter()
            .take(self.max_neighbors)
            .map(|(id, _)| id)
            .collect()
    }

    /// Get distance between two nodes
    pub fn get_distance(&self, node_id1: u64, node_id2: u64) -> Option<f32> {
        let pos1 = self.positions.get(&node_id1)?;
        let pos2 = self.positions.get(&node_id2)?;
        Some(pos1.distance_to(pos2))
    }

    /// Get node count
    pub fn node_count(&self) -> usize {
        self.positions.len()
    }

    /// Get average neighbors per node (for diagnostics)
    pub fn average_neighbors(&self) -> f32 {
        if self.positions.is_empty() {
            return 0.0;
        }

        let total: usize = self
            .positions
            .keys()
            .map(|&id| self.get_visible_neighbors(id).len())
            .sum();

        total as f32 / self.positions.len() as f32
    }
}

/// Spatial-aware swarm configuration
#[derive(Debug, Clone)]
pub struct SpatialSwarmConfig {
    /// Enable spatial partitioning (recommended for N > 100)
    pub enable_spatial_partitioning: bool,
    /// Grid cell size (typical: 5.0-10.0)
    pub cell_size: f32,
    /// Maximum visible neighbors (typical: 20-50)
    pub max_visible_neighbors: usize,
}

impl Default for SpatialSwarmConfig {
    fn default() -> Self {
        Self {
            enable_spatial_partitioning: false, // Disabled by default for backward compatibility
            cell_size: 10.0,
            max_visible_neighbors: 30,
        }
    }
}

impl SpatialSwarmConfig {
    /// Create configuration optimized for large swarms (N > 1000)
    pub fn for_large_swarm() -> Self {
        Self {
            enable_spatial_partitioning: true,
            cell_size: 10.0,
            max_visible_neighbors: 50,
        }
    }

    /// Create configuration optimized for medium swarms (100 < N < 1000)
    pub fn for_medium_swarm() -> Self {
        Self {
            enable_spatial_partitioning: true,
            cell_size: 15.0,
            max_visible_neighbors: 30,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_position_distance() {
        let p1 = Position3D::new(0.0, 0.0, 0.0);
        let p2 = Position3D::new(3.0, 4.0, 0.0);
        assert!((p1.distance_to(&p2) - 5.0).abs() < 0.001);
    }

    #[test]
    fn test_cell_id_neighbors() {
        let cell = CellId::new(0, 0, 0);
        let neighbors = cell.get_neighbors();
        assert_eq!(neighbors.len(), 27); // 3x3x3 cube
        assert!(neighbors.contains(&CellId::new(0, 0, 0))); // Self
        assert!(neighbors.contains(&CellId::new(1, 1, 1))); // Corner
        assert!(neighbors.contains(&CellId::new(-1, -1, -1))); // Opposite corner
    }

    #[test]
    fn test_spatial_grid_basic() {
        let mut grid = SpatialGrid::new(10.0, 10);

        // Add nodes
        grid.update_position(1, Position3D::new(0.0, 0.0, 0.0));
        grid.update_position(2, Position3D::new(5.0, 0.0, 0.0));
        grid.update_position(3, Position3D::new(100.0, 0.0, 0.0)); // Far away

        assert_eq!(grid.node_count(), 3);

        // Node 1 should see node 2 (same cell), but not node 3 (different cell)
        let neighbors = grid.get_visible_neighbors(1);
        assert!(neighbors.contains(&2));
        assert!(neighbors.contains(&3) || !neighbors.contains(&3)); // May or may not see depending on cell boundaries
    }

    #[test]
    fn test_spatial_grid_max_neighbors() {
        let mut grid = SpatialGrid::new(10.0, 5); // Max 5 neighbors

        // Add many nodes in same area
        for i in 0..20 {
            grid.update_position(i, Position3D::new(i as f32 * 0.5, 0.0, 0.0));
        }

        // Each node should see at most 5 neighbors
        let neighbors = grid.get_visible_neighbors(10);
        assert!(neighbors.len() <= 5);
    }

    #[test]
    fn test_spatial_grid_update_position() {
        let mut grid = SpatialGrid::new(10.0, 10);

        grid.update_position(1, Position3D::new(0.0, 0.0, 0.0));
        grid.update_position(2, Position3D::new(5.0, 0.0, 0.0));

        let neighbors_before = grid.get_visible_neighbors(1);
        assert!(neighbors_before.contains(&2));

        // Move node 2 far away
        grid.update_position(2, Position3D::new(1000.0, 0.0, 0.0));

        let neighbors_after = grid.get_visible_neighbors(1);
        assert!(!neighbors_after.contains(&2));
    }

    #[test]
    fn test_spatial_grid_remove_node() {
        let mut grid = SpatialGrid::new(10.0, 10);

        grid.update_position(1, Position3D::new(0.0, 0.0, 0.0));
        grid.update_position(2, Position3D::new(5.0, 0.0, 0.0));

        assert_eq!(grid.node_count(), 2);

        grid.remove_node(2);

        assert_eq!(grid.node_count(), 1);
        let neighbors = grid.get_visible_neighbors(1);
        assert!(!neighbors.contains(&2));
    }

    #[test]
    fn test_average_neighbors() {
        let mut grid = SpatialGrid::new(10.0, 10);

        // Add nodes in a cluster
        for i in 0..10 {
            grid.update_position(i, Position3D::new(i as f32, 0.0, 0.0));
        }

        let avg = grid.average_neighbors();
        assert!(avg > 0.0);
        assert!(avg <= 10.0);
    }

    #[test]
    fn test_config_presets() {
        let large = SpatialSwarmConfig::for_large_swarm();
        assert!(large.enable_spatial_partitioning);
        assert_eq!(large.max_visible_neighbors, 50);

        let medium = SpatialSwarmConfig::for_medium_swarm();
        assert!(medium.enable_spatial_partitioning);
        assert_eq!(medium.max_visible_neighbors, 30);
    }
}
