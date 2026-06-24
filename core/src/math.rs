//! Mathematical types for 3D voxel world
//!
//! All positions, directions, and bounds are defined here.

use serde::{Serialize, Deserialize};
use std::fmt;
use std::ops::*;
use std::hash::{Hash, Hasher};

/// A position in world space (continuous)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3 {
    pub const ZERO: Self = Self { x: 0.0, y: 0.0, z: 0.0 };
    pub const ONE: Self = Self { x: 1.0, y: 1.0, z: 1.0 };
    pub const UP: Self = Self { x: 0.0, y: 1.0, z: 0.0 };
    pub const DOWN: Self = Self { x: 0.0, y: -1.0, z: 0.0 };
    pub const LEFT: Self = Self { x: -1.0, y: 0.0, z: 0.0 };
    pub const RIGHT: Self = Self { x: 1.0, y: 0.0, z: 0.0 };
    pub const FORWARD: Self = Self { x: 0.0, y: 0.0, z: 1.0 };
    pub const BACK: Self = Self { x: 0.0, y: 0.0, z: -1.0 };

    #[inline]
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    #[inline]
    pub fn splat(v: f32) -> Self {
        Self { x: v, y: v, z: v }
    }

    #[inline]
    pub fn from_array(arr: [f32; 3]) -> Self {
        Self { x: arr[0], y: arr[1], z: arr[2] }
    }

    #[inline]
    pub fn to_array(&self) -> [f32; 3] {
        [self.x, self.y, self.z]
    }

    #[inline]
    pub fn to_vec3d(&self) -> vecmath::Vector3<f32> {
        [self.x, self.y, self.z]
    }

    #[inline]
    pub fn length(&self) -> f32 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }

    #[inline]
    pub fn length_squared(&self) -> f32 {
        self.x * self.x + self.y * self.y + self.z * self.z
    }

    #[inline]
    pub fn normalize(&self) -> Self {
        let len = self.length();
        if len > 0.0 {
            Self {
                x: self.x / len,
                y: self.y / len,
                z: self.z / len,
            }
        } else {
            Self::ZERO
        }
    }

    #[inline]
    pub fn dot(&self, other: &Self) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    #[inline]
    pub fn cross(&self, other: &Self) -> Self {
        Self {
            x: self.y * other.z - self.z * other.y,
            y: self.z * other.x - self.x * other.z,
            z: self.x * other.y - self.y * other.x,
        }
    }

    #[inline]
    pub fn distance(&self, other: &Self) -> f32 {
        (*self - *other).length()
    }

    #[inline]
    pub fn distance_squared(&self, other: &Self) -> f32 {
        (*self - *other).length_squared()
    }

    #[inline]
    pub fn lerp(&self, other: &Self, t: f32) -> Self {
        *self + (*other - *self) * t
    }

    #[inline]
    pub fn clamp(&self, min: &Self, max: &Self) -> Self {
        Self {
            x: self.x.clamp(min.x, max.x),
            y: self.y.clamp(min.y, max.y),
            z: self.z.clamp(min.z, max.z),
        }
    }

    #[inline]
    pub fn abs(&self) -> Self {
        Self {
            x: self.x.abs(),
            y: self.y.abs(),
            z: self.z.abs(),
        }
    }

    #[inline]
    pub fn floor(&self) -> Self {
        Self {
            x: self.x.floor(),
            y: self.y.floor(),
            z: self.z.floor(),
        }
    }

    #[inline]
    pub fn ceil(&self) -> Self {
        Self {
            x: self.x.ceil(),
            y: self.y.ceil(),
            z: self.z.ceil(),
        }
    }

    #[inline]
    pub fn round(&self) -> Self {
        Self {
            x: self.x.round(),
            y: self.y.round(),
            z: self.z.round(),
        }
    }

    #[inline]
    pub fn min(&self, other: &Self) -> Self {
        Self {
            x: self.x.min(other.x),
            y: self.y.min(other.y),
            z: self.z.min(other.z),
        }
    }

    #[inline]
    pub fn max(&self, other: &Self) -> Self {
        Self {
            x: self.x.max(other.x),
            y: self.y.max(other.y),
            z: self.z.max(other.z),
        }
    }

    #[inline]
    pub fn sqrt(&self) -> Self {
        Self {
            x: self.x.sqrt(),
            y: self.y.sqrt(),
            z: self.z.sqrt(),
        }
    }
}

impl Default for Vec3 {
    fn default() -> Self {
        Self::ZERO
    }
}

