//! Mesh system for VoxelNaut
//!
//! Greedy meshing and chunk mesh generation.

use core::math::{Vec3, BlockPos, Direction, CHUNK_SIZE};
use core::block::{BlockId, BLOCK_AIR, get_block};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

/// Vertex data for rendering
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub color: u32,
    pub tex_coord: [f32; 2],
    pub tex_layer: u32,
}

impl Vertex {
    pub fn new(x: f32, y: f32, z: f32, normal: [f32; 3], color: u32, u: f32, v: f32, layer: u32) -> Self {
        Self {
            position: [x, y, z],
            normal,
            color,
            tex_coord: [u, v],
            tex_layer: layer,
        }
    }
}

/// Mesh data structure
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

impl Mesh {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn clear(&mut self) {
        self.vertices.clear();
        self.indices.clear();
    }

    pub fn add_quad(&mut self, pos: Vec3, dir: Direction, size: u32, color: u32, tex_layer: u32) {
        let normal = dir.to_normal().to_array();
        let start_idx = self.vertices.len() as u32;

        let (p1, p2, p3, p4) = match dir {
            Direction::Up => {
                (
                    Vec3::new(pos.x, pos.y + 1.0, pos.z),
                    Vec3::new(pos.x + size as f32, pos.y + 1.0, pos.z),
                    Vec3::new(pos.x + size as f32, pos.y + 1.0, pos.z + size as f32),
                    Vec3::new(pos.x, pos.y + 1.0, pos.z + size as f32),
                )
            }
            Direction::Down => {
                (
                    Vec3::new(pos.x, pos.y, pos.z + size as f32),
                    Vec3::new(pos.x + size as f32, pos.y, pos.z + size as f32),
                    Vec3::new(pos.x + size as f32, pos.y, pos.z),
                    Vec3::new(pos.x, pos.y, pos.z),
                )
            }
            Direction::North => {
                (
                    Vec3::new(pos.x + size as f32, pos.y, pos.z),
                    Vec3::new(pos.x, pos.y, pos.z),
                    Vec3::new(pos.x, pos.y + size as f32, pos.z),
                    Vec3::new(pos.x + size as f32, pos.y + size as f32, pos.z),
                )
            }
            Direction::South => {
                (
                    Vec3::new(pos.x, pos.y, pos.z + size as f32),
                    Vec3::new(pos.x + size as f32, pos.y, pos.z + size as f32),
                    Vec3::new(pos.x + size as f32, pos.y + size as f32, pos.z + size as f32),
                    Vec3::new(pos.x, pos.y + size as f32, pos.z + size as f32),
                )
            }
            Direction::East => {
                (
                    Vec3::new(pos.x + size as f32, pos.y, pos.z + size as f32),
                    Vec3::new(pos.x + size as f32, pos.y, pos.z),
                    Vec3::new(pos.x + size as f32, pos.y + size as f32, pos.z),
                    Vec3::new(pos.x + size as f32, pos.y + size as f32, pos.z + size as f32),
                )
            }
            Direction::West => {
                (
                    Vec3::new(pos.x, pos.y, pos.z),
                    Vec3::new(pos.x, pos.y, pos.z + size as f32),
                    Vec3::new(pos.x, pos.y + size as f32, pos.z + size as f32),
                    Vec3::new(pos.x, pos.y + size as f32, pos.z),
                )
            }
        };

        // Add vertices
        self.vertices.push(Vertex::new(p1.x, p1.y, p1.z, normal, color, 0.0, 0.0, tex_layer));
        self.vertices.push(Vertex::new(p2.x, p2.y, p2.z, normal, color, 1.0, 0.0, tex_layer));
        self.vertices.push(Vertex::new(p3.x, p3.y, p3.z, normal, color, 1.0, 1.0, tex_layer));
        self.vertices.push(Vertex::new(p4.x, p4.y, p4.z, normal, color, 0.0, 1.0, tex_layer));

        // Add indices (two triangles)
        self.indices.push(start_idx);
        self.indices.push(start_idx + 1);
        self.indices.push(start_idx + 2);
        self.indices.push(start_idx);
        self.indices.push(start_idx + 2);
        self.indices.push(start_idx + 3);
    }

    pub fn merge(&mut self, other: &Mesh) {
        let offset = self.vertices.len() as u32;
        self.vertices.extend_from_slice(&other.vertices);
        for &idx in &other.indices {
            self.indices.push(idx + offset);
        }
    }
}

