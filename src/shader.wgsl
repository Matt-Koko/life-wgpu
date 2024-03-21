@group(0) @binding(0) var<uniform> grid: vec2<f32>;
@group(0) @binding(1) var<storage> cell_state_in: array<u32>;
@group(0) @binding(2) var<storage, read_write> cell_state_out: array<u32>;

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

    let state = f32(cell_state_in[input.instance]);

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

    let red_lamp_location   = vec2<f32>(0.0, 1.0/6.0);
    let green_lamp_location = vec2<f32>(0.5, 1.0);
    let blue_lamp_location  = vec2<f32>(1.0, 1.0/6.0);

    let red_val = colour_lamp_brightness(cell_clipped, red_lamp_location);
    let green_val = colour_lamp_brightness(cell_clipped, green_lamp_location);
    let blue_val = colour_lamp_brightness(cell_clipped, blue_lamp_location);

    return vec4<f32>(red_val, green_val, blue_val, 1.0);
}

fn colour_lamp_brightness(cell_clipped: vec2<f32>, lamp_location: vec2<f32>) -> f32 {
    let distance_to_lamp = distance(cell_clipped, lamp_location);
    let brightness = 1 - distance_to_lamp;
    return brightness;
}

// Compute shader

fn cell_index(cell: vec2<u32>) -> u32 {
    return (cell.y % u32(grid.y)) * u32(grid.x) +
           (cell.x % u32(grid.x));
}

fn cell_active(x: u32, y: u32) -> u32 {
    return cell_state_in[cell_index(vec2(x, y))];
}

@compute @workgroup_size(8, 8)
fn cs_main(@builtin(global_invocation_id) cell: vec3<u32>) {
    // Determine how many active neighbors this cell has.
    let active_neighbours = cell_active(cell.x+1, cell.y+1) +
                          cell_active(cell.x+1, cell.y) +
                          cell_active(cell.x+1, cell.y-1) +
                          cell_active(cell.x, cell.y-1) +
                          cell_active(cell.x-1, cell.y-1) +
                          cell_active(cell.x-1, cell.y) +
                          cell_active(cell.x-1, cell.y+1) +
                          cell_active(cell.x, cell.y+1);

    let i = cell_index(cell.xy);

    // Conway's game of life rules:
    switch active_neighbours {
        case 2u: { // Active cells with 2 neighbors stay active.
            cell_state_out[i] = cell_state_in[i];
        }
        case 3u: { // Cells with 3 neighbors become or stay active.
            cell_state_out[i] = 1u;
        }
        default: { // Cells with < 2 or > 3 neighbors become inactive.
            cell_state_out[i] = 0u;
        }
    }
}