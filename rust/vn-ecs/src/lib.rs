pub mod collections;
pub mod entity;
pub mod index;
pub mod storage;
pub mod system;
pub mod world;

pub use entity::{Entity, EntityManager};
pub use index::{BTreeIndex, BTreeIndexBuilder, Index, RTreeIndex, RTreeIndexBuilder};
pub use storage::{ComponentStorage, SparseSet};
pub use system::{System, SystemManager};
pub use world::World;

#[cfg(test)]
mod tests {
    use super::*;
    use std::any::TypeId;

    #[test]
    fn test_spawn_despawn() {
        let mut world = World::new();
        let e1 = world.spawn();
        let e2 = world.spawn();
        assert_ne!(e1, e2);

        world.despawn(e1);
        let e3 = world.spawn();
        assert_eq!(e1.id, e3.id);
        assert_ne!(e1.generation, e3.generation);
    }

    #[test]
    fn test_components() {
        let mut world = World::new();
        let e = world.spawn();
        world.add_component(e, 42i32);
        assert_eq!(world.get_component::<i32>(e), Some(&42));

        world.despawn(e);
        assert_eq!(world.get_component::<i32>(e), None);
    }

    #[test]
    fn test_indexing() {
        let mut world = World::new();
        #[derive(Debug, Clone, Copy)]
        struct Pos {
            x: i32,
        }
        world.add_index::<Pos, BTreeIndex<Pos, i32, 8>, _>(BTreeIndexBuilder::new(|p: &Pos| p.x));

        let e1 = world.spawn();
        world.add_component(e1, Pos { x: 10 });
        let e2 = world.spawn();
        world.add_component(e2, Pos { x: 20 });

        {
            let index = world.get_index::<Pos, BTreeIndex<Pos, i32, 8>>().unwrap();
            let results = index.query_range(5..=15);
            assert_eq!(results, vec![e1]);
        }

        // Test with more elements to trigger splits
        for i in 0..20 {
            let e = world.spawn();
            world.add_component(e, Pos { x: i });
        }

        {
            let index = world.get_index::<Pos, BTreeIndex<Pos, i32, 8>>().unwrap();
            let results = index.query_range(0..=5);
            // x=0, 1, 2, 3, 4, 5 from the loop.
            assert_eq!(results.len(), 6);
        }

        // Test removal
        world.despawn(e1);
        {
            let index = world.get_index::<Pos, BTreeIndex<Pos, i32, 8>>().unwrap();
            let results = index.query_range(5..=15);
            // x=5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15 from the loop (11 values)
            // e1 (x=10) was also there but is now despawned.
            assert_eq!(results.len(), 11);
            assert!(!results.contains(&e1));
        }
    }

    #[test]
    fn test_rtree_indexing() {
        let mut world = World::new();
        struct SpatialPos {
            p: [f32; 2],
        }
        world.add_index::<SpatialPos, RTreeIndex<SpatialPos, f32, 2>, _>(
            RTreeIndexBuilder::new(|s: &SpatialPos| s.p),
        );

        let e1 = world.spawn();
        world.add_component(e1, SpatialPos { p: [1.0, 1.0] });
        let e2 = world.spawn();
        world.add_component(e2, SpatialPos { p: [10.0, 10.0] });
        let e3 = world.spawn();
        world.add_component(e3, SpatialPos { p: [5.0, 5.0] });

        {
            let index = world
                .get_index::<SpatialPos, RTreeIndex<SpatialPos, f32, 2>>()
                .unwrap();

            // Query box [0,0] to [6,6]
            let results = index.query_bounds([0.0, 0.0], [6.0, 6.0]);
            assert_eq!(results.len(), 2);
            assert!(results.contains(&e1));
            assert!(results.contains(&e3));
            assert!(!results.contains(&e2));
        }

        // Test update
        world.add_component(e1, SpatialPos { p: [100.0, 100.0] });
        {
            let index = world
                .get_index::<SpatialPos, RTreeIndex<SpatialPos, f32, 2>>()
                .unwrap();
            let results_after = index.query_bounds([0.0, 0.0], [6.0, 6.0]);
            assert_eq!(results_after.len(), 1);
            assert!(results_after.contains(&e3));
        }

        // Test remove
        world.despawn(e3);
        {
            let index = world
                .get_index::<SpatialPos, RTreeIndex<SpatialPos, f32, 2>>()
                .unwrap();
            let results_final = index.query_bounds([0.0, 0.0], [6.0, 6.0]);
            assert!(results_final.is_empty());
        }
    }

