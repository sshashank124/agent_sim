const PI = radians(180.0);
const DELTA_TIME: f32 = 1.0 / 60.0;

struct Params {
    speed: f32,
    turning_speed: f32,
    sensor_distance: f32,
    sensor_angle: f32,
    sensor_radius: u32,
    frame_number: u32,
};

struct Agent {
    position: vec2<f32>,
    heading: f32,
    _pad0: u32,
};

@group(0) @binding(0) var<uniform> params: Params;
@group(0) @binding(1) var<storage, read_write> agents : array<Agent>;

@group(1) @binding(0) var canvas_in: texture_2d<f32>;
@group(1) @binding(1) var canvas_out: texture_storage_2d<rgba8unorm, write>;

fn clamp_screenspace(pos: vec2<f32>) -> vec2<f32> {
    return clamp(pos, vec2<f32>(-1), vec2<f32>(1));
}

// maps [-1, 1] to [0, width/height]
fn logical_to_physical(pos: vec2<f32>) -> vec2<i32> {
    return vec2<i32>((pos * 0.5 + 0.5) * vec2<f32>(textureDimensions(canvas_in)));
}

fn rng_next(n: u32) -> u32 {
    var state = n;
    state ^= 2747636419u;
    state *= 2654435769u;
    state ^= state >> 16;
    state *= 2654435769u;
    state ^= state >> 16;
    state *= 2654435769u;
    return state;
}

fn uint_to_float(n: u32) -> f32 {
    return f32(n) / 4294967295.0;
}

fn sense(agent: Agent, angle_offset: f32) -> f32 {
    let angle = agent.heading + angle_offset;
    let delta_pos = vec2<f32>(cos(angle), sin(angle));
    let sense_location = logical_to_physical(agent.position + delta_pos * params.sensor_distance);
    let r = i32(params.sensor_radius);

    var sum: f32 = 0;
    for (var x: i32 = sense_location.x - r; x <= sense_location.x + r; x += 1) {
        for (var y: i32 = sense_location.y - r; y <= sense_location.y + r; y += 1) {
            // sum += dot(textureLoad(canvas_in, vec2<i32>(x, y), 0).xy, vec2<f32>(1));
            // textureStore(canvas_out, vec2<i32>(x, y), vec4<f32>(0, 0, 1, 1));
            sum += dot(textureLoad(canvas_in, vec2<i32>(x, y), 0).xyz, vec3<f32>(1));
        }
    }
    return sum;
}

@compute @workgroup_size(64)
fn main(
    @builtin(global_invocation_id) global_invocation_id: vec3<u32>
) {
    let idx = global_invocation_id.x;
    if (idx >= arrayLength(&agents)) { return; }

    var agent = agents[idx];

    let physical_pos = logical_to_physical(agent.position);
    var seed = u32(physical_pos.y * 100000 + physical_pos.x) + params.frame_number + u32(idx);
    seed = rng_next(seed);
    let steer_scale = uint_to_float(seed);

    let w_left = sense(agent, -params.sensor_angle);
    let w_middle = sense(agent, 0.);
    let w_right = sense(agent, params.sensor_angle);

    if (w_left > w_right && w_left > w_middle) {
        agent.heading -= DELTA_TIME * steer_scale * params.speed * params.turning_speed;
    } else if (w_right > w_left && w_right > w_middle) {
        agent.heading += DELTA_TIME * steer_scale * params.speed * params.turning_speed;
    }

    let delta_position = vec2<f32>(cos(agent.heading), sin(agent.heading));
    agent.position = clamp_screenspace(agent.position + DELTA_TIME * params.speed * delta_position);

    if (agent.position.x <= -1 || agent.position.x >= 1) {
        agent.heading = PI - agent.heading;
    }

    if (agent.position.y <= -1 || agent.position.y >= 1) {
        agent.heading = -agent.heading;
    }

    agents[idx] = agent;
    // textureStore(canvas_out, logical_to_physical(agent.position), vec4<f32>(1, 1, 0, 1));
    textureStore(canvas_out, logical_to_physical(agent.position), vec4<f32>(1));
}
