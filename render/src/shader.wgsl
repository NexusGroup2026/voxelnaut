// VoxelNaut WGSL Shader
// Vertex shader
struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) color: vec4<f32>,
    @location(3) tex_coord: vec2<f32>,
    @location(4) tex_layer: u32,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) color: vec4<f32>,
    @location(3) tex_coord: vec2<f32>,
    @location(4) tex_layer: u32,
};

struct Uniforms {
    view_projection: mat4x4<f32>,
    camera_pos: vec3<f32>,
    _padding1: u32,
    time: f32,
    _padding2: array<u32, 3>,
};

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    output.clip_position = uniforms.view_projection * vec4<f32>(input.position, 1.0);
    output.world_position = input.position;
    output.normal = input.normal;
    output.color = input.color;
    output.tex_coord = input.tex_coord;
    output.tex_layer = input.tex_layer;
    return output;
}

// Fragment shader
@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    // Base color from vertex
    var color = input.color;
    
    // Simple directional lighting
    let light_dir = normalize(vec3<f32>(0.5, 1.0, 0.3));
    let light_intensity = max(dot(input.normal, light_dir), 0.0);
    let ambient = 0.4;
    let diffuse = light_intensity * 0.6;
    
    // Apply lighting
    color.rgb = color.rgb * (ambient + diffuse);
    
    // Add some fog based on distance
    let dist = distance(input.world_position, uniforms.camera_pos);
    let fog_start = 50.0;
    let fog_end = 200.0;
    let fog_factor = clamp((dist - fog_start) / (fog_end - fog_start), 0.0, 1.0);
    
    let fog_color = vec3<f32>(0.529, 0.808, 0.922); // Sky blue
    color.rgb = mix(color.rgb, fog_color, fog_factor);
    
    return color;
}