    #[test]
    fn test_rtree_i32() {
        let mut world = World::new();
        struct GridPos {
            p: [i32; 2],
        }
        world.add_index::<GridPos, RTreeIndex<GridPos, i32, 2>, _>(
            RTreeIndexBuilder::new(|s: &GridPos| s.p),
        );

        let e1 = world.spawn();
        world.add_component(e1, GridPos { p: [1, 1] });
        let e2 = world.spawn();
        world.add_component(e2, GridPos { p: [10, 10] });

        let index = world.get_index::<GridPos, RTreeIndex<GridPos, i32, 2>>().unwrap();
        let results = index.query_bounds([0, 0], [5, 5]);
        assert_eq!(results, vec![e1]);
    }

    #[test]
    fn test_rtree_root_underflow() {
        let mut world = World::new();
        struct Pos {
            p: [f32; 2],
        }
        // Max children is 8 by default in RTreeIndex
        world.add_index::<Pos, RTreeIndex<Pos, f32, 2>, _>(
            RTreeIndexBuilder::new(|s: &Pos| s.p),
        );

        let mut entities = Vec::new();
        for i in 0..10 {
            let e = world.spawn();
            world.add_component(e, Pos { p: [i as f32, i as f32] });
            entities.push(e);
        }

        // With 10 entities and max_children=8, we should have an internal root.
        // Let's verify results before removal
        {
            let index = world.get_index::<Pos, RTreeIndex<Pos, f32, 2>>().unwrap();
            let results = index.query_bounds([0.0, 0.0], [10.0, 10.0]);
            assert_eq!(results.len(), 10);
        }

        // Remove 5 entities, might still be internal depending on distribution,
        // but let's remove almost all to be sure it collapses.
        for e in entities.iter().take(9) {
            world.despawn(*e);
        }

        {
            let index = world.get_index::<Pos, RTreeIndex<Pos, f32, 2>>().unwrap();
            let results = index.query_bounds([0.0, 0.0], [10.0, 10.0]);
            assert_eq!(results.len(), 1);
            assert_eq!(results[0], entities[9]);
        }

        // Remove the last one
        world.despawn(entities[9]);
        {
            let index = world.get_index::<Pos, RTreeIndex<Pos, f32, 2>>().unwrap();
            let results = index.query_bounds([0.0, 0.0], [10.0, 10.0]);
            assert!(results.is_empty());
        }
    }

    #[test]
    fn test_tagging() {
        let mut world = World::new();
        let e = world.spawn();
        world.add_component(e, 42i32);
        
        struct Important;
        struct Temp;
        
        world.tag_component::<i32, Important>(e);
        world.tag_component::<i32, Temp>(e);

        assert_eq!(world.get_component_tags::<i32>(e), vec![TypeId::of::<Important>(), TypeId::of::<Temp>()]);

        world.untag_component::<i32, Temp>(e);
        assert_eq!(world.get_component_tags::<i32>(e), vec![TypeId::of::<Important>()]);

        world.untag_component::<i32, Important>(e);
        assert!(world.get_component_tags::<i32>(e).is_empty());
    }