/// Greedy meshing result
#[derive(Debug, Clone)]
pub struct GreedyMesh {
    pub opaque: Mesh,
    pub transparent: Mesh,
}

/// Face data for comparison
#[derive(Debug, Clone, PartialEq)]
struct FaceData {
    block_id: BlockId,
    color: u32,
    tex_layer: u32,
}

/// Generate a greedy mesh for a chunk
pub fn generate_greedy_mesh<F>(blocks: F, chunk_pos: BlockPos) -> GreedyMesh
where
    F: Fn(i32, i32, i32) -> BlockId,
{
    let mut opaque = Mesh::new();
    let mut transparent = Mesh::new();

    // Generate mesh per-face for simplicity, greedy will be added later
    for y in 0..CHUNK_SIZE as i32 {
        for z in 0..CHUNK_SIZE as i32 {
            for x in 0..CHUNK_SIZE as i32 {
                let block_id = blocks(x, y, z);
                if block_id == BLOCK_AIR {
                    continue;
                }

                let block = get_block_unchecked(block_id);
                let world_x = (chunk_pos.x << 5) + x;
                let world_z = (chunk_pos.z << 5) + z;

                // Check each face
                for dir in Direction::all() {
                    let neighbor = blocks(
                        x + dir.offset().x,
                        y + dir.offset().y,
                        z + dir.offset().z,
                    );

                    let neighbor_block = get_block_unchecked(neighbor);
                    let neighbor_transparent = neighbor_block.is_transparent();

                    // Only render face if neighbor is transparent or air
                    if neighbor == BLOCK_AIR || neighbor_transparent {
                        let pos = Vec3::new(world_x as f32, y as f32, world_z as f32);
                        let color = block.color;
                        let tex_layer = block.texture[match dir {
                            Direction::Up => 2,
                            Direction::Down => 3,
                            _ => 0,
                        }];

                        // Split into opaque and transparent meshes
                        let mesh = if block.is_transparent() {
                            &mut transparent
                        } else {
                            &mut opaque
                        };

                        mesh.add_quad(pos, dir, 1, color, tex_layer as u32);
                    }
                }
            }
        }
    }

    GreedyMesh { opaque, transparent }
}

/// Generate mesh for a single chunk
pub fn generate_chunk_mesh(chunk_blocks: &[BlockId], chunk_pos: core::math::ChunkPos) -> Mesh {
    let mut mesh = Mesh::new();
    let base_x = (chunk_pos.x << 5) as f32;
    let base_z = (chunk_pos.z << 5) as f32;

    let get_block = |x, y, z| -> BlockId {
        if x < 0 || x >= CHUNK_SIZE as i32 || y < 0 || y >= CHUNK_SIZE as i32 || z < 0 || z >= CHUNK_SIZE as i32 {
            return BLOCK_AIR;
        }
        let idx = x as usize + (z as usize * CHUNK_SIZE) + (y as usize * CHUNK_SIZE * CHUNK_SIZE);
        chunk_blocks.get(idx).copied().unwrap_or(BLOCK_AIR)
    };

    for y in 0..CHUNK_SIZE as i32 {
        for z in 0..CHUNK_SIZE as i32 {
            for x in 0..CHUNK_SIZE as i32 {
                let block_id = get_block(x, y, z);
                if block_id == BLOCK_AIR {
                    continue;
                }

                let block = get_block_unchecked(block_id);

                // Check each face
                for dir in Direction::all() {
                    let neighbor_id = get_block(
                        x + dir.offset().x,
                        y + dir.offset().y,
                        z + dir.offset().z,
                    );

                    if neighbor_id == BLOCK_AIR {
                        let pos = Vec3::new(base_x + x as f32, y as f32, base_z + z as f32);
                        let color = block.color;
                        let tex_layer = block.texture[0] as u32;

                        mesh.add_quad(pos, dir, 1, color, tex_layer);
                    }
                }
            }
        }
    }

    mesh
}

/// Chunk mesh cache entry
#[derive(Debug, Clone)]
pub struct CachedMesh {
    pub mesh: Mesh,
    pub version: u64,
}

impl CachedMesh {
    pub fn new(mesh: Mesh) -> Self {
        Self { mesh, version: 0 }
    }
}