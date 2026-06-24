//! Pathfinding for VoxelNaut
//!
//! A* pathfinding algorithm for mob navigation.

use core::math::{BlockPos, Vec3};
use std::collections::{BinaryHeap, HashMap, HashSet};

/// A* node for pathfinding
#[derive(Debug, Clone)]
struct PathNode {
    pos: BlockPos,
    g_cost: f32,
    h_cost: f32,
    parent: Option<BlockPos>,
}

impl PathNode {
    fn f_cost(&self) -> f32 {
        self.g_cost + self.h_cost
    }
}

impl PartialEq for PathNode {
    fn eq(&self, other: &Self) -> bool {
        self.pos == other.pos && self.f_cost() == other.f_cost()
    }
}

impl Eq for PathNode {}

impl PartialOrd for PathNode {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PathNode {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.f_cost().partial_cmp(&self.f_cost()).unwrap_or(std::cmp::Ordering::Equal)
    }
}

/// Pathfinding request
pub struct PathRequest {
    pub start: BlockPos,
    pub end: BlockPos,
    pub max_distance: i32,
}

/// Path result
#[derive(Debug, Clone)]
pub struct PathResult {
    pub waypoints: Vec<BlockPos>,
    pub reached: bool,
}

impl PathResult {
    pub fn empty() -> Self {
        Self {
            waypoints: Vec::new(),
            reached: false,
        }
    }

    pub fn success(waypoints: Vec<BlockPos>) -> Self {
        Self {
            waypoints,
            reached: true,
        }
    }

    pub fn partial(waypoints: Vec<BlockPos>) -> Self {
        Self {
            waypoints,
            reached: false,
        }
    }
}

/// A* pathfinder
pub struct Pathfinder {
    world_checker: Box<dyn Fn(BlockPos) -> bool + Send + Sync>,
}

impl Pathfinder {
    pub fn new(world_checker: impl Fn(BlockPos) -> bool + Send + Sync + 'static) -> Self {
        Self {
            world_checker: Box::new(world_checker),
        }
    }

    /// Find path using A*
    pub fn find_path(&self, request: &PathRequest) -> PathResult {
        let mut open_set: BinaryHeap<PathNode> = BinaryHeap::new();
        let mut came_from: HashMap<BlockPos, BlockPos> = HashMap::new();
        let mut g_score: HashMap<BlockPos, f32> = HashMap::new();
        let mut closed_set: HashSet<BlockPos> = HashSet::new();

        let start = request.start;
        let end = request.end;

        // Check if start and end are valid
        if !(self.world_checker)(start) || !(self.world_checker)(end) {
            return PathResult::empty();
        }

        g_score.insert(start, 0.0);
        open_set.push(PathNode {
            pos: start,
            g_cost: 0.0,
            h_cost: self.heuristic(start, end),
            parent: None,
        });

        let mut iterations = 0;
        let max_iterations = request.max_distance * request.max_distance * 10;

        while let Some(current) = open_set.pop() {
            iterations += 1;
            if iterations > max_iterations {
                break;
            }

            if current.pos == end {
                return PathResult::success(self.reconstruct_path(came_from, current.pos));
            }

            if closed_set.contains(&current.pos) {
                continue;
            }
            closed_set.insert(current.pos);

            // Check neighbors
            for neighbor in self.get_neighbors(current.pos) {
                if closed_set.contains(&neighbor) {
                    continue;
                }

                let tentative_g = current.g_cost + 1.0; // All moves cost 1

                let existing_g = *g_score.get(&neighbor).unwrap_or(&f32::MAX);

                if tentative_g < existing_g {
                    came_from.insert(neighbor, current.pos);
                    g_score.insert(neighbor, tentative_g);
                    let h = self.heuristic(neighbor, end);
                    open_set.push(PathNode {
                        pos: neighbor,
                        g_cost: tentative_g,
                        h_cost: h,
                        parent: Some(current.pos),
                    });
                }
            }
        }

        // Couldn't find exact path, return best effort
        if let Some(last_pos) = g_score.keys().max_by(|a, b| {
            let ha = self.heuristic(**a, end);
            let hb = self.heuristic(**b, end);
            ha.partial_cmp(&hb).unwrap_or(std::cmp::Ordering::Equal)
        }) {
            return PathResult::partial(self.reconstruct_path(came_from, *last_pos));
        }

        PathResult::empty()
    }

    fn heuristic(&self, a: BlockPos, b: BlockPos) -> f32 {
        // Euclidean distance
        let dx = (a.x - b.x).abs() as f32;
        let dy = (a.y - b.y).abs() as f32;
        let dz = (a.z - b.z).abs() as f32;
        (dx * dx + dy * dy + dz * dz).sqrt()
    }

    fn get_neighbors(&self, pos: BlockPos) -> Vec<BlockPos> {
        let mut neighbors = Vec::new();
        
        // 6-directional movement (can be extended for 8-directional)
        let directions = [
            BlockPos::UP,
            BlockPos::DOWN,
            BlockPos::LEFT,
            BlockPos::RIGHT,
            BlockPos::FORWARD,
            BlockPos::BACK,
        ];

        for dir in &directions {
            let new_pos = pos + *dir;
            if (self.world_checker)(new_pos) {
                neighbors.push(new_pos);
            }
        }

        neighbors
    }

    fn reconstruct_path(&self, came_from: HashMap<BlockPos, BlockPos>, mut current: BlockPos) -> Vec<BlockPos> {
        let mut path = vec![current];
        
        while let Some(parent) = came_from.get(&current) {
            current = *parent;
            path.push(current);
        }
        
        path.reverse();
        path
    }
}

/// Simplify path by removing unnecessary waypoints
pub fn simplify_path(path: &[BlockPos]) -> Vec<BlockPos> {
    if path.len() < 3 {
        return path.to_vec();
    }

    let mut simplified = vec![path[0]];
    let mut prev = path[0];
    
    for &point in path.iter().skip(1) {
        // Check if direction changes
        let dx = point.x - prev.x;
        let dy = point.y - prev.y;
        let dz = point.z - prev.z;
        
        // Keep if direction is different
        if dx != 0 || dy != 0 || dz != 0 {
            simplified.push(prev);
            prev = point;
        }
    }
    
    simplified.push(path[path.len() - 1]);
    simplified
}