    #[test]
    fn test_despawn_cleanup() {
        let mut world = World::new();
        let e = world.spawn();
        world.add_component(e, 42i32);
        
        struct Important;
        world.tag_component::<i32, Important>(e);
        
        #[derive(Debug, Clone, Copy)]
        struct Pos { x: i32 }
        world.add_index::<Pos, BTreeIndex<Pos, i32>, _>(BTreeIndexBuilder::new(|p: &Pos| p.x));
        world.add_component(e, Pos { x: 1 });

        // Verify it's there
        assert!(world.get_component::<i32>(e).is_some());
        assert!(!world.get_component_tags::<i32>(e).is_empty());
        assert!(!world.get_index::<Pos, BTreeIndex<Pos, i32>>().unwrap().query_range(0..=2).is_empty());

        world.despawn(e);

        // Verify it's gone from components
        assert!(world.get_component::<i32>(e).is_none());
        // Verify tags are gone
        assert!(world.get_component_tags::<i32>(e).is_empty());
        // Verify index is updated
        assert!(world.get_index::<Pos, BTreeIndex<Pos, i32>>().unwrap().query_range(0..=2).is_empty());
    }

    #[test]
    fn test_query_all() {
        let mut world = World::new();
        let e1 = world.spawn();
        world.add_component(e1, 10i32);
        world.add_component(e1, true);

        let e2 = world.spawn();
        world.add_component(e2, 20i32);

        let results = world.query_entities_with_all(&[TypeId::of::<i32>(), TypeId::of::<bool>()]);
        assert_eq!(results, vec![e1]);
    }

    #[test]
    fn test_removal() {
        let mut world = World::new();
        let e = world.spawn();
        world.add_component(e, 42i32);
        struct MyTag;
        world.tag_component::<i32, MyTag>(e);
        
        #[derive(Debug, Clone, Copy, PartialEq)]
        struct Pos { x: i32 }
        world.add_index::<Pos, BTreeIndex<Pos, i32>, _>(BTreeIndexBuilder::new(|p: &Pos| p.x));
        world.add_component(e, Pos { x: 10 });

        // Remove component
        let val = world.remove_component::<i32>(e);
        assert_eq!(val, Some(42));
        assert_eq!(world.get_component::<i32>(e), None);
        assert!(world.get_component_tags::<i32>(e).is_empty());

        // Verify index update via component removal
        {
            let index = world.get_index::<Pos, BTreeIndex<Pos, i32>>().unwrap();
            assert!(!index.query_range(9..=11).is_empty());
        }
        world.remove_component::<Pos>(e);
        {
            let index = world.get_index::<Pos, BTreeIndex<Pos, i32>>().unwrap();
            assert!(index.query_range(9..=11).is_empty());
        }

        // Resources
        world.insert_resource(100u32);
        assert_eq!(world.remove_resource::<u32>(), Some(100));
        assert_eq!(world.get_resource::<u32>(), None);

        // Named resources
        world.insert_named_resource("res", 200u32);
        assert_eq!(world.remove_named_resource::<u32>("res"), Some(200));
        assert_eq!(world.get_named_resource::<u32>("res"), None);
    }

    #[test]
    fn test_register_storage() {
        let mut world = World::new();
        world.register_storage::<i32>(Box::new(SparseSet::<i32>::new())).unwrap();
        
        // Already registered
        assert!(world.register_storage::<i32>(Box::new(SparseSet::<i32>::new())).is_err());
        
        let e = world.spawn();
        world.add_component(e, 42i32);
        assert_eq!(world.get_component::<i32>(e), Some(&42));
    }

