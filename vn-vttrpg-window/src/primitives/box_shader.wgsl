struct VertexInput {
    @location(0) v_position: vec2<f32>,
}

struct InstanceInput {
    @location(1) i_pos: vec2<f32>,
    @location(2) i_size: vec2<f32>,
    @location(3) i_rotation: f32,
    @location(4) i_scale: vec2<f32>,
    @location(5) i_clip_area: vec4<f32>,
    @location(6) i_color: vec4<f32>,
    @location(7) i_border_color: vec4<f32>,
    @location(8) i_border_thickness: f32,
    @location(9) i_corner_radius: f32,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) local_pos: vec2<f32>,
    @location(1) size: vec2<f32>,
    @location(2) color: vec4<f32>,
    @location(3) border_color: vec4<f32>,
    @location(4) border_thickness: f32,
    @location(5) corner_radius: f32,
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

    // Vertex position is in [0, 1] range
    // We want to center it for rotation, or just use the pivot.
    // Let's assume vertex position [0, 1] and we scale it by instance size and scale.
    let scaled_pos = vertex.v_position * instance.i_size * instance.i_scale;
    
    // Rotate around (0,0) - which is the top-left of the box if v_position starts at 0,0
    let rotated_pos = rot_mat * scaled_pos;
    
    let final_pos = rotated_pos + instance.i_pos;

    // Convert to clip space. 
    // For now, let's assume a simple orthographic projection or just NDC.
    // Actually, we might need a camera/view uniform, but for now we'll just pass it through.
    // Assuming NDC for simplicity, we might need to adjust this later.
    out.clip_position = vec4<f32>(final_pos, 0.0, 1.0);
    
    out.local_pos = vertex.v_position * instance.i_size; // Pos in pixels/units relative to top-left
    out.size = instance.i_size;
    out.color = instance.i_color;
    out.border_color = instance.i_border_color;
    out.border_thickness = instance.i_border_thickness;
    out.corner_radius = instance.i_corner_radius;

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
            // Anti-aliasing could be added here
            color = in.border_color;
        }
    }
    
    return color;
}
