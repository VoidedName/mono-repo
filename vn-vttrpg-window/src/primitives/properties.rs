use crate::graphics::VertexDescription;
use crate::primitives::rect::Rect;
use crate::primitives::transform::Transform;

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct PrimitiveProperties {
    pub transform: Transform,
    pub clip_area: Rect,
}

impl Default for PrimitiveProperties {
    fn default() -> Self {
        Self {
            transform: Transform::default(),
            clip_area: Rect::full_screen(),
        }
    }
}

impl VertexDescription for PrimitiveProperties {
    fn stride() -> wgpu::BufferAddress {
        size_of::<Self>() as wgpu::BufferAddress
    }

    fn location_count() -> u32 {
        Transform::location_count() + Rect::location_count()
    }

    fn size_in_buffer() -> wgpu::BufferAddress {
        size_of::<Self>() as wgpu::BufferAddress
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
