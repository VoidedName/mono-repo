use std::ops::RangeInclusive;

struct BTreeNode<K, V, const ORDER: usize> {
    keys: Vec<K>,
    values: Vec<V>,
    children: Vec<BTreeNode<K, V, ORDER>>,
    is_leaf: bool,
}

impl<K: Ord + Clone, V: Clone, const ORDER: usize> BTreeNode<K, V, ORDER> {
    const MIN_KEYS: usize = ORDER / 2 - 1;

    fn new(is_leaf: bool) -> Self {
        Self {
            keys: Vec::with_capacity(ORDER),
            values: Vec::with_capacity(ORDER),
            children: Vec::with_capacity(ORDER + 1),
            is_leaf,
        }
    }

    fn split_child(&mut self, i: usize) {
        let mut y = self.children.remove(i);
        let mut z = BTreeNode::new(y.is_leaf);

        z.keys = y.keys.split_off(Self::MIN_KEYS + 1);
        z.values = y.values.split_off(Self::MIN_KEYS + 1);
        let mid_key = y.keys.pop().unwrap();
        let mid_val = y.values.pop().unwrap();

        if !y.is_leaf {
            z.children = y.children.split_off(Self::MIN_KEYS + 1);
        }

        self.keys.insert(i, mid_key);
        self.values.insert(i, mid_val);
        self.children.insert(i, y);
        self.children.insert(i + 1, z);
    }

    fn insert_non_full(&mut self, key: K, value: V) {
        if self.is_leaf {
            match self.keys.binary_search(&key) {
                Ok(idx) => self.values[idx] = value,
                Err(idx) => {
                    self.keys.insert(idx, key);
                    self.values.insert(idx, value);
                }
            }
        } else {
            match self.keys.binary_search(&key) {
                Ok(idx) => self.values[idx] = value,
                Err(idx) => {
                    if self.children[idx].keys.len() == ORDER - 1 {
                        self.split_child(idx);
                        match self.keys.binary_search(&key) {
                            Ok(new_idx) => self.values[new_idx] = value,
                            Err(new_idx) => self.children[new_idx].insert_non_full(key, value),
                        }
                    } else {
                        self.children[idx].insert_non_full(key, value);
                    }
                }
            }
        }
    }

    fn get(&self, key: &K) -> Option<&V> {
        match self.keys.binary_search(key) {
            Ok(idx) => Some(&self.values[idx]),
            Err(idx) => {
                if self.is_leaf {
                    None
                } else {
                    self.children[idx].get(key)
                }
            }
        }
    }

    fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        match self.keys.binary_search(key) {
            Ok(idx) => Some(&mut self.values[idx]),
            Err(idx) => {
                if self.is_leaf {
                    None
                } else {
                    self.children[idx].get_mut(key)
                }
            }
        }
    }

    fn range(&self, range: &RangeInclusive<K>, results: &mut Vec<(K, V)>) {
        let start_idx = self
            .keys
            .binary_search(range.start())
            .unwrap_or_else(|idx| idx);

        for i in start_idx..self.keys.len() {
            if !self.is_leaf {
                self.children[i].range(range, results);
            }
            if range.contains(&self.keys[i]) {
                results.push((self.keys[i].clone(), self.values[i].clone()));
            } else if &self.keys[i] > range.end() {
                return;
            }
        }

        if !self.is_leaf {
            self.children[self.keys.len()].range(range, results);
        }
    }

    fn remove(&mut self, key: &K) -> Option<V> {
        let idx = match self.keys.binary_search(key) {
            Ok(idx) => {
                if self.is_leaf {
                    self.keys.remove(idx);
                    return Some(self.values.remove(idx));
                }

                // Internal node
                if self.children[idx].keys.len() > Self::MIN_KEYS {
                    // Predecessor
                    let (k, v) = self.children[idx].pop_last();
                    let old_val = std::mem::replace(&mut self.values[idx], v);
                    self.keys[idx] = k;
                    return Some(old_val);
                } else if self.children[idx + 1].keys.len() > Self::MIN_KEYS {
                    // Successor
                    let (k, v) = self.children[idx + 1].pop_first();
                    let old_val = std::mem::replace(&mut self.values[idx], v);
                    self.keys[idx] = k;
                    return Some(old_val);
                } else {
                    // Merge
                    self.merge(idx);
                    return self.children[idx].remove(key);
                }
            }
            Err(idx) => idx,
        };

        if self.is_leaf {
            return None;
        }

        // Key not in this node, it might be in children[idx]
        if self.children[idx].keys.len() <= Self::MIN_KEYS {
            self.fill(idx);
        }

        match self.keys.binary_search(key) {
            Ok(_) => self.remove(key),
            Err(new_idx) => self.children[new_idx].remove(key),
        }
    }

    fn fill(&mut self, i: usize) {
        if i != 0 && self.children[i - 1].keys.len() > Self::MIN_KEYS {
            self.borrow_from_prev(i);
        } else if i != self.keys.len() && self.children[i + 1].keys.len() > Self::MIN_KEYS {
            self.borrow_from_next(i);
        } else {
            if i != self.keys.len() {
                self.merge(i);
            } else {
                self.merge(i - 1);
            }
        }
    }

    fn borrow_from_prev(&mut self, i: usize) {
        let (left, right) = self.children.split_at_mut(i);
        let sibling = &mut left[i - 1];
        let child = &mut right[0];

        let k = self.keys.remove(i - 1);
        let v = self.values.remove(i - 1);

        let sibling_k = sibling.keys.pop().unwrap();
        let sibling_v = sibling.values.pop().unwrap();

        self.keys.insert(i - 1, sibling_k);
        self.values.insert(i - 1, sibling_v);

        child.keys.insert(0, k);
        child.values.insert(0, v);
        if !child.is_leaf {
            let sibling_child = sibling.children.pop().unwrap();
            child.children.insert(0, sibling_child);
        }
    }

    fn borrow_from_next(&mut self, i: usize) {
        let (left, right) = self.children.split_at_mut(i + 1);
        let child = &mut left[i];
        let sibling = &mut right[0];

        let k = self.keys.remove(i);
        let v = self.values.remove(i);

        let sibling_k = sibling.keys.remove(0);
        let sibling_v = sibling.values.remove(0);

        self.keys.insert(i, sibling_k);
        self.values.insert(i, sibling_v);

        child.keys.push(k);
        child.values.push(v);
        if !child.is_leaf {
            let sibling_child = sibling.children.remove(0);
            child.children.push(sibling_child);
        }
    }

    fn merge(&mut self, i: usize) {
        let mut next = self.children.remove(i + 1);
        let k = self.keys.remove(i);
        let v = self.values.remove(i);

        let child = &mut self.children[i];
        child.keys.push(k);
        child.values.push(v);
        child.keys.append(&mut next.keys);
        child.values.append(&mut next.values);
        if !child.is_leaf {
            child.children.append(&mut next.children);
        }
    }

    fn pop_first(&mut self) -> (K, V) {
        if self.is_leaf {
            (self.keys.remove(0), self.values.remove(0))
        } else {
            if self.children[0].keys.len() <= Self::MIN_KEYS {
                self.fill(0);
            }
            self.children[0].pop_first()
        }
    }

    fn pop_last(&mut self) -> (K, V) {
        if self.is_leaf {
            (self.keys.pop().unwrap(), self.values.pop().unwrap())
        } else {
            let last_idx = self.children.len() - 1;
            if self.children[last_idx].keys.len() <= Self::MIN_KEYS {
                self.fill(last_idx);
            }
            let last_idx = self.children.len() - 1;
            self.children[last_idx].pop_last()
        }
    }
}

pub struct BTree<K, V, const ORDER: usize = 8> {
    root: Option<BTreeNode<K, V, ORDER>>,
}

impl<K: Ord + Clone, V: Clone, const ORDER: usize> BTree<K, V, ORDER> {
    pub fn new() -> Self {
        Self { root: None }
    }

    pub fn insert(&mut self, key: K, value: V) {
        if let Some(ref mut root) = self.root {
            if root.keys.len() == ORDER - 1 {
                let mut new_root = BTreeNode::new(false);
                let old_root = std::mem::replace(root, BTreeNode::new(true));
                new_root.children.push(old_root);
                new_root.split_child(0);
                new_root.insert_non_full(key, value);
                *root = new_root;
            } else {
                root.insert_non_full(key, value);
            }
        } else {
            let mut root = BTreeNode::new(true);
            root.keys.push(key);
            root.values.push(value);
            self.root = Some(root);
        }
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        self.root.as_ref().and_then(|r| r.get(key))
    }

    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        self.root.as_mut().and_then(|r| r.get_mut(key))
    }

    pub fn range(&self, range: RangeInclusive<K>) -> Vec<(K, V)> {
        let mut results = Vec::new();
        if let Some(ref root) = self.root {
            root.range(&range, &mut results);
        }
        results
    }

    pub fn remove(&mut self, key: &K) -> Option<V> {
        let res = self.root.as_mut().and_then(|r| r.remove(key));
        if let Some(ref root) = self.root {
            if root.keys.is_empty() {
                if root.is_leaf {
                    self.root = None;
                } else {
                    self.root = Some(self.root.take().unwrap().children.remove(0));
                }
            }
        }
        res
    }
}
