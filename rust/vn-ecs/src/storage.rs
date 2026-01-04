use std::any::Any;

pub trait ComponentStorage: Any {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn remove(&mut self, entity_id: u32);
    fn contains(&self, entity_id: u32) -> bool;
    fn entities(&self) -> &[u32];
    fn insert_any(&mut self, entity_id: u32, component: Box<dyn Any>);
    fn get_any(&self, entity_id: u32) -> Option<&dyn Any>;
    fn get_any_mut(&mut self, entity_id: u32) -> Option<&mut dyn Any>;
    fn remove_any(&mut self, entity_id: u32) -> Option<Box<dyn Any>>;
}

pub struct SparseSet<T> {
    pub(crate) sparse: Vec<Option<u32>>,
    pub(crate) dense: Vec<u32>,
    pub(crate) data: Vec<T>,
}

impl<T> SparseSet<T> {
    pub fn new() -> Self {
        Self {
            sparse: Vec::new(),
            dense: Vec::new(),
            data: Vec::new(),
        }
    }

    pub fn insert(&mut self, entity_id: u32, component: T) {
        let index = entity_id as usize;
        if index >= self.sparse.len() {
            self.sparse.resize(index + 1, None);
        }

        if let Some(dense_idx) = self.sparse[index] {
            self.data[dense_idx as usize] = component;
        } else {
            let dense_idx = self.dense.len() as u32;
            self.sparse[index] = Some(dense_idx);
            self.dense.push(entity_id);
            self.data.push(component);
        }
    }

    pub fn get(&self, entity_id: u32) -> Option<&T> {
        let index = entity_id as usize;
        if index < self.sparse.len() {
            self.sparse[index].map(|dense_idx| &self.data[dense_idx as usize])
        } else {
            None
        }
    }

    pub fn get_mut(&mut self, entity_id: u32) -> Option<&mut T> {
        let index = entity_id as usize;
        if index < self.sparse.len() {
            let dense_idx = self.sparse[index]?;
            Some(&mut self.data[dense_idx as usize])
        } else {
            None
        }
    }

    pub fn remove(&mut self, entity_id: u32) -> Option<T> {
        let index = entity_id as usize;
        if index < self.sparse.len() {
            if let Some(dense_idx) = self.sparse[index] {
                let last_entity_id = *self.dense.last().unwrap();
                let last_idx = self.dense.len() - 1;

                self.dense.swap(dense_idx as usize, last_idx);
                self.data.swap(dense_idx as usize, last_idx);

                self.sparse[last_entity_id as usize] = Some(dense_idx);
                self.sparse[index] = None;

                self.dense.pop();
                return self.data.pop();
            }
        }
        None
    }

    pub fn contains(&self, entity_id: u32) -> bool {
        let index = entity_id as usize;
        index < self.sparse.len() && self.sparse[index].is_some()
    }
}

impl<T: Any> ComponentStorage for SparseSet<T> {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
    fn remove(&mut self, entity_id: u32) {
        self.remove(entity_id);
    }
    fn contains(&self, entity_id: u32) -> bool {
        self.contains(entity_id)
    }
    fn entities(&self) -> &[u32] {
        &self.dense
    }
    fn insert_any(&mut self, entity_id: u32, component: Box<dyn Any>) {
        if let Ok(component) = component.downcast::<T>() {
            self.insert(entity_id, *component);
        }
    }
    fn get_any(&self, entity_id: u32) -> Option<&dyn Any> {
        self.get(entity_id).map(|c| c as &dyn Any)
    }
    fn get_any_mut(&mut self, entity_id: u32) -> Option<&mut dyn Any> {
        self.get_mut(entity_id).map(|c| c as &mut dyn Any)
    }
    fn remove_any(&mut self, entity_id: u32) -> Option<Box<dyn Any>> {
        self.remove(entity_id).map(|c| Box::new(c) as Box<dyn Any>)
    }
}
