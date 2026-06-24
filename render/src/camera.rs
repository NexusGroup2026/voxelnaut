//! Camera system for VoxelNaut
//!
//! First-person camera with mouse look and movement.

use core::math::{Vec3, Rotation, AABB};
use serde::{Serialize, Deserialize};

/// Camera mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CameraMode {
    FirstPerson,
    ThirdPerson,
    Spectator,
}

/// Camera projection type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Projection {
    Perspective,
    Orthographic,
}

/// Camera struct
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Camera {
    pub position: Vec3,
    pub rotation: Rotation,
    pub fov: f32,
    pub aspect_ratio: f32,
    pub near: f32,
    pub far: f32,
    pub projection: Projection,
    pub mode: CameraMode,
    pub view_distance: f32,
    pub bob_enabled: bool,
    pub bob_amount: f32,
    pub bob_time: f32,
}

impl Camera {
    pub fn new(position: Vec3, fov: f32, aspect_ratio: f32) -> Self {
        Self {
            position,
            rotation: Rotation::default(),
            fov,
            aspect_ratio,
            near: 0.1,
            far: 1000.0,
            projection: Projection::Perspective,
            mode: CameraMode::FirstPerson,
            view_distance: 256.0,
            bob_enabled: true,
            bob_amount: 0.0,
            bob_time: 0.0,
        }
    }

    /// Get forward direction vector
    #[inline]
    pub fn forward(&self) -> Vec3 {
        self.rotation.forward()
    }

    /// Get right direction vector
    #[inline]
    pub fn right(&self) -> Vec3 {
        self.rotation.right()
    }

    /// Get up direction vector
    #[inline]
    pub fn up(&self) -> Vec3 {
        self.rotation.up()
    }

    /// Get view matrix
    pub fn view_matrix(&self) -> [[f32; 4]; 4] {
        let forward = self.forward();
        let right = self.right();
        let up = self.up();
        
        // Create view matrix
        let eye = self.position;
        
        // Using look_at approach
        let center = eye + forward;
        
        // Calculate view matrix components
        let f = forward.normalize();
        let s = right.normalize();
        let u = up;
        
        let mut result = [[0.0f32; 4]; 4];
        result[0][0] = s.x;
        result[1][0] = s.y;
        result[2][0] = s.z;
        result[0][1] = u.x;
        result[1][1] = u.y;
        result[2][1] = u.z;
        result[0][2] = -f.x;
        result[1][2] = -f.y;
        result[2][2] = -f.z;
        result[3][3] = 1.0;
        
        // Translation
        result[0][3] = -s.dot(&eye);
        result[1][3] = -u.dot(&eye);
        result[2][3] = f.dot(&eye);
        
        result
    }

    /// Get projection matrix
    pub fn projection_matrix(&self) -> [[f32; 4]; 4] {
        match self.projection {
            Projection::Perspective => self.perspective_matrix(),
            Projection::Orthographic => self.orthographic_matrix(),
        }
    }

    /// Get perspective projection matrix
    fn perspective_matrix(&self) -> [[f32; 4]; 4] {
        let fov_rad = self.fov.to_radians();
        let f = 1.0 / (fov_rad / 2.0).tan();
        let aspect = self.aspect_ratio;
        let near = self.near;
        let far = self.far;
        
        let mut result = [[0.0f32; 4]; 4];
        result[0][0] = f / aspect;
        result[1][1] = f;
        result[2][2] = (far + near) / (near - far);
        result[2][3] = (2.0 * far * near) / (near - far);
        result[3][2] = -1.0;
        
        result
    }

    /// Get orthographic projection matrix
    fn orthographic_matrix(&self) -> [[f32; 4]; 4] {
        let right = self.view_distance;
        let top = self.view_distance / self.aspect_ratio;
        let near = self.near;
        let far = self.far;
        
        let mut result = [[0.0f32; 4]; 4];
        result[0][0] = 1.0 / right;
        result[1][1] = 1.0 / top;
        result[2][2] = -2.0 / (far - near);
        result[2][3] = -(far + near) / (far - near);
        result[3][3] = 1.0;
        
        result
    }

    /// Get view-projection matrix
    pub fn view_projection(&self) -> [[f32; 4]; 4] {
        self.multiply_matrices(self.view_matrix(), self.projection_matrix())
    }

