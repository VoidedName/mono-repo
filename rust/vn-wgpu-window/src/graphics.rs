use crate::errors::RenderError;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use wgpu::TextureFormat;
use winit::window::Window;

/// Wraps the core wgpu device and queue.
pub struct WgpuContext {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
}

/// Holds the graphical context for rendering, including the surface and device configuration.
pub struct GraphicsContext {
    pub wgpu: Rc<WgpuContext>,
    pub surface: wgpu::Surface<'static>,
    pub config: RefCell<wgpu::SurfaceConfiguration>,
    /// Indicates if the surface is ready for rendering (e.g., after the first resize).
    pub surface_ready_for_rendering: RefCell<bool>,
    pub window: Arc<Window>,
}

impl GraphicsContext {
    /// Creates a new graphics context for the given window.
    pub async fn new(window: Window) -> anyhow::Result<Self> {
        let window = Arc::new(window);
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
        let surface_format = TextureFormat::Rgba8Unorm;

        log::info!("Surface format: {:?}", surface_format);

        let alpha_mode = if surface_capabilities
            .alpha_modes
            .contains(&wgpu::CompositeAlphaMode::PreMultiplied)
        {
            wgpu::CompositeAlphaMode::PreMultiplied
        } else {
            surface_capabilities.alpha_modes[0]
        };

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::COPY_SRC
                | wgpu::TextureUsages::COPY_DST,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode,
            view_formats: vec![surface_format],
            desired_maximum_frame_latency: 2,
        };

        Ok(Self {
            wgpu: Rc::new(WgpuContext { device, queue }),
            surface,
            window,
            config: RefCell::new(config),
            surface_ready_for_rendering: RefCell::new(false),
        })
    }

    pub fn device(&self) -> &wgpu::Device {
        &self.wgpu.device
    }

    pub fn queue(&self) -> &wgpu::Queue {
        &self.wgpu.queue
    }

    pub fn size(&self) -> (u32, u32) {
        let config = self.config.borrow();
        (config.width, config.height)
    }
}

/// Defines the layout of vertices for a specific type to be used in a pipeline.
pub struct VertexLayout {
    pub array_stride: wgpu::BufferAddress,
    pub step_mode: wgpu::VertexStepMode,
    pub attributes: Vec<wgpu::VertexAttribute>,
}

/// A trait for types that can describe their vertex layout for GPU buffers.
pub trait VertexDescription: Sized {
    /// Returns the stride between consecutive elements of this type in a buffer.
    fn stride() -> wgpu::BufferAddress {
        size_of::<Self>() as wgpu::BufferAddress
    }

    /// Returns the number of shader locations occupied by this type.
    fn location_count() -> u32;

    /// Returns the total size in bytes occupied by this type in a buffer.
    fn size_in_buffer() -> wgpu::BufferAddress {
        size_of::<Self>() as wgpu::BufferAddress
    }

    /// Returns the vertex attributes for this type starting from the specified shader location.
    fn attributes(
        shader_location_start: u32,
        offset: wgpu::BufferAddress,
    ) -> Vec<wgpu::VertexAttribute>;

    /// Generates a [`VertexLayout`] for this type.
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
