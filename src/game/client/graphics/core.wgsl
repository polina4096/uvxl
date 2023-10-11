//Vertex
struct CameraUniform {
    view_proj: mat4x4<f32>,
};

struct ModelUniform {
    transform: mat4x4<f32>,
};

@group(0)
@binding(0)
var<uniform> camera: CameraUniform;

struct InstanceInput {
    @location(5) model_matrix_0: vec4<f32>,
    @location(6) model_matrix_1: vec4<f32>,
    @location(7) model_matrix_2: vec4<f32>,
    @location(8) model_matrix_3: vec4<f32>,
};

struct VertexInput {
    @location(0) pos : vec3<f32>,
    @location(1) uv  : vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_pos : vec4<f32>,
    @location(0)       uv       : vec2<f32>,
};

@vertex
fn vertex_main(in: VertexInput, instance: InstanceInput) -> VertexOutput {
    let model_matrix = mat4x4<f32>(
        instance.model_matrix_0,
        instance.model_matrix_1,
        instance.model_matrix_2,
        instance.model_matrix_3,
    );

    var out: VertexOutput;
    out.clip_pos = camera.view_proj * model_matrix * vec4<f32>(in.pos, 1.0);
    out.uv       = in.uv;

    return out;
}

// Fragment
@group(1)
@binding(0)
var t0: texture_2d<f32>;
@group(1)
@binding(1)
var s0: sampler;

@fragment
fn fragment_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(t0, s0, in.uv);
}