    /// Multiply two 4x4 matrices
    fn multiply_matrices(&self, a: [[f32; 4]; 4], b: [[f32; 4]; 4]) -> [[f32; 4]; 4] {
        let mut result = [[0.0f32; 4]; 4];
        for i in 0..4 {
            for j in 0..4 {
                for k in 0..4 {
                    result[i][j] += a[i][k] * b[k][j];
                }
            }
        }
        result
    }

    /// Update camera bob (for walking)
    pub fn update_bob(&mut self, delta: f32, moving: bool) {
        if !self.bob_enabled {
            return;
        }
        
        if moving {
            self.bob_time += delta * 10.0;
            self.bob_amount = ((self.bob_time * 0.5).sin() * 0.05).abs();
        } else {
            self.bob_amount *= 0.9;
            if self.bob_amount < 0.001 {
                self.bob_amount = 0.0;
            }
        }
    }

    /// Get camera position with bob applied
    pub fn bobbed_position(&self) -> Vec3 {
        let mut pos = self.position;
        pos.y += self.bob_amount;
        pos
    }

    /// Get frustum planes for culling
    pub fn frustum_planes(&self) -> [Vec4; 6] {
        let vp = self.view_projection();
        
        // Extract frustum planes from view-projection matrix
        let mut planes = [Vec4::ZERO; 6];
        
        // Left plane
        planes[0] = Vec4::new(vp[0][3] + vp[0][0], vp[1][3] + vp[1][0], vp[2][3] + vp[2][0], vp[3][3] + vp[3][0]);
        // Right plane
        planes[1] = Vec4::new(vp[0][3] - vp[0][0], vp[1][3] - vp[1][0], vp[2][3] - vp[2][0], vp[3][3] - vp[3][0]);
        // Bottom plane
        planes[2] = Vec4::new(vp[0][3] + vp[0][1], vp[1][3] + vp[1][1], vp[2][3] + vp[2][1], vp[3][3] + vp[3][1]);
        // Top plane
        planes[3] = Vec4::new(vp[0][3] - vp[0][1], vp[1][3] - vp[1][1], vp[2][3] - vp[2][1], vp[3][3] - vp[3][1]);
        // Near plane
        planes[4] = Vec4::new(vp[0][3] + vp[0][2], vp[1][3] + vp[1][2], vp[2][3] + vp[2][2], vp[3][3] + vp[3][2]);
        // Far plane
        planes[5] = Vec4::new(vp[0][3] - vp[0][2], vp[1][3] - vp[1][2], vp[2][3] - vp[2][2], vp[3][3] - vp[3][2]);
        
        // Normalize planes
        for plane in &mut planes {
            let len = (plane.x * plane.x + plane.y * plane.y + plane.z * plane.z).sqrt();
            if len > 0.0 {
                plane.x /= len;
                plane.y /= len;
                plane.z /= len;
                plane.w /= len;
            }
        }
        
        planes
    }

    /// Check if an AABB is visible in the frustum
    pub fn is_visible(&self, aabb: &AABB) -> bool {
        let planes = self.frustum_planes();
        let center = aabb.center();
        let half = aabb.size() * 0.5;
        
        for plane in &planes {
            let dist = plane.x * center.x + plane.y * center.y + plane.z * center.z + plane.w;
            let max_dist = plane.x.abs() * half.x + plane.y.abs() * half.y + plane.z.abs() * half.z;
            if dist < -max_dist {
                return false;
            }
        }
        true
    }
}

/// 4D vector for plane calculations
#[derive(Debug, Clone, Copy)]
pub struct Vec4 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl Vec4 {
    pub fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self { x, y, z, w }
    }
    
    pub const ZERO: Self = Self { x: 0.0, y: 0.0, z: 0.0, w: 0.0 };
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            position: Vec3::ZERO,
            rotation: Rotation::default(),
            fov: 70.0,
            aspect_ratio: 16.0 / 9.0,
            near: 0.1,
            far: 1000.0,
            projection: Projection::Perspective,
            mode: CameraMode::FirstPerson,
            view_distance: 256.0,
            bob_enabled: true,
            bob_amount: 0.0,
            bob_time: 0.0,
        }
    }
}