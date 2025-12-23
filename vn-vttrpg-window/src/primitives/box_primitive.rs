use crate::graphics::VertexDescription;
use crate::primitives::color::Color;
use crate::primitives::properties::PrimitiveProperties;

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct BoxPrimitive {
    pub common: PrimitiveProperties,
    pub size: [f32; 2],
    pub color: Color,
    pub border_color: Color,
    pub border_thickness: f32,
    pub corner_radius: f32,
}

impl VertexDescription for BoxPrimitive {
    fn stride() -> wgpu::BufferAddress {
        size_of::<Self>() as wgpu::BufferAddress
    }

    fn location_count() -> u32 {
        PrimitiveProperties::location_count() + 1 + Color::location_count() * 2 + 2 // size (1) + color (1) + border_color (1) + thickness (1) + radius (1) = 5 locations
    }

    fn size_in_buffer() -> wgpu::BufferAddress {
        size_of::<Self>() as wgpu::BufferAddress
    }

    fn attributes(
        shader_location_start: u32,
        offset: wgpu::BufferAddress,
    ) -> Vec<wgpu::VertexAttribute> {
        let mut attrs = PrimitiveProperties::attributes(shader_location_start, offset);
        let mut current_location = shader_location_start + PrimitiveProperties::location_count();
        let mut current_offset = offset + PrimitiveProperties::stride();

        // size
        attrs.push(wgpu::VertexAttribute {
            offset: current_offset,
            shader_location: current_location,
            format: wgpu::VertexFormat::Float32x2,
        });
        current_location += 1;
        current_offset += size_of::<[f32; 2]>() as wgpu::BufferAddress;

        // color
        attrs.extend(Color::attributes(current_location, current_offset));
        current_location += Color::location_count();
        current_offset += Color::stride();

        // border_color
        attrs.extend(Color::attributes(current_location, current_offset));
        current_location += Color::location_count();
        current_offset += Color::stride();

        // border_thickness (Float32)
        attrs.push(wgpu::VertexAttribute {
            offset: current_offset,
            shader_location: current_location,
            format: wgpu::VertexFormat::Float32,
        });
        current_location += 1;
        current_offset += size_of::<f32>() as wgpu::BufferAddress;

        // corner_radius (Float32)
        attrs.push(wgpu::VertexAttribute {
            offset: current_offset,
            shader_location: current_location,
            format: wgpu::VertexFormat::Float32,
        });

        attrs
    }
}