    #[test]
    fn test_btree_direct() {
        use crate::collections::btree::BTree;
        let mut btree: BTree<i32, i32, 8> = BTree::new();
        for i in 0..100 {
            btree.insert(i, i * 2);
        }
        for i in 0..100 {
            assert_eq!(btree.get(&i), Some(&(i * 2)));
        }
        for i in (0..100).step_by(2) {
            btree.remove(&i);
        }
        for i in 0..100 {
            if i % 2 == 0 {
                assert_eq!(btree.get(&i), None);
            } else {
                assert_eq!(btree.get(&i), Some(&(i * 2)));
            }
        }
    }
    #[test]
    fn test_btree_custom_order() {
        use crate::collections::btree::BTree;
        // Test with a very small order to force frequent splits/merges
        let mut btree: BTree<i32, i32, 4> = BTree::new();
        for i in 0..50 {
            btree.insert(i, i);
        }
        for i in 0..50 {
            assert_eq!(btree.get(&i), Some(&i));
        }
        for i in 0..50 {
            btree.remove(&i);
            assert_eq!(btree.get(&i), None);
        }
    }
    #[test]
    fn test_index_population_on_add() {
        let mut world = World::new();
        let e1 = world.spawn();
        world.add_component(e1, 10i32);
        let e2 = world.spawn();
        world.add_component(e2, 20i32);

        #[derive(Debug, Clone, Copy)]
        struct Pos {
            x: i32,
        }
        world.add_component(e1, Pos { x: 10 });
        world.add_component(e2, Pos { x: 20 });

        // Add index AFTER components are already there
        world.add_index::<Pos, BTreeIndex<Pos, i32>, _>(BTreeIndexBuilder::new(|p: &Pos| p.x));

        let index = world.get_index::<Pos, BTreeIndex<Pos, i32>>().unwrap();
        let results = index.query_range(5..=15);
        assert_eq!(results, vec![e1]);
        
        let results_all = index.query_range(0..=100);
        assert_eq!(results_all.len(), 2);
        assert!(results_all.contains(&e1));
        assert!(results_all.contains(&e2));
    }

    #[test]
    fn test_systems() {
        let mut world = World::new();
        let mut sm = SystemManager::new();

        #[derive(Copy, Clone, Debug)]
        struct Velocity(f32);
        #[derive(Copy, Clone, Debug)]
        struct Position(f32);

        struct MovementSystem;
        impl System for MovementSystem {
            fn run(&mut self, world: &mut World) {
                let entities = world.query_entities_with_all(&[
                    TypeId::of::<Position>(),
                    TypeId::of::<Velocity>(),
                ]);

                for entity in entities {
                    let vel = *world.get_component::<Velocity>(entity).unwrap();
                    let pos = world.get_component::<Position>(entity).unwrap();
                    let new_pos = Position(pos.0 + vel.0);
                    world.add_component(entity, new_pos);
                }
            }
        }

        sm.add_system(MovementSystem);

        let e = world.spawn();
        world.add_component(e, Position(0.0));
        world.add_component(e, Velocity(1.0));

        sm.run(&mut world);

        assert_eq!(world.get_component::<Position>(e).unwrap().0, 1.0);

        sm.run(&mut world);

        assert_eq!(world.get_component::<Position>(e).unwrap().0, 2.0);
    }

    #[test]
    fn test_system_management() {
        let mut world = World::new();
        let mut sm = SystemManager::new();

        #[derive(Copy, Clone, Debug)]
        struct Counter(i32);

        struct IncrementSystem;
        impl System for IncrementSystem {
            fn run(&mut self, world: &mut World) {
                let entities = world.query_entities_with_all(&[TypeId::of::<Counter>()]);
                for entity in entities {
                    let c = world.get_component::<Counter>(entity).unwrap();
                    let new_c = Counter(c.0 + 1);
                    world.add_component(entity, new_c);
                }
            }
        }

        sm.add_system(IncrementSystem);
        let e = world.spawn();
        world.add_component(e, Counter(0));

        sm.run(&mut world);
        assert_eq!(world.get_component::<Counter>(e).unwrap().0, 1);

        // Disable
        let name = std::any::type_name::<IncrementSystem>();
        sm.set_enabled_by_name(name, false);
        sm.run(&mut world);
        assert_eq!(world.get_component::<Counter>(e).unwrap().0, 1);

        // Enable
        sm.set_enabled_by_name(name, true);
        sm.run(&mut world);
        assert_eq!(world.get_component::<Counter>(e).unwrap().0, 2);

        // Remove
        sm.remove_system_by_name(name);
        sm.run(&mut world);
        assert_eq!(world.get_component::<Counter>(e).unwrap().0, 2);
    }
}
