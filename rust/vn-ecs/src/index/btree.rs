use crate::collections::btree::BTree;
use crate::entity::Entity;
use crate::index::{Index, IndexBuilder};
use std::any::Any;
use std::collections::HashMap;

pub struct BTreeIndex<T, V, const ORDER: usize = 8> {
    map: BTree<V, Vec<Entity>, ORDER>,
    extractor: fn(&T) -> V,
    entity_values: HashMap<Entity, V>,
}

pub struct BTreeIndexBuilder<T, V, const ORDER: usize = 8> {
    extractor: fn(&T) -> V,
    _phantom: std::marker::PhantomData<T>,
}

impl<T: Any, V: Ord + Clone + Any, const ORDER: usize> BTreeIndexBuilder<T, V, ORDER> {
    pub fn new(extractor: fn(&T) -> V) -> Self {
        Self {
            extractor,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<T: Any, V: Ord + Clone + Any, const ORDER: usize> IndexBuilder<BTreeIndex<T, V, ORDER>>
    for BTreeIndexBuilder<T, V, ORDER>
{
    fn build(self) -> BTreeIndex<T, V, ORDER> {
        BTreeIndex::new(self.extractor)
    }

    fn build_with_data(self, data: &[(Entity, &dyn Any)]) -> BTreeIndex<T, V, ORDER> {
        BTreeIndex::new_with_data(self.extractor, data)
    }
}

impl<T: Any, V: Ord + Clone + Any, const ORDER: usize> BTreeIndex<T, V, ORDER> {
    pub fn new(extractor: fn(&T) -> V) -> Self {
        Self {
            map: BTree::new(),
            extractor,
            entity_values: HashMap::new(),
        }
    }

    pub fn new_with_data(extractor: fn(&T) -> V, data: &[(Entity, &dyn Any)]) -> Self {
        let mut index = Self::new(extractor);
        index.update_many(data);
        index
    }

    pub fn query_range(&self, range: std::ops::RangeInclusive<V>) -> Vec<Entity> {
        self.map
            .range(range)
            .into_iter()
            .flat_map(|(_, entities)| entities.into_iter())
            .collect()
    }
}

impl<T: Any, V: Ord + Clone + Any, const ORDER: usize> Index for BTreeIndex<T, V, ORDER> {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
    fn update(&mut self, entity: Entity, component: &dyn Any) {
        if let Some(c) = component.downcast_ref::<T>() {
            let val = (self.extractor)(c);

            if let Some(old_val) = self.entity_values.get(&entity) {
                if *old_val == val {
                    return;
                }
                // Remove from old position
                let mut should_remove_key = false;
                if let Some(entities) = self.map.get_mut(old_val) {
                    entities.retain(|&e| e != entity);
                    if entities.is_empty() {
                        should_remove_key = true;
                    }
                }
                if should_remove_key {
                    self.map.remove(old_val);
                }
            }

            self.entity_values.insert(entity, val.clone());
            if let Some(entities) = self.map.get_mut(&val) {
                if !entities.contains(&entity) {
                    entities.push(entity);
                }
            } else {
                self.map.insert(val, vec![entity]);
            }
        }
    }
    fn remove(&mut self, entity: Entity) {
        if let Some(val) = self.entity_values.remove(&entity) {
            let mut should_remove_key = false;
            if let Some(entities) = self.map.get_mut(&val) {
                entities.retain(|&e| e != entity);
                if entities.is_empty() {
                    should_remove_key = true;
                }
            }
            if should_remove_key {
                self.map.remove(&val);
            }
        }
    }
}
