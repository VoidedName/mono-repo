struct Globals {
    resolution: vec2<f32>,
}

@group(0) @binding(0)
var<uniform> globals: Globals;

struct VertexInput {
    @location(0) v_position: vec2<f32>,
}

@group(1) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(1) @binding(1)
var s_diffuse: sampler;

struct InstanceInput {
    @location(1) i_translation: vec2<f32>,
    @location(2) i_rotation: f32,
    @location(3) i_scale: vec2<f32>,
    @location(4) i_origin: vec2<f32>,
    @location(5) i_clip_area: vec4<f32>,
    @location(6) i_size: vec2<f32>,
    @location(7) i_tint: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) local_pos: vec2<f32>,
    @location(1) size: vec2<f32>,
    @location(2) tint: vec4<f32>,
    @location(3) clip_area: vec4<f32>,
    @location(4) uv: vec2<f32>,
}

@vertex
fn vs_main(
    vertex: VertexInput,
    instance: InstanceInput,
) -> VertexOutput {
    var out: VertexOutput;

    let sin_r = sin(instance.i_rotation);
    let cos_r = cos(instance.i_rotation);
    let rot_mat = mat2x2<f32>(
        cos_r, sin_r,
        -sin_r, cos_r
    );

    // Vertex translation is in [0, 1] range.
    // We offset it by the instance origin to define the pivot point.
    let origin_offset_pos = (vertex.v_position - instance.i_origin) * instance.i_size * instance.i_scale;
    
    // Rotate around the origin
    let rotated_pos = rot_mat * origin_offset_pos;
    
    // Move it to the world translation
    let final_pos = rotated_pos + instance.i_translation;

    // Convert to clip space. (NDC)
    let ndc_x = (final_pos.x / globals.resolution.x) * 2.0 - 1.0;
    let ndc_y = 1.0 - (final_pos.y / globals.resolution.y) * 2.0;
    out.clip_position = vec4<f32>(ndc_x, ndc_y, 0.0, 1.0);
    
    out.local_pos = vertex.v_position * instance.i_size;
    out.size = instance.i_size;
    out.tint = instance.i_tint;
    out.clip_area = instance.i_clip_area;
    out.uv = vertex.v_position;

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Clip area check
    let clip_pos = in.clip_area.xy;
    let clip_size = in.clip_area.zw;
    if (in.local_pos.x < clip_pos.x || in.local_pos.x > clip_pos.x + clip_size.x ||
        in.local_pos.y < clip_pos.y || in.local_pos.y > clip_pos.y + clip_size.y) {
        discard;
    }

    let tex_color = textureSample(t_diffuse, s_diffuse, in.uv);
    return tex_color * in.tint;
}
