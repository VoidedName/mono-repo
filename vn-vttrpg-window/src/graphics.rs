use std::sync::Arc;
use winit::window::Window;
use crate::errors::RenderError;

pub struct WgpuContext {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
}

pub struct GraphicsContext {
    pub wgpu: Arc<WgpuContext>,
    pub surface: wgpu::Surface<'static>,
    pub config: wgpu::SurfaceConfiguration,
    pub surface_ready_for_rendering: bool,
    pub window: Arc<Window>,
}

impl GraphicsContext {
    pub async fn new(window: Arc<Window>) -> anyhow::Result<Self> {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            // TODO (GPU BACKENDS): Investigate browser support and if this works. There appear to be some issues?
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });

        let surface = instance
            .create_surface(window.clone())
            .expect("Failed to create surface!");

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await;

        let adapter = match adapter {
            Ok(a) => a,
            Err(_) => return Err(RenderError::AdapterRequestFailed.into()),
        };

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                experimental_features: wgpu::ExperimentalFeatures::disabled(),
                required_limits: wgpu::Limits::default(),
                memory_hints: Default::default(),
                trace: wgpu::Trace::Off,
            })
            .await?;

        let surface_capabilities = surface.get_capabilities(&adapter);
        let surface_format = surface_capabilities
            .formats
            .iter()
            .copied()
            .find(|format| format.is_srgb())
            .unwrap_or(surface_capabilities.formats[0]);

        let alpha_mode = if surface_capabilities
            .alpha_modes
            .contains(&wgpu::CompositeAlphaMode::PreMultiplied)
        {
            wgpu::CompositeAlphaMode::PreMultiplied
        } else {
            surface_capabilities.alpha_modes[0]
        };

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        Ok(Self {
            wgpu: Arc::new(WgpuContext { device, queue }),
            surface,
            config,
            surface_ready_for_rendering: false,
            window,
        })
    }

    pub fn device(&self) -> &wgpu::Device {
        &self.wgpu.device
    }

    pub fn queue(&self) -> &wgpu::Queue {
        &self.wgpu.queue
    }
}

pub struct VertexLayout {
    pub array_stride: wgpu::BufferAddress,
    pub step_mode: wgpu::VertexStepMode,
    pub attributes: Vec<wgpu::VertexAttribute>,
}

pub trait VertexDescription: Sized {
    fn stride() -> wgpu::BufferAddress {
        size_of::<Self>() as wgpu::BufferAddress
    }
    
    fn location_count() -> u32;
    
    fn size_in_buffer() -> wgpu::BufferAddress {
        size_of::<Self>() as wgpu::BufferAddress
    }

    fn attributes(
        shader_location_start: u32,
        offset: wgpu::BufferAddress,
    ) -> Vec<wgpu::VertexAttribute>;

    fn vertex_description(
        shader_location_start: Option<u32>,
        offset: Option<wgpu::BufferAddress>,
        step_mode: wgpu::VertexStepMode,
    ) -> VertexLayout {
        VertexLayout {
            array_stride: Self::stride(),
            step_mode,
            attributes: Self::attributes(shader_location_start.unwrap_or(0), offset.unwrap_or(0)),
        }
    }
}
