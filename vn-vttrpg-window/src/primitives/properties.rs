use crate::graphics::VertexDescription;
use crate::primitives::rect::Rect;
use crate::primitives::transform::Transform;

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable, Default)]
pub struct PrimitiveProperties {
    pub transform: Transform,
    pub clip_area: Rect,
}

impl VertexDescription for PrimitiveProperties {
    fn location_count() -> u32 {
        Transform::location_count() + Rect::location_count()
    }

    fn attributes(
        shader_location_start: u32,
        offset: wgpu::BufferAddress,
    ) -> Vec<wgpu::VertexAttribute> {
        let mut attrs = Transform::attributes(shader_location_start, offset);
        attrs.extend(Rect::attributes(
            shader_location_start + Transform::location_count(),
            offset + Transform::stride(),
        ));
        attrs
    }
}
