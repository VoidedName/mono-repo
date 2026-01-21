use crate::ElementId;

pub struct ElementWorld {
    next_id: u32,
}

impl ElementWorld {
    pub fn new() -> Self {
        Self { next_id: 0 }
    }

    pub fn next_id(&mut self) -> ElementId {
        let id = ElementId(self.next_id);
        self.next_id += 1;
        id
    }
}
