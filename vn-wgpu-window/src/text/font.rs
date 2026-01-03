use std::rc::Rc;

pub struct Font {
    pub data: Rc<Vec<u8>>,
}

impl Font {
    pub fn new(data: Vec<u8>) -> Self {
        Self {
            data: Rc::new(data),
        }
    }

    pub fn face(&self) -> Result<ttf_parser::Face<'_>, ttf_parser::FaceParsingError> {
        ttf_parser::Face::parse(&self.data, 0)
    }
}
