struct Globals {
    resolution: vec2<f32>,
}

@group(0) @binding(0)
var<uniform> globals: Globals;

struct VertexInput {
    @location(0) v_position: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) @interpolate(flat) instance_id: u32,
};

struct GlyphData {
    rect_min: vec2<f32>,
    rect_max: vec2<f32>,
    segment_start: u32,
    segment_count: u32,
};

struct Segment {
    p0: vec2<f32>,
    p1: vec2<f32>,
};

@group(1) @binding(0) var<storage, read> glyphs: array<GlyphData>;
@group(2) @binding(0) var<storage, read> segments: array<Segment>;

@vertex
fn vs_main(input: VertexInput, @builtin(instance_index) instance_index: u32) -> VertexOutput {
    let glyph = glyphs[instance_index];

    let pos = glyph.rect_min + input.v_position * (glyph.rect_max - glyph.rect_min);

    var out: VertexOutput;
    // Map screen [0, w], [0, h] to [-1, 1], [1, -1]
    let normalized_pos = (pos / globals.resolution) * 2.0 - 1.0;
    out.position = vec4<f32>(normalized_pos.x, -normalized_pos.y, 0.0, 1.0);
    out.uv = pos; // Use screen space coordinates for winding test
    out.instance_id = instance_index;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let glyph = glyphs[in.instance_id];
    let p = in.uv;

    var winding: i32 = 0;

    for (var i: u32 = 0u; i < glyph.segment_count; i = i + 1u) {
        let seg = segments[glyph.segment_start + i];
        let a = seg.p0;
        let b = seg.p1;

        // Non-zero winding rule
        if (a.y <= p.y) {
            if (b.y > p.y && (b.x - a.x) * (p.y - a.y) - (p.x - a.x) * (b.y - a.y) > 0.0) {
                winding = winding + 1;
            }
        } else {
            if (b.y <= p.y && (b.x - a.x) * (p.y - a.y) - (p.x - a.x) * (b.y - a.y) < 0.0) {
                winding = winding - 1;
            }
        }
    }

    if (winding != 0) {
        return vec4<f32>(1.0, 1.0, 1.0, 1.0);
    }

    return vec4<f32>(0.0, 0.0, 0.0, 0.0);
}
