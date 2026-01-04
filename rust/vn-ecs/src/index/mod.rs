use crate::entity::Entity;
use std::any::Any;

pub trait Index: Any {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn update(&mut self, entity: Entity, component: &dyn Any);
    fn remove(&mut self, entity: Entity);
    fn update_many(&mut self, data: &[(Entity, &dyn Any)]) {
        for (entity, component) in data {
            self.update(*entity, *component);
        }
    }
}

pub trait IndexBuilder<I: Index> {
    fn build(self) -> I;
    fn build_with_data(self, data: &[(Entity, &dyn Any)]) -> I;
}

pub mod btree;
pub mod rtree;

pub use btree::{BTreeIndex, BTreeIndexBuilder};
pub use rtree::{RTreeIndex, RTreeIndexBuilder};
