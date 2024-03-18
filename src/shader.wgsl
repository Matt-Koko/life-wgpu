@group(0) @binding(0) var<uniform> grid: vec2<f32>;

// Vertex shader

struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) color: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
};

@vertex
fn vs_main(
    input: VertexInput,
    @builtin(instance_index) instance: u32,
) -> VertexOutput {
    var out: VertexOutput;
    
    out.color = input.color;

    let i = f32(instance);
    let cell = vec2<f32>(i % grid.x, floor(i / grid.x));
    let cell_offset = cell / grid * 2;

    let grid_pos = (input.position + 1) / grid - 1 + cell_offset;
    out.clip_position = vec4<f32>(grid_pos, 0.0, 1.0);

    return out;
}

// Fragment shader

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(in.color, 1.0);
}
 