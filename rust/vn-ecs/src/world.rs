use crate::entity::{Entity, EntityManager};
use crate::index::{Index, IndexBuilder};
use crate::storage::{ComponentStorage, SparseSet};
use std::any::{Any, TypeId};
use std::collections::HashMap;

pub struct World {
    entities: EntityManager,
    components: HashMap<TypeId, Box<dyn ComponentStorage>>,
    resources: HashMap<TypeId, Box<dyn Any>>,
    named_resources: HashMap<(String, TypeId), Box<dyn Any>>,
    indices: HashMap<(TypeId, TypeId), Box<dyn Index>>, // (ComponentType, IndexType)
    component_tags: HashMap<(u32, TypeId), Vec<TypeId>>,
}

impl World {
    pub fn new() -> Self {
        Self {
            entities: EntityManager::new(),
            components: HashMap::new(),
            resources: HashMap::new(),
            named_resources: HashMap::new(),
            indices: HashMap::new(),
            component_tags: HashMap::new(),
        }
    }

    pub fn register_storage<T: Any>(&mut self, storage: Box<dyn ComponentStorage>) -> Result<(), String> {
        let type_id = TypeId::of::<T>();
        if self.components.contains_key(&type_id) {
            return Err(format!("Storage for type {:?} already registered", type_id));
        }
        self.components.insert(type_id, storage);
        Ok(())
    }

    pub fn spawn(&mut self) -> Entity {
        self.entities.spawn()
    }

    pub fn despawn(&mut self, entity: Entity) {
        if self.entities.despawn(entity) {
            for storage in self.components.values_mut() {
                storage.remove(entity.id);
            }
            for index in self.indices.values_mut() {
                index.remove(entity);
            }
            self.component_tags.retain(|(e_id, _), _| *e_id != entity.id);
        }
    }

    pub fn add_index<C: Any, I: Index, B: IndexBuilder<I>>(&mut self, builder: B) {
        let type_id = TypeId::of::<C>();
        let index = if let Some(storage) = self.components.get(&type_id) {
            let mut data = Vec::with_capacity(storage.entities().len());
            for &id in storage.entities() {
                let entity = Entity {
                    id,
                    generation: self.entities.generations[id as usize],
                };
                if let Some(component) = storage.get_any(id) {
                    data.push((entity, component));
                }
            }
            builder.build_with_data(&data)
        } else {
            builder.build()
        };

        self.indices
            .insert((type_id, TypeId::of::<I>()), Box::new(index));
    }

    pub fn get_index<C: Any, I: Index>(&self) -> Option<&I> {
        self.indices
            .get(&(TypeId::of::<C>(), TypeId::of::<I>()))?
            .as_any()
            .downcast_ref::<I>()
    }

    pub fn add_component<T: Any>(&mut self, entity: Entity, component: T) {
        if !self.entities.is_alive(entity) {
            return;
        }

        // Update indices
        for ((c_type, _), index) in self.indices.iter_mut() {
            if *c_type == TypeId::of::<T>() {
                index.update(entity, &component);
            }
        }

        let type_id = TypeId::of::<T>();
        let storage = self
            .components
            .entry(type_id)
            .or_insert_with(|| Box::new(SparseSet::<T>::new()));
        storage.insert_any(entity.id, Box::new(component));
    }

    pub fn get_component<T: Any>(&self, entity: Entity) -> Option<&T> {
        if !self.entities.is_alive(entity) {
            return None;
        }

        let type_id = TypeId::of::<T>();
        let storage = self.components.get(&type_id)?;
        storage.get_any(entity.id)?.downcast_ref::<T>()
    }

    pub fn remove_component<T: Any>(&mut self, entity: Entity) -> Option<T> {
        if !self.entities.is_alive(entity) {
            return None;
        }

        let type_id = TypeId::of::<T>();
        
        // Remove from indices
        for ((c_type, _), index) in self.indices.iter_mut() {
            if *c_type == type_id {
                index.remove(entity);
            }
        }

        // Remove tags
        self.component_tags.remove(&(entity.id, type_id));

        // Remove from storage
        let storage = self.components.get_mut(&type_id)?;
        storage.remove_any(entity.id)?.downcast::<T>().ok().map(|b| *b)
    }

