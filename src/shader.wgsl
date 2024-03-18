@group(0) @binding(0) var<uniform> grid: vec2<f32>;
@group(0) @binding(1) var<storage> cell_state: array<u32>;

// Vertex shader

struct VertexInput {
    @location(0) position: vec2<f32>,
    @builtin(instance_index) instance: u32,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) cell: vec2<f32>,
};

@vertex
fn vs_main(
    input: VertexInput,
) -> VertexOutput {
    var output: VertexOutput;
    
    let i = f32(input.instance);
    let cell = vec2<f32>(i % grid.x, floor(i / grid.x));
    let cell_offset = cell / grid * 2;
    output.cell = cell;

    let state = f32(cell_state[input.instance]);

    let grid_pos = (input.position * state + 1) / grid - 1 + cell_offset;
    output.clip_position = vec4<f32>(grid_pos, 0.0, 1.0);

    return output;
}

// Fragment shader 

 @fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    // Cell values range from 0 to grid-1.
    // Colors are in the range 0 to 1. So we divide cell position by grid.
    let cell_clipped = input.cell / grid;

    let red_lamp_location = vec2<f32>(0, 1.0/6.0);
    let green_lamp_location = vec2<f32>(0.5, 1);
    let blue_lamp_location = vec2<f32>(1, 1.0/6.0);

    let red_val = colourLampBrightness(cell_clipped, red_lamp_location);
    let green_val = colourLampBrightness(cell_clipped, green_lamp_location);
    let blue_val = colourLampBrightness(cell_clipped, blue_lamp_location);

    return vec4<f32>(red_val, green_val, blue_val, 1);
}

fn colourLampBrightness(cell_clipped: vec2<f32>, lamp_location: vec2<f32>) -> f32 {
    let distance_to_lamp = distance(cell_clipped, lamp_location);
    let brightness = 1 - distance_to_lamp;
    return brightness;
}