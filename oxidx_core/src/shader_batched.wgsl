// OxidX Batched Shader - WGSL
// Supports orthographic projection and UV coordinates for future texture support.

// Uniform buffer containing the projection matrix.
// This transforms pixel coordinates to clip space.
struct Uniforms {
    projection: mat4x4<f32>,
};

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

// Vertex shader input - matches our Vertex struct in Rust
struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) color: vec4<f32>,
    @location(2) uv: vec2<f32>,
};

// Vertex shader output / Fragment shader input
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) uv: vec2<f32>,
};

// Vertex shader entry point
// Transforms 2D pixel position to clip space using orthographic projection
@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    // Apply orthographic projection to convert pixel coords to clip space
    out.clip_position = uniforms.projection * vec4<f32>(in.position, 0.0, 1.0);
    out.color = in.color;
    out.uv = in.uv;
    return out;
}

// Fragment shader entry point
// For now, just outputs the interpolated color.
// Future: sample from texture using UV coordinates.
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}
