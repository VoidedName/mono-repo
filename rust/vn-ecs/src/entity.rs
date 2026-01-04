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
    free_indices: Vec<u32>,
}

impl EntityManager {
    pub fn new() -> Self {
        Self {
            generations: Vec::new(),
            free_indices: Vec::new(),
        }
    }

    pub fn spawn(&mut self) -> Entity {
        if let Some(id) = self.free_indices.pop() {
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
        if index < self.generations.len() && self.generations[index] == entity.generation {
            self.generations[index] += 1;
            self.free_indices.push(entity.id);
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
