struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) uv: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) texCoord: vec2<f32>,
};

struct Camera {
    view: mat4x4<f32>,
    inverse_view: mat4x4<f32>,
    projection: mat4x4<f32>,
};

struct Model {
    model: mat4x4<f32>,
}

@group(0) @binding(0)
var<uniform> model: Model;

@group(1) @binding(0)
var<uniform> camera: Camera;

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = camera.projection * camera.view * model.model * vec4(in.position, 1.0);
    out.texCoord = in.uv;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4(in.texCoord, 1.0, 1.0);
}
