//! Shader module placeholder
//! WGSL shaders are compiled at runtime via wgpu

use std::collections::HashMap;

/// Shader module - handles WGSL shader compilation
pub struct ShaderModule;

impl ShaderModule {
    pub fn new() -> Self {
        Self
    }
}

impl Default for ShaderModule {
    fn default() -> Self {
        Self::new()
    }
}