use std::collections::VecDeque;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Entity {
    pub(crate) id: u32,
    pub(crate) generation: u32,
}

impl Entity {
    pub fn id(&self) -> u32 {
        self.id
    }
}

pub struct EntityManager {
    pub(crate) generations: Vec<u32>,
    free_indices: VecDeque<u32>,
}

impl EntityManager {
    pub fn new() -> Self {
        Self {
            generations: Vec::new(),
            free_indices: VecDeque::new(),
        }
    }

    pub fn spawn(&mut self) -> Entity {
        if let Some(id) = self.free_indices.pop_front() {
            Entity {
                id,
                generation: self.generations[id as usize],
            }
        } else {
            let id = self.generations.len() as u32;
            self.generations.push(0);
            Entity { id, generation: 0 }
        }
    }

    pub fn despawn(&mut self, entity: Entity) -> bool {
        let index = entity.id as usize;
        if self.is_alive(entity) {
            self.generations[index] += 1;
            self.free_indices.push_back(entity.id);
            true
        } else {
            false
        }
    }

    pub fn is_alive(&self, entity: Entity) -> bool {
        let index = entity.id as usize;
        index < self.generations.len() && self.generations[index] == entity.generation
    }
}