    pub fn insert_resource<T: Any>(&mut self, resource: T) {
        self.resources.insert(TypeId::of::<T>(), Box::new(resource));
    }

    pub fn get_resource<T: Any>(&self) -> Option<&T> {
        self.resources.get(&TypeId::of::<T>())?.downcast_ref::<T>()
    }

    pub fn remove_resource<T: Any>(&mut self) -> Option<T> {
        self.resources
            .remove(&TypeId::of::<T>())?
            .downcast::<T>()
            .ok()
            .map(|b| *b)
    }

    pub fn insert_named_resource<T: Any>(&mut self, name: &str, resource: T) {
        self.named_resources
            .insert((name.to_string(), TypeId::of::<T>()), Box::new(resource));
    }

    pub fn get_named_resource<T: Any>(&self, name: &str) -> Option<&T> {
        self.named_resources
            .get(&(name.to_string(), TypeId::of::<T>()))?
            .downcast_ref::<T>()
    }

    pub fn remove_named_resource<T: Any>(&mut self, name: &str) -> Option<T> {
        self.named_resources
            .remove(&(name.to_string(), TypeId::of::<T>()))?
            .downcast::<T>()
            .ok()
            .map(|b| *b)
    }

    pub fn query_entities_with<T: Any>(&self) -> Vec<Entity> {
        let type_id = TypeId::of::<T>();
        if let Some(storage) = self.components.get(&type_id) {
            storage
                .entities()
                .iter()
                .map(|&id| Entity {
                    id,
                    generation: self.entities.generations[id as usize],
                })
                .collect()
        } else {
            Vec::new()
        }
    }

    pub fn query_entities_with_all(&self, types: &[TypeId]) -> Vec<Entity> {
        if types.is_empty() {
            return Vec::new();
        }

        let mut storages = Vec::new();
        for &type_id in types {
            if let Some(storage) = self.components.get(&type_id) {
                storages.push(storage.as_ref());
            } else {
                return Vec::new();
            }
        }

        storages.sort_by_key(|s| s.entities().len());
        let smallest = storages[0];
        let others = &storages[1..];

        smallest
            .entities()
            .iter()
            .filter(|&&id| others.iter().all(|s| s.contains(id)))
            .map(|&id| Entity {
                id,
                generation: self.entities.generations[id as usize],
            })
            .collect()
    }

    pub fn get_entity_components(&self, entity: Entity) -> Vec<TypeId> {
        if !self.entities.is_alive(entity) {
            return Vec::new();
        }

        self.components
            .iter()
            .filter(|(_, storage)| storage.contains(entity.id))
            .map(|(&type_id, _)| type_id)
            .collect()
    }

    pub fn tag_component<T: Any, TAG: Any>(&mut self, entity: Entity) {
        if !self.entities.is_alive(entity) {
            return;
        }
        let type_id = TypeId::of::<T>();
        let tag_id = TypeId::of::<TAG>();
        let tags = self.component_tags
            .entry((entity.id, type_id))
            .or_default();
        if !tags.contains(&tag_id) {
            tags.push(tag_id);
        }
    }

    pub fn untag_component<T: Any, TAG: Any>(&mut self, entity: Entity) {
        if !self.entities.is_alive(entity) {
            return;
        }
        let type_id = TypeId::of::<T>();
        let tag_id = TypeId::of::<TAG>();
        if let Some(tags) = self.component_tags.get_mut(&(entity.id, type_id)) {
            tags.retain(|&t| t != tag_id);
            if tags.is_empty() {
                self.component_tags.remove(&(entity.id, type_id));
            }
        }
    }

    pub fn get_component_tags<T: Any>(&self, entity: Entity) -> Vec<TypeId> {
        self.component_tags
            .get(&(entity.id, TypeId::of::<T>()))
            .cloned()
            .unwrap_or_default()
    }

    pub fn has_tag<T: Any, TAG: Any>(&self, entity: Entity) -> bool {
        if let Some(tags) = self.component_tags.get(&(entity.id, TypeId::of::<T>())) {
            tags.contains(&TypeId::of::<TAG>())
        } else {
            false
        }
    }
}
