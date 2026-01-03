// OxidX Shader - WGSL
// A simple shader for rendering colored rectangles.

// Vertex shader input - matches our Vertex struct in Rust
struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) color: vec4<f32>,
};

// Vertex shader output / Fragment shader input
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
};

// Vertex shader entry point
// Transforms 2D position to clip space and passes color through
@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    // Position is already in NDC (-1 to 1), just add z=0 and w=1
    out.clip_position = vec4<f32>(in.position, 0.0, 1.0);
    out.color = in.color;
    return out;
}

// Fragment shader entry point
// Simply outputs the interpolated color
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}