impl Add for Vec3 {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self::new(self.x + other.x, self.y + other.y, self.z + other.z)
    }
}

impl AddAssign for Vec3 {
    fn add_assign(&mut self, other: Self) {
        self.x += other.x;
        self.y += other.y;
        self.z += other.z;
    }
}

impl Sub for Vec3 {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        Self::new(self.x - other.x, self.y - other.y, self.z - other.z)
    }
}

impl SubAssign for Vec3 {
    fn sub_assign(&mut self, other: Self) {
        self.x -= other.x;
        self.y -= other.y;
        self.z -= other.z;
    }
}

impl Mul<f32> for Vec3 {
    type Output = Self;
    fn mul(self, scalar: f32) -> Self {
        Self::new(self.x * scalar, self.y * scalar, self.z * scalar)
    }
}

impl MulAssign<f32> for Vec3 {
    fn mul_assign(&mut self, scalar: f32) {
        self.x *= scalar;
        self.y *= scalar;
        self.z *= scalar;
    }
}

impl Div<f32> for Vec3 {
    type Output = Self;
    fn div(self, scalar: f32) -> Self {
        Self::new(self.x / scalar, self.y / scalar, self.z / scalar)
    }
}

impl Neg for Vec3 {
    type Output = Self;
    fn neg(self) -> Self {
        Self::new(-self.x, -self.y, -self.z)
    }
}

/// A position in world space (integer, for block positions)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl BlockPos {
    pub const ZERO: Self = Self { x: 0, y: 0, z: 0 };
    pub const UP: Self = Self { x: 0, y: 1, z: 0 };
    pub const DOWN: Self = Self { x: 0, y: -1, z: 0 };
    pub const LEFT: Self = Self { x: -1, y: 0, z: 0 };
    pub const RIGHT: Self = Self { x: 1, y: 0, z: 0 };
    pub const FORWARD: Self = Self { x: 0, y: 0, z: 1 };
    pub const BACK: Self = Self { x: 0, y: 0, z: -1 };

    #[inline]
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    #[inline]
    pub fn from_vec3(v: &Vec3) -> Self {
        Self {
            x: v.x as i32,
            y: v.y as i32,
            z: v.z as i32,
        }
    }

    #[inline]
    pub fn from_array(arr: [i32; 3]) -> Self {
        Self { x: arr[0], y: arr[1], z: arr[2] }
    }

    #[inline]
    pub fn to_array(&self) -> [i32; 3] {
        [self.x, self.y, self.z]
    }

    #[inline]
    pub fn to_vec3(&self) -> Vec3 {
        Vec3::new(self.x as f32, self.y as f32, self.z as f32)
    }

    #[inline]
    pub fn to_vec3_centered(&self) -> Vec3 {
        Vec3::new(self.x as f32 + 0.5, self.y as f32 + 0.5, self.z as f32 + 0.5)
    }

    #[inline]
    pub fn length(&self) -> f32 {
        self.to_vec3().length()
    }

    #[inline]
    pub fn distance(&self, other: &Self) -> f32 {
        self.to_vec3().distance(&other.to_vec3())
    }

    #[inline]
    pub fn distance_squared(&self, other: &Self) -> f32 {
        self.to_vec3().distance_squared(&other.to_vec3())
    }

    #[inline]
    pub fn dot(&self, other: &Self) -> i32 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    #[inline]
    pub fn cross(&self, other: &Self) -> Self {
        Self {
            x: self.y * other.z - self.z * other.y,
            y: self.z * other.x - self.x * other.z,
            z: self.x * other.y - self.y * other.x,
        }
    }

    #[inline]
    pub fn min(&self, other: &Self) -> Self {
        Self {
            x: self.x.min(other.x),
            y: self.y.min(other.y),
            z: self.z.min(other.z),
        }
    }

    #[inline]
    pub fn max(&self, other: &Self) -> Self {
        Self {
            x: self.x.max(other.x),
            y: self.y.max(other.y),
            z: self.z.max(other.z),
        }
    }

    #[inline]
    pub fn abs(&self) -> Self {
        Self {
            x: self.x.abs(),
            y: self.y.abs(),
            z: self.z.abs(),
        }
    }
}

impl Default for BlockPos {
    fn default() -> Self {
        Self::ZERO
    }
}

impl Add for BlockPos {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self::new(self.x + other.x, self.y + other.y, self.z + other.z)
    }
}

impl AddAssign for BlockPos {
    fn add_assign(&mut self, other: Self) {
        self.x += other.x;
        self.y += other.y;
        self.z += other.z;
    }
}

