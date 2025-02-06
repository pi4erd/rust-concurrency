struct VertexInput {
    @builtin(vertex_index) vertex_index: u32,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) ray: vec3<f32>,
    @location(1) tex_coord: vec2<f32>,
};

struct Camera {
    view: mat4x4<f32>,
    inverse_view: mat4x4<f32>,
    projection: mat4x4<f32>,
};

@group(0) @binding(0)
var<uniform> camera: Camera;

const positions: array<vec2<f32>, 4> = array(
    vec2(1.0, -1.0),
    vec2(1.0, 1.0),
    vec2(-1.0, -1.0),
    vec2(-1.0, 1.0),
);

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    out.clip_position = vec4(positions[in.vertex_index], 0.0, 1.0);
    out.tex_coord = positions[in.vertex_index];
    out.ray = -(vec4(normalize(vec3(out.tex_coord, 1.0)), 1.0)).xyz;

    return out;
}

fn sample_sky(rd: vec3<f32>) -> vec3<f32> {
    let sky_color = vec3(0.3, 0.6, 1.0);
    let sun_color = vec3(1.0, 0.97, 0.93);
    let sun_direction = normalize(vec3(2.0, 3.0, 1.0));

    let sun = pow(clamp(dot(sun_direction, rd), 0.0, 1.0), 32.0);
    return sky_color; // TODO: Fix rays
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4(sample_sky(in.ray), 1.0);
}
