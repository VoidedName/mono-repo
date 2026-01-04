use thiserror::Error;

#[derive(Error, Debug)]
pub enum AssetError {
    #[error("Failed to load image: {0}")]
    ImageError(#[from] image::ImageError),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Asset not found: {0}")]
    NotFound(String),
}

#[derive(Error, Debug)]
pub enum RenderError {
    #[error("Surface error: {0}")]
    SurfaceError(#[from] wgpu::SurfaceError),
    #[error("Failed to request adapter")]
    AdapterRequestFailed,
    #[error("Failed to request device: {0}")]
    DeviceRequestFailed(#[from] wgpu::RequestDeviceError),
    #[error("Pipeline creation failed: {0}")]
    PipelineError(String),
}