impl Sub for BlockPos {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        Self::new(self.x - other.x, self.y - other.y, self.z - other.z)
    }
}

impl SubAssign for BlockPos {
    fn sub_assign(&mut self, other: Self) {
        self.x -= other.x;
        self.y -= other.y;
        self.z -= other.z;
    }
}

impl Mul<i32> for BlockPos {
    type Output = Self;
    fn mul(self, scalar: i32) -> Self {
        Self::new(self.x * scalar, self.y * scalar, self.z * scalar)
    }
}

/// A 2D position (for chunk coordinates)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ChunkPos {
    pub x: i32,
    pub z: i32,
}

impl ChunkPos {
    pub const ZERO: Self = Self { x: 0, z: 0 };

    #[inline]
    pub fn new(x: i32, z: i32) -> Self {
        Self { x, z }
    }

    #[inline]
    pub fn from_block(block: &BlockPos) -> Self {
        Self {
            x: block.x >> CHUNK_BITS,
            z: block.z >> CHUNK_BITS,
        }
    }

    #[inline]
    pub fn to_block(&self, y: i32) -> BlockPos {
        BlockPos::new(
            self.x << CHUNK_BITS,
            y,
            self.z << CHUNK_BITS,
        )
    }

    #[inline]
    pub fn to_array(&self) -> [i32; 2] {
        [self.x, self.z]
    }
}

impl Default for ChunkPos {
    fn default() -> Self {
        Self::ZERO
    }
}

impl Add for ChunkPos {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self::new(self.x + other.x, self.z + other.z)
    }
}

impl Sub for ChunkPos {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        Self::new(self.x - other.x, self.z - other.z)
    }
}

/// Chunk size in blocks
pub const CHUNK_SIZE: usize = 32;
pub const CHUNK_BITS: i32 = 5;
pub const CHUNK_VOLUME: usize = CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE;

/// AABB (Axis-Aligned Bounding Box)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct AABB {
    pub min: Vec3,
    pub max: Vec3,
}

impl AABB {
    #[inline]
    pub fn new(min: Vec3, max: Vec3) -> Self {
        Self { min, max }
    }

    #[inline]
    pub fn from_min_max(min_x: f32, min_y: f32, min_z: f32, max_x: f32, max_y: f32, max_z: f32) -> Self {
        Self {
            min: Vec3::new(min_x, min_y, min_z),
            max: Vec3::new(max_x, max_y, max_z),
        }
    }

    #[inline]
    pub fn from_center_size(center: Vec3, size: Vec3) -> Self {
        let half = size * 0.5;
        Self {
            min: center - half,
            max: center + half,
        }
    }

    #[inline]
    pub fn width(&self) -> f32 {
        self.max.x - self.min.x
    }

    #[inline]
    pub fn height(&self) -> f32 {
        self.max.y - self.min.y
    }

    #[inline]
    pub fn depth(&self) -> f32 {
        self.max.z - self.min.z
    }

    #[inline]
    pub fn center(&self) -> Vec3 {
        (self.min + self.max) * 0.5
    }

    #[inline]
    pub fn size(&self) -> Vec3 {
        self.max - self.min
    }

    #[inline]
    pub fn volume(&self) -> f32 {
        self.width() * self.height() * self.depth()
    }

    #[inline]
    pub fn contains_point(&self, point: &Vec3) -> bool {
        point.x >= self.min.x && point.x <= self.max.x
            && point.y >= self.min.y && point.y <= self.max.y
            && point.z >= self.min.z && point.z <= self.max.z
    }

    #[inline]
    pub fn intersects(&self, other: &AABB) -> bool {
        self.min.x <= other.max.x && self.max.x >= other.min.x
            && self.min.y <= other.max.y && self.max.y >= other.min.y
            && self.min.z <= other.max.z && self.max.z >= other.min.z
    }

    #[inline]
    pub fn intersection(&self, other: &AABB) -> Option<AABB> {
        let min = self.min.max(&other.min);
        let max = self.max.min(&other.max);
        if min.x <= max.x && min.y <= max.y && min.z <= max.z {
            Some(AABB::new(min, max))
        } else {
            None
        }
    }

    #[inline]
    pub fn union(&self, other: &AABB) -> AABB {
        AABB::new(self.min.min(&other.min), self.max.max(&other.max))
    }

    #[inline]
    pub fn expand(&self, amount: f32) -> Self {
        AABB::new(self.min - Vec3::splat(amount), self.max + Vec3::splat(amount))
    }

