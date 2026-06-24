//! Fluid rendering system - Water and Lava
//!
//! Features:
//! - WGSL shaders for fluid rendering
//! - Flow simulation
//! - Refraction and luminosity effects
//! - Surface animation

use serde::{Serialize, Deserialize};

/// Fluid type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FluidType {
    Water,
    Lava,
    Milk,
    Potion,
}

/// Fluid render data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FluidRenderData {
    pub fluid_type: FluidType,
    
    // Visual properties
    pub base_color: [f32; 4],
    pub emissive: [f32; 3],
    pub opacity: f32,
    
    // Animation
    pub wave_speed: f32,
    pub wave_amplitude: f32,
    pub wave_frequency: f32,
    
    // Flow
    pub flow_speed: f32,
    pub flow_direction: [f32; 2], // 2D flow vector
    
    // Level (for water simulation)
    pub level: u8, // 0-15 like Minecraft
}

impl FluidRenderData {
    pub fn water() -> Self {
        Self {
            fluid_type: FluidType::Water,
            base_color: [0.3, 0.5, 1.0, 0.8],
            emissive: [0.0, 0.0, 0.0],
            opacity: 0.8,
            wave_speed: 2.0,
            wave_amplitude: 0.05,
            wave_frequency: 4.0,
            flow_speed: 1.0,
            flow_direction: [0.0, 0.0],
            level: 15,
        }
    }

    pub fn lava() -> Self {
        Self {
            fluid_type: FluidType::Lava,
            base_color: [1.0, 0.3, 0.0, 1.0],
            emissive: [1.0, 0.5, 0.0],
            opacity: 1.0,
            wave_speed: 0.5,
            wave_amplitude: 0.08,
            wave_frequency: 2.0,
            flow_speed: 0.3,
            flow_direction: [0.0, 0.0],
            level: 15,
        }
    }
}

/// Water shader code for WGSL
pub const WATER_VERTEX_SHADER: &str = r#"
struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) view_dir: vec3<f32>,
};

struct Uniforms {
    view_pos: vec4<f32>,
    view_proj: mat4x4<f32>,
    time: f32,
    fluid_level: f32,
    wave_amplitude: f32,
    wave_frequency: f32,
    wave_speed: f32,
};

@group(0) @binding(0) var<uniform> uniforms: Uniforms;

@vertex
fn vertex_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    
    // Animated wave displacement
    var pos = input.position;
    let wave = sin(pos.x * uniforms.wave_frequency + uniforms.time * uniforms.wave_speed) 
             * cos(pos.z * uniforms.wave_frequency + uniforms.time * uniforms.wave_speed * 0.7);
    pos.y += wave * uniforms.wave_amplitude;
    
    output.clip_position = uniforms.view_proj * vec4(pos, 1.0);
    output.world_position = pos;
    output.normal = input.normal;
    output.uv = input.uv;
    output.view_dir = normalize(uniforms.view_pos.xyz - pos);
    
    return output;
}
"#;

pub const WATER_FRAGMENT_SHADER: &str = r#"
struct FragmentInput {
    @location(0) world_position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) view_dir: vec3<f32>,
};

struct Uniforms {
    view_pos: vec4<f32>,
    view_proj: mat4x4<f32>,
    time: f32,
    fluid_level: f32,
    wave_amplitude: f32,
    wave_frequency: f32,
    wave_speed: f32,
};

@group(0) @binding(0) var<uniform> uniforms: Uniforms;
@group(0) @binding(1) var diffuse_texture: texture_2d<f32>;
@group(0) @binding(2) var texture_sampler: sampler;

@fragment
fn fragment_main(input: FragmentInput) -> @location(0) vec4<f32> {
    let base_color = vec4<f32>(0.3, 0.5, 1.0, 0.7);
    let emissive = vec3<f32>(0.0, 0.0, 0.0);
    
    // Fresnel effect for water
    let fresnel = pow(1.0 - max(dot(input.normal, input.view_dir), 0.0), 3.0);
    
    // Reflection/refraction blend
    let reflect_color = vec4<f32>(0.7, 0.85, 1.0, 0.5);
    let refract_color = base_color;
    let fresnel_blend = mix(refract_color, reflect_color, fresnel * 0.5);
    
    // Add caustics-like pattern
    let caustic_pattern = sin(input.world_position.x * 10.0 + uniforms.time) 
                        * cos(input.world_position.z * 10.0 + uniforms.time * 1.3);
    let caustics = (caustic_pattern + 1.0) * 0.1;
    
    var final_color = fresnel_blend + vec4(caustics * 0.2, caustics * 0.4, caustics, 0.0);
    
    // Surface highlight
    let specular = pow(max(dot(reflect(-input.view_dir, input.normal), vec3(0.5, 1.0, 0.5)), 0.0), 32.0);
    final_color = final_color + vec4(specular * 0.3, specular * 0.3, specular * 0.3, 0.0);
    
    return final_color;
}
"#;

