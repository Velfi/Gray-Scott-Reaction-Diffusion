struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
}

struct Uniforms {
    window_aspect_ratio: f32,
    simulation_aspect_ratio: f32,
}

// Bind groups
@group(0) @binding(0) var<uniform> uniforms: Uniforms;
@group(0) @binding(1) var t_texture: texture_2d<f32>;
@group(0) @binding(2) var s_texture: sampler;
@group(0) @binding(3) var<storage> lut: array<u32>;

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var out: VertexOutput;
    let x = f32(vertex_index & 1u);
    let y = f32((vertex_index >> 1u) & 1u);
    
    // Simple full screen quad
    let pos = vec2<f32>(x, y) * 2.0 - 1.0;
    
    // Simple texture coordinates
    let tex_coords = vec2<f32>(x, y);
    
    out.position = vec4<f32>(pos, 0.0, 1.0);
    out.tex_coords = tex_coords;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let dims = textureDimensions(t_texture);
    let px = vec2<i32>(
        i32(in.tex_coords.x * f32(dims.x)),
        i32(in.tex_coords.y * f32(dims.y))
    );
    
    // Clamp pixel coordinates to valid range
    let px_clamped = clamp(
        px,
        vec2<i32>(0),
        vec2<i32>(i32(dims.x) - 1, i32(dims.y) - 1)
    );
    
    let uv = textureLoad(t_texture, px_clamped, 0);
    
    // Map the v component (concentration) to LUT index
    let lut_index = u32(clamp(255.0 * uv.y, 0.0, 255.0));
    
    return vec4<f32>(
        f32(lut[lut_index]) / 255.0,
        f32(lut[lut_index + 256u]) / 255.0,
        f32(lut[lut_index + 512u]) / 255.0,
        1.0
    );
}

@fragment
fn fs_text_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let alpha = textureSample(t_texture, s_texture, in.tex_coords).r;
    // Mix between black background (when alpha > 0) and transparent
    let bg_alpha = min(alpha * 2.0, 0.8);
    // Mix between white text and background
    return vec4<f32>(mix(vec3<f32>(0.0), vec3<f32>(1.0), alpha), max(alpha, bg_alpha));
} 