    pub fn raycast(&self, origin: Vec3, direction: Vec3, max_distance: f32) -> Option<RayHit> {
        let inv_dir = Vec3::new(
            if direction.x != 0.0 { 1.0 / direction.x } else { f32::INFINITY },
            if direction.y != 0.0 { 1.0 / direction.y } else { f32::INFINITY },
            if direction.z != 0.0 { 1.0 / direction.z } else { f32::INFINITY },
        );

        let t0 = (self.min - origin) * inv_dir;
        let t1 = (self.max - origin) * inv_dir;

        let tmin = t0.min(&t1);
        let tmax = t0.max(&t1);

        let mut tmin_final = tmin.x.max(tmin.y).max(tmin.z);
        let mut tmax_final = tmax.x.min(tmax.y).min(tmax.z);

        if tmax_final < 0.0 || tmin_final > tmax_final {
            return None;
        }

        let t = if tmin_final < 0.0 { tmax_final } else { tmin_final };
        if t < 0.0 || t > max_distance {
            return None;
        }

        let hit_point = origin + direction * t;
        
        let epsilon = 0.001;
        let normal = if (hit_point.x - self.min.x).abs() < epsilon {
            Vec3::LEFT
        } else if (hit_point.x - self.max.x).abs() < epsilon {
            Vec3::RIGHT
        } else if (hit_point.y - self.min.y).abs() < epsilon {
            Vec3::DOWN
        } else if (hit_point.y - self.max.y).abs() < epsilon {
            Vec3::UP
        } else if (hit_point.z - self.min.z).abs() < epsilon {
            Vec3::BACK
        } else {
            Vec3::FORWARD
        };

        Some(RayHit {
            point: hit_point,
            normal,
            distance: t,
        })
    }
}

impl Default for AABB {
    fn default() -> Self {
        Self {
            min: Vec3::ZERO,
            max: Vec3::ONE,
        }
    }
}

/// Ray cast hit result
#[derive(Debug, Clone, PartialEq)]
pub struct RayHit {
    pub point: Vec3,
    pub normal: Vec3,
    pub distance: f32,
}

/// Rotation in 3D space
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Rotation {
    pub yaw: f32,
    pub pitch: f32,
}

impl Rotation {
    pub fn new(yaw: f32, pitch: f32) -> Self {
        Self { yaw, pitch: pitch.clamp(-89.0, 89.0) }
    }

    #[inline]
    pub fn forward(&self) -> Vec3 {
        let yaw_rad = self.yaw.to_radians();
        let pitch_rad = self.pitch.to_radians();
        Vec3::new(
            -yaw_rad.sin() * pitch_rad.cos(),
            pitch_rad.sin(),
            -yaw_rad.cos() * pitch_rad.cos(),
        ).normalize()
    }

    #[inline]
    pub fn right(&self) -> Vec3 {
        let yaw_rad = self.yaw.to_radians();
        Vec3::new(yaw_rad.cos(), 0.0, -yaw_rad.sin()).normalize()
    }

    #[inline]
    pub fn up(&self) -> Vec3 {
        self.forward().cross(&self.right())
    }
}

impl Default for Rotation {
    fn default() -> Self {
        Self { yaw: 0.0, pitch: 0.0 }
    }
}

/// Direction enum for faces and orientations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Direction {
    Up,
    Down,
    North,
    South,
    East,
    West,
}

impl Direction {
    pub fn all() -> [Self; 6] {
        [
            Self::Up,
            Self::Down,
            Self::North,
            Self::South,
            Self::East,
            Self::West,
        ]
    }

    #[inline]
    pub fn opposite(&self) -> Self {
        match self {
            Self::Up => Self::Down,
            Self::Down => Self::Up,
            Self::North => Self::South,
            Self::South => Self::North,
            Self::East => Self::West,
            Self::West => Self::East,
        }
    }

    #[inline]
    pub fn offset(&self) -> BlockPos {
        match self {
            Self::Up => BlockPos::UP,
            Self::Down => BlockPos::DOWN,
            Self::North => BlockPos::BACK,
            Self::South => BlockPos::FORWARD,
            Self::East => BlockPos::RIGHT,
            Self::West => BlockPos::LEFT,
        }
    }

    #[inline]
    pub fn to_normal(&self) -> Vec3 {
        match self {
            Self::Up => Vec3::UP,
            Self::Down => Vec3::DOWN,
            Self::North => Vec3::BACK,
            Self::South => Vec3::FORWARD,
            Self::East => Vec3::RIGHT,
            Self::West => Vec3::LEFT,
        }
    }
}

impl Default for Direction {
    fn default() -> Self {
        Self::North
    }
}