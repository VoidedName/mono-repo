use crate::entity::Entity;
use std::ops::{Add, Mul, Sub};

#[derive(Clone, Copy, Debug)]
pub struct Rect<K, const N: usize> {
    pub min: [K; N],
    pub max: [K; N],
}

pub trait RTreeNum:
    Copy + PartialOrd + Sub<Output = Self> + Mul<Output = Self> + Add<Output = Self> + Default
{
    fn zero() -> Self;
    fn one() -> Self;
    fn max_value() -> Self;
    fn abs_diff(self, other: Self) -> Self;
}

impl RTreeNum for f32 {
    fn zero() -> Self {
        0.0
    }
    fn one() -> Self {
        1.0
    }
    fn max_value() -> Self {
        f32::MAX
    }
    fn abs_diff(self, other: Self) -> Self {
        (self - other).abs()
    }
}

impl RTreeNum for f64 {
    fn zero() -> Self {
        0.0
    }
    fn one() -> Self {
        1.0
    }
    fn max_value() -> Self {
        f64::MAX
    }
    fn abs_diff(self, other: Self) -> Self {
        (self - other).abs()
    }
}

impl RTreeNum for i32 {
    fn zero() -> Self {
        0
    }
    fn one() -> Self {
        1
    }
    fn max_value() -> Self {
        i32::MAX
    }
    fn abs_diff(self, other: Self) -> Self {
        (self - other).abs()
    }
}

impl RTreeNum for i64 {
    fn zero() -> Self {
        0
    }
    fn one() -> Self {
        1
    }
    fn max_value() -> Self {
        i64::MAX
    }
    fn abs_diff(self, other: Self) -> Self {
        (self - other).abs()
    }
}

impl<K: RTreeNum, const N: usize> Rect<K, N> {
    pub fn from_point(p: [K; N]) -> Self {
        Self { min: p, max: p }
    }

    pub fn area(&self) -> K {
        let mut a = K::one();
        for i in 0..N {
            a = a * (self.max[i] - self.min[i]);
        }
        a
    }

    pub fn union(&self, other: &Self) -> Self {
        let mut min = [K::zero(); N];
        let mut max = [K::zero(); N];
        for i in 0..N {
            min[i] = if self.min[i] < other.min[i] {
                self.min[i]
            } else {
                other.min[i]
            };
            max[i] = if self.max[i] > other.max[i] {
                self.max[i]
            } else {
                other.max[i]
            };
        }
        Self { min, max }
    }

    pub fn enlarged_area(&self, other: &Self) -> K {
        self.union(other).area()
    }

    pub fn intersects(&self, other: &Self) -> bool {
        for i in 0..N {
            if self.min[i] > other.max[i] || other.min[i] > self.max[i] {
                return false;
            }
        }
        true
    }

    pub fn contains_point(&self, p: [K; N]) -> bool {
        for i in 0..N {
            if p[i] < self.min[i] || p[i] > self.max[i] {
                return false;
            }
        }
        true
    }
}

pub enum RTreeNode<K, const N: usize> {
    Leaf {
        mbr: Rect<K, N>,
        entries: Vec<([K; N], Entity)>,
    },
    Internal {
        mbr: Rect<K, N>,
        children: Vec<RTreeNode<K, N>>,
    },
}

impl<K: RTreeNum, const N: usize> RTreeNode<K, N> {
    pub fn mbr(&self) -> Rect<K, N> {
        match self {
            RTreeNode::Leaf { mbr, .. } => *mbr,
            RTreeNode::Internal { mbr, .. } => *mbr,
        }
    }

    pub fn update_mbr(&mut self) {
        match self {
            RTreeNode::Leaf { mbr, entries } => {
                if let Some((first_pos, _)) = entries.first() {
                    let mut new_mbr = Rect::from_point(*first_pos);
                    for (pos, _) in entries.iter().skip(1) {
                        new_mbr = new_mbr.union(&Rect::from_point(*pos));
                    }
                    *mbr = new_mbr;
                }
            }
            RTreeNode::Internal { mbr, children } => {
                if let Some(first_child) = children.first() {
                    let mut new_mbr = first_child.mbr();
                    for child in children.iter().skip(1) {
                        new_mbr = new_mbr.union(&child.mbr());
                    }
                    *mbr = new_mbr;
                }
            }
        }
    }

    pub fn query(&self, query_rect: &Rect<K, N>, results: &mut Vec<Entity>) {
        if !self.mbr().intersects(query_rect) {
            return;
        }
        match self {
            RTreeNode::Leaf { entries, .. } => {
                for (pos, entity) in entries {
                    if query_rect.contains_point(*pos) {
                        results.push(*entity);
                    }
                }
            }
            RTreeNode::Internal { children, .. } => {
                for child in children {
                    child.query(query_rect, results);
                }
            }
        }
    }

    pub fn remove(&mut self, entity: Entity, pos: [K; N]) -> bool {
        match self {
            RTreeNode::Leaf { entries, .. } => {
                let initial_len = entries.len();
                entries.retain(|(p, e)| *e != entity || !Self::pos_eq(*p, pos));
                if entries.len() != initial_len {
                    self.update_mbr();
                    return true;
                }
                false
            }
            RTreeNode::Internal { children, .. } => {
                let mut removed = false;
                for child in children.iter_mut() {
                    if child.mbr().contains_point(pos) {
                        if child.remove(entity, pos) {
                            removed = true;
                            break;
                        }
                    }
                }
                if removed {
                    self.update_mbr();
                }
                removed
            }
        }
    }

    fn pos_eq(p1: [K; N], p2: [K; N]) -> bool {
        for i in 0..N {
            if p1[i] != p2[i] {
                return false;
            }
        }
        true
    }
}
