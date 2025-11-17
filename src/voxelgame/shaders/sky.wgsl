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

    let uv = out.tex_coord;
    let ro = (camera.inverse_view * vec4(0.0, 0.0, 0.0, 1.0)).xyz;
    var rd = (camera.inverse_view * vec4(normalize(vec3(
        uv.x,
        -uv.y,
        1.0,
    )), 1.0)).xyz - ro;

    out.ray = rd;

    return out;
}

fn sample_sky(rd: vec3<f32>) -> vec3<f32> {
    let sky_color = vec3(0.3, 0.6, 1.0);
    let ground_color = vec3(0.1, 0.06, 0.03);

    let t = (dot(rd, vec3(0.0, 1.0, 0.0)) * 32.0) / 2.0 + 0.5;

    return mix(sky_color, ground_color, clamp(t, 0.0, 1.0));
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4(sample_sky(in.ray), 1.0);
}
