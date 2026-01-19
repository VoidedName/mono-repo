use crate::collections::rtree::{RTreeNode, RTreeNum, Rect};
use crate::entity::Entity;
use crate::index::{Index, IndexBuilder};
use std::any::Any;
use std::collections::HashMap;

// Remark (generalization): We could further generalize this, but not really worth it atm.
pub struct RTreeIndex<T, K, const DIMENSIONS: usize> {
    root: Option<RTreeNode<K, DIMENSIONS>>,
    extractor: fn(&T) -> [K; DIMENSIONS],
    max_children: usize,
    entity_positions: HashMap<Entity, [K; DIMENSIONS]>,
}

pub struct RTreeIndexBuilder<T, K, const DIMENSIONS: usize> {
    extractor: fn(&T) -> [K; DIMENSIONS],
    _phantom: std::marker::PhantomData<T>,
}

impl<T: Any, K: RTreeNum + Any, const DIMENSIONS: usize> RTreeIndexBuilder<T, K, DIMENSIONS> {
    pub fn new(extractor: fn(&T) -> [K; DIMENSIONS]) -> Self {
        Self {
            extractor,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<T: Any, K: RTreeNum + Any, const DIMENSIONS: usize> IndexBuilder<RTreeIndex<T, K, DIMENSIONS>>
    for RTreeIndexBuilder<T, K, DIMENSIONS>
{
    fn build(self) -> RTreeIndex<T, K, DIMENSIONS> {
        RTreeIndex::new(self.extractor)
    }

    fn build_with_data(self, data: &[(Entity, &dyn Any)]) -> RTreeIndex<T, K, DIMENSIONS> {
        RTreeIndex::new_with_data(self.extractor, data)
    }
}

impl<T: Any, K: RTreeNum + Any, const DIMENSIONS: usize> RTreeIndex<T, K, DIMENSIONS> {
    pub fn new(extractor: fn(&T) -> [K; DIMENSIONS]) -> Self {
        Self {
            root: None,
            extractor,
            max_children: 8,
            entity_positions: HashMap::new(),
        }
    }

    pub fn new_with_data(
        extractor: fn(&T) -> [K; DIMENSIONS],
        data: &[(Entity, &dyn Any)],
    ) -> Self {
        let mut index = Self::new(extractor);
        index.update_many(data);
        index
    }

    pub fn query_bounds(&self, min: [K; DIMENSIONS], max: [K; DIMENSIONS]) -> Vec<Entity> {
        let mut results = Vec::new();
        let query_rect = Rect { min, max };
        if let Some(root) = &self.root {
            root.query(&query_rect, &mut results);
        }
        results
    }

    fn insert_into_node(
        node: &mut RTreeNode<K, DIMENSIONS>,
        pos: [K; DIMENSIONS],
        entity: Entity,
        max_children: usize,
    ) -> Option<RTreeNode<K, DIMENSIONS>> {
        match node {
            RTreeNode::Leaf { mbr, entries } => {
                entries.push((pos, entity));
                *mbr = mbr.union(&Rect::from_point(pos));
                if entries.len() > max_children {
                    return Some(Self::split_leaf(node));
                }
                None
            }
            RTreeNode::Internal { mbr, children } => {
                // Choose subtree
                let mut best_idx = 0;
                let mut min_enlargement = K::max_value();
                let point_rect = Rect::from_point(pos);

                for (i, child) in children.iter().enumerate() {
                    let enlargement = child.mbr().enlarged_area(&point_rect) - child.mbr().area();
                    if enlargement < min_enlargement {
                        min_enlargement = enlargement;
                        best_idx = i;
                    } else if enlargement == min_enlargement {
                        if child.mbr().area() < children[best_idx].mbr().area() {
                            best_idx = i;
                        }
                    }
                }

                let split_node =
                    Self::insert_into_node(&mut children[best_idx], pos, entity, max_children);
                *mbr = mbr.union(&children[best_idx].mbr());

                if let Some(new_child) = split_node {
                    children.push(new_child);
                    *mbr = mbr.union(&children.last().unwrap().mbr());
                    if children.len() > max_children {
                        return Some(Self::split_internal(node));
                    }
                }
                None
            }
        }
    }

    fn split_leaf(node: &mut RTreeNode<K, DIMENSIONS>) -> RTreeNode<K, DIMENSIONS> {
        if let RTreeNode::Leaf { entries, .. } = node {
            let rects: Vec<Rect<K, DIMENSIONS>> =
                entries.iter().map(|e| Rect::from_point(e.0)).collect();
            let (idx1, idx2) = Self::pick_seeds(&rects);
            let entry1 = entries.remove(idx1.max(idx2));
            let entry2 = entries.remove(idx1.min(idx2));

            let mut entries1 = vec![entry1];
            let mut entries2 = vec![entry2];
            let mut mbr1 = Rect::from_point(entry1.0);
            let mut mbr2 = Rect::from_point(entry2.0);

            let old_entries = std::mem::take(entries);
            for entry in old_entries {
                let rect = Rect::from_point(entry.0);
                let e1 = mbr1.enlarged_area(&rect) - mbr1.area();
                let e2 = mbr2.enlarged_area(&rect) - mbr2.area();

                if e1 < e2 {
                    entries1.push(entry);
                    mbr1 = mbr1.union(&rect);
                } else if e2 < e1 {
                    entries2.push(entry);
                    mbr2 = mbr2.union(&rect);
                } else {
                    if mbr1.area() < mbr2.area() {
                        entries1.push(entry);
                        mbr1 = mbr1.union(&rect);
                    } else {
                        entries2.push(entry);
                        mbr2 = mbr2.union(&rect);
                    }
                }
            }

            *node = RTreeNode::Leaf {
                mbr: mbr1,
                entries: entries1,
            };
            RTreeNode::Leaf {
                mbr: mbr2,
                entries: entries2,
            }
        } else {
            panic!("Expected leaf node")
        }
    }

    fn split_internal(node: &mut RTreeNode<K, DIMENSIONS>) -> RTreeNode<K, DIMENSIONS> {
        if let RTreeNode::Internal { children, .. } = node {
            let rects: Vec<Rect<K, DIMENSIONS>> = children.iter().map(|c| c.mbr()).collect();
            let (idx1, idx2) = Self::pick_seeds(&rects);
            let child1 = children.remove(idx1.max(idx2));
            let child2 = children.remove(idx1.min(idx2));

            let mut group1 = vec![child1];
            let mut group2 = vec![child2];
            let mut mbr1 = group1[0].mbr();
            let mut mbr2 = group2[0].mbr();

            let old_children = std::mem::take(children);
            for child in old_children {
                let rect = child.mbr();
                let e1 = mbr1.enlarged_area(&rect) - mbr1.area();
                let e2 = mbr2.enlarged_area(&rect) - mbr2.area();

                if e1 < e2 {
                    group1.push(child);
                    mbr1 = mbr1.union(&rect);
                } else {
                    group2.push(child);
                    mbr2 = mbr2.union(&rect);
                }
            }

            *node = RTreeNode::Internal {
                mbr: mbr1,
                children: group1,
            };
            RTreeNode::Internal {
                mbr: mbr2,
                children: group2,
            }
        } else {
            panic!("Expected internal node")
        }
    }

    fn pick_seeds(rects: &[Rect<K, DIMENSIONS>]) -> (usize, usize) {
        let mut best_pair = (0, 1);
        let mut max_waste = K::zero();
        let mut first = true;

        for i in 0..rects.len() {
            for j in (i + 1)..rects.len() {
                let waste = rects[i].enlarged_area(&rects[j]) - rects[i].area() - rects[j].area();
                if first || waste > max_waste {
                    max_waste = waste;
                    best_pair = (i, j);
                    first = false;
                }
            }
        }
        best_pair
    }
}

impl<T: Any, K: RTreeNum + Any, const DIMENSIONS: usize> Index for RTreeIndex<T, K, DIMENSIONS> {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
    fn update(&mut self, entity: Entity, component: &dyn Any) {
        if let Some(c) = component.downcast_ref::<T>() {
            let pos = (self.extractor)(c);
            self.remove(entity);

            let max_children = self.max_children;
            if let Some(ref mut root) = self.root {
                if let Some(new_node) = Self::insert_into_node(root, pos, entity, max_children) {
                    let mut new_root_children = Vec::with_capacity(max_children);
                    let old_root = std::mem::replace(
                        root,
                        RTreeNode::Leaf {
                            mbr: Rect::from_point(pos),
                            entries: Vec::new(),
                        },
                    ); // dummy
                    new_root_children.push(old_root);
                    new_root_children.push(new_node);
                    let new_root = RTreeNode::Internal {
                        mbr: new_root_children[0]
                            .mbr()
                            .union(&new_root_children[1].mbr()),
                        children: new_root_children,
                    };
                    *root = new_root;
                }
            } else {
                self.root = Some(RTreeNode::Leaf {
                    mbr: Rect::from_point(pos),
                    entries: vec![(pos, entity)],
                });
            }
            self.entity_positions.insert(entity, pos);
        }
    }
    fn remove(&mut self, entity: Entity) {
        if let Some(pos) = self.entity_positions.remove(&entity) {
            if let Some(ref mut root) = self.root {
                root.remove(entity, pos);

                // Handle root underflow
                let mut should_collapse = false;
                let mut new_root = None;

                match root {
                    RTreeNode::Leaf { entries, .. } => {
                        if entries.is_empty() {
                            should_collapse = true;
                        }
                    }
                    RTreeNode::Internal { children, .. } => {
                        if children.is_empty() {
                            should_collapse = true;
                        } else if children.len() == 1 {
                            should_collapse = true;
                            new_root = Some(children.remove(0));
                        }
                    }
                }

                if should_collapse {
                    self.root = new_root;
                }
            }
        }
    }
}
