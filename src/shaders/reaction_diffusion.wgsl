struct SimulationParams {
    feed_rate: f32,
    kill_rate: f32,
    delta_u: f32,
    delta_v: f32,
    width: u32,
    height: u32,
}

struct UVPair {
    u: f32,
    v: f32,
}

@group(0) @binding(0) var<storage, read> uvs_in: array<UVPair>;
@group(0) @binding(1) var<storage, read_write> uvs_out: array<UVPair>;
@group(0) @binding(2) var<uniform> params: SimulationParams;

fn get_index(x: i32, y: i32) -> u32 {
    let width = i32(params.width);
    let height = i32(params.height);
    let wrapped_x = (x + width) % width;
    let wrapped_y = (y + height) % height;
    return u32(wrapped_y * width + wrapped_x);
}

fn get_laplacian(x: i32, y: i32) -> vec2f {
    let idx = get_index(x, y);
    let current = uvs_in[idx];
    
    var laplacian = vec2f(0.0);
    
    // Center weight
    laplacian -= vec2f(current.u, current.v) * 1.0;
    
    // Cardinal directions (weight 0.2)
    let left = uvs_in[get_index(x - 1, y)];
    let right = uvs_in[get_index(x + 1, y)];
    let up = uvs_in[get_index(x, y - 1)];
    let down = uvs_in[get_index(x, y + 1)];
    laplacian += vec2f(left.u, left.v) * 0.2;
    laplacian += vec2f(right.u, right.v) * 0.2;
    laplacian += vec2f(up.u, up.v) * 0.2;
    laplacian += vec2f(down.u, down.v) * 0.2;
    
    // Diagonal directions (weight 0.05)
    let up_left = uvs_in[get_index(x - 1, y - 1)];
    let up_right = uvs_in[get_index(x + 1, y - 1)];
    let down_left = uvs_in[get_index(x - 1, y + 1)];
    let down_right = uvs_in[get_index(x + 1, y + 1)];
    laplacian += vec2f(up_left.u, up_left.v) * 0.05;
    laplacian += vec2f(up_right.u, up_right.v) * 0.05;
    laplacian += vec2f(down_left.u, down_left.v) * 0.05;
    laplacian += vec2f(down_right.u, down_right.v) * 0.05;
    
    return laplacian;
}

@compute @workgroup_size(8, 8)
fn main(@builtin(global_invocation_id) global_id: vec3u) {
    let x = i32(global_id.x);
    let y = i32(global_id.y);
    
    if (x >= i32(params.width) || y >= i32(params.height)) {
        return;
    }
    
    let idx = get_index(x, y);
    let uv = uvs_in[idx];
    let reaction_rate = uv.u * uv.v * uv.v;
    
    let laplacian = get_laplacian(x, y);
    
    let delta_u = params.delta_u * laplacian.x - reaction_rate + params.feed_rate * (1.0 - uv.u);
    let delta_v = params.delta_v * laplacian.y + reaction_rate - (params.kill_rate + params.feed_rate) * uv.v;
    
    let new_u = clamp(uv.u + delta_u, 0.0, 1.0);
    let new_v = clamp(uv.v + delta_v, 0.0, 1.0);
    
    uvs_out[idx] = UVPair(new_u, new_v);
} 