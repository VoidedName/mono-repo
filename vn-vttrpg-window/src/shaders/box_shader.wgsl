struct Globals {
    resolution: vec2<f32>,
    _padding: vec2<f32>,
}

@group(0) @binding(0)
var<uniform> globals: Globals;

struct VertexInput {
    @location(0) v_position: vec2<f32>,
}

struct InstanceInput {
    @location(1) i_translation: vec2<f32>,
    @location(2) i_rotation: f32,
    @location(3) i_scale: vec2<f32>,
    @location(4) i_origin: vec2<f32>,
    @location(5) i_clip_area: vec4<f32>,
    @location(6) i_size: vec2<f32>,
    @location(7) i_color: vec4<f32>,
    @location(8) i_border_color: vec4<f32>,
    @location(9) i_border_thickness: f32,
    @location(10) i_corner_radius: f32,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) local_pos: vec2<f32>,
    @location(1) size: vec2<f32>,
    @location(2) color: vec4<f32>,
    @location(3) border_color: vec4<f32>,
    @location(4) border_thickness: f32,
    @location(5) corner_radius: f32,
    @location(6) clip_area: vec4<f32>,
    @location(7) world_pos: vec2<f32>,
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
    
    out.local_pos = vertex.v_position * instance.i_size; // Pos in pixels/units relative to top-left
    out.size = instance.i_size;
    out.color = instance.i_color;
    out.border_color = instance.i_border_color;
    out.border_thickness = instance.i_border_thickness;
    out.corner_radius = instance.i_corner_radius;
    out.clip_area = instance.i_clip_area;
    out.world_pos = final_pos;

    return out;
}

fn sd_rounded_box(p: vec2<f32>, b: vec2<f32>, r: vec4<f32>) -> f32 {
    var res_r = r;
    if (p.x > 0.0) {
        if (p.y > 0.0) { res_r.x = r.x; } else { res_r.x = r.w; }
    } else {
        if (p.y > 0.0) { res_r.x = r.y; } else { res_r.x = r.z; }
    }
    let q = abs(p) - b + res_r.x;
    return min(max(q.x, q.y), 0.0) + length(max(q, vec2<f32>(0.0))) - res_r.x;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Clip area check (relative to transform origin/local space)
    let clip_pos = in.clip_area.xy;
    let clip_size = in.clip_area.zw;
    if (in.local_pos.x < clip_pos.x || in.local_pos.x > clip_pos.x + clip_size.x ||
        in.local_pos.y < clip_pos.y || in.local_pos.y > clip_pos.y + clip_size.y) {
        discard;
    }

    // Center of the box for SDF
    let half_size = in.size * 0.5;
    let p = in.local_pos - half_size;
    
    // SDF for rounded box
    let d = sd_rounded_box(p, half_size, vec4<f32>(in.corner_radius));
    
    if (d > 0.0) {
        discard;
    }
    
    var color = in.color;
    
    if (in.border_thickness > 0.0) {
        let border_d = d + in.border_thickness;
        if (border_d > 0.0) {
            // anti aliasing
            let aa = smoothstep(0.0, 1.0, border_d);
            color = mix(in.color, in.border_color, aa);
        }
    }
    
    return color;
}