/// Lava shader with glow effect
pub const LAVA_FRAGMENT_SHADER: &str = r#"
struct FragmentInput {
    @location(0) world_position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) view_dir: vec3<f32>,
};

struct Uniforms {
    view_pos: vec4<f32>,
    view_proj: mat4x4<f32>,
    time: f32,
    fluid_level: f32,
    wave_amplitude: f32,
    wave_frequency: f32,
    wave_speed: f32,
};

@group(0) @binding(0) var<uniform> uniforms: Uniforms;

@fragment
fn fragment_main(input: FragmentInput) -> @location(0) vec4<f32> {
    // Base lava colors
    let hot_color = vec4<f32>(1.0, 0.9, 0.5, 1.0);
    let cool_color = vec4<f32>(0.8, 0.2, 0.0, 1.0);
    let crust_color = vec4<f32>(0.3, 0.1, 0.0, 1.0);
    
    // Animated flow pattern
    let flow = sin(input.world_position.x * 3.0 + uniforms.time * 0.5) 
             * cos(input.world_position.z * 3.0 + uniforms.time * 0.3);
    let flow_pattern = (flow + 1.0) * 0.5;
    
    // Hot spots animation
    let hot_spot = sin(input.world_position.x * 8.0 + uniforms.time * 2.0) 
                 * cos(input.world_position.z * 8.0 - uniforms.time * 1.5);
    let hot_intensity = max(hot_spot, 0.0);
    
    // Color blending based on temperature
    var color = mix(cool_color, hot_color, flow_pattern * 0.5 + hot_intensity * 0.5);
    
    // Crusty edges
    let crust_threshold = 0.7;
    if flow_pattern > crust_threshold {
        color = mix(color, crust_color, (flow_pattern - crust_threshold) * 2.0);
    }
    
    // Glow effect
    let glow = vec3<f32>(1.0, 0.4, 0.0) * (1.0 + hot_intensity);
    color = color + vec4(glow * 0.3, 0.0);
    
    // Pulsing emission
    let pulse = (sin(uniforms.time * 3.0) + 1.0) * 0.1 + 0.9;
    color = color * pulse;
    
    return color;
}
"#;

/// Fluid simulation state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FluidSimulation {
    pub water_sources: Vec<WaterSource>,
    pub lava_sources: Vec<LavaSource>,
    pub tick_counter: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaterSource {
    pub position: [i32; 3],
    pub level: u8,
    pub flowing: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LavaSource {
    pub position: [i32; 3],
    pub level: u8,
    pub flowing: bool,
}

impl FluidSimulation {
    pub fn new() -> Self {
        Self {
            water_sources: Vec::new(),
            lava_sources: Vec::new(),
            tick_counter: 0,
        }
    }

    /// Add water source
    pub fn add_water(&mut self, x: i32, y: i32, z: i32, level: u8) {
        self.water_sources.push(WaterSource {
            position: [x, y, z],
            level,
            flowing: true,
        });
    }

    /// Add lava source
    pub fn add_lava(&mut self, x: i32, y: i32, z: i32, level: u8) {
        self.lava_sources.push(LavaSource {
            position: [x, y, z],
            level,
            flowing: true,
        });
    }

    /// Simulate fluid flow (called each tick)
    pub fn simulate(&mut self) {
        self.tick_counter += 1;
        
        // Simple water flow simulation
        for source in &mut self.water_sources {
            if source.flowing && self.tick_counter % 5 == 0 {
                // Water flows down and spreads
                source.level = source.level.saturating_sub(1);
                if source.level == 0 {
                    source.flowing = false;
                }
            }
        }
        
        // Simple lava flow (slower)
        for source in &mut self.lava_sources {
            if source.flowing && self.tick_counter % 20 == 0 {
                source.level = source.level.saturating_sub(1);
                if source.level == 0 {
                    source.flowing = false;
                }
            }
        }
        
        // Clean up dry sources periodically
        if self.tick_counter % 100 == 0 {
            self.water_sources.retain(|s| s.flowing);
            self.lava_sources.retain(|s| s.flowing);
        }
    }
}

impl Default for FluidSimulation {
    fn default() -> Self {
        Self::new()
    }
}