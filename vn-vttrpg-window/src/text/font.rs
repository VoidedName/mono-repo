use std::sync::Arc;

pub struct Font {
    pub data: Arc<Vec<u8>>,
    // ttf_parser::Face requires the data to live as long as the face.
    // Since we store data in an Arc, we can't easily store the Face directly if we want Font to be 'static.
    // We'll parse it on demand or use a different approach.
}

impl Font {
    pub fn new(data: Vec<u8>) -> Self {
        Self {
            data: Arc::new(data),
        }
    }

    pub fn face(&self) -> Result<ttf_parser::Face<'_>, ttf_parser::FaceParsingError> {
        ttf_parser::Face::parse(&self.data, 0)
    }
}
