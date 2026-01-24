use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

pub type NodePtr<T> = Rc<RefCell<Option<LinkedListNode<T>>>>;

/// Kinda scuffed LinkedList implementation that allows holding a reference to a node.
/// You could do quite a bit of nonsense with this, but it's useful for implementing caches.
/// Might want to keep sparse arrays for efficiency and cpu cache locality
pub struct LinkedList<T> {
    head: NodePtr<T>,
    tail: NodePtr<T>,
    length: usize,
}

impl<T> LinkedList<T> {
    pub fn new() -> Self {
        Self {
            head: Rc::new(RefCell::new(None)),
            tail: Rc::new(RefCell::new(None)),
            length: 0,
        }
    }

    pub fn push_back(&mut self, value: T) -> NodePtr<T> {
        let node = Rc::new(RefCell::new(Some(LinkedListNode {
            previous: self.tail.clone(),
            next: Rc::new(RefCell::new(None)),
            value: Some(value),
        })));

        if let Some(tail_node) = self.tail.borrow_mut().as_mut() {
            tail_node.next = node.clone();
        } else {
            // was empty -> also set head
            self.head = node.clone();
        }

        self.tail = node.clone();
        self.length += 1;

        node
    }

    pub fn push_front(&mut self, value: T) -> NodePtr<T> {
        let node = Rc::new(RefCell::new(Some(LinkedListNode {
            previous: Rc::new(RefCell::new(None)),
            next: self.head.clone(),
            value: Some(value),
        })));

        if let Some(head_node) = self.head.borrow_mut().as_mut() {
            head_node.previous = node.clone();
        } else {
            // was empty -> also set tail
            self.tail = node.clone();
        }

        self.head = node.clone();
        self.length += 1;

        node
    }

    pub fn move_to_back(&mut self, node_ptr: &NodePtr<T>) {
        if self.length <= 1 {
            return;
        }

        // If it's already the tail, do nothing
        if Rc::ptr_eq(node_ptr, &self.tail) {
            return;
        }

        let (prev, next) = {
            let borrow = node_ptr.borrow();
            let node = borrow.as_ref().unwrap();
            (node.previous.clone(), node.next.clone())
        };

        // If it was the head
        if Rc::ptr_eq(node_ptr, &self.head) {
            self.head = next.clone();
        }

        // Remove from current position
        if let Some(p) = prev.borrow_mut().as_mut() {
            p.next = next.clone();
        }

        if let Some(n) = next.borrow_mut().as_mut() {
            n.previous = prev.clone();
        }

        // Attach to tail
        {
            let mut borrow = node_ptr.borrow_mut();
            let node = borrow.as_mut().unwrap();
            node.next = Rc::new(RefCell::new(None));
            node.previous = self.tail.clone();
        }

        if let Some(tail_node) = self.tail.borrow_mut().as_mut() {
            tail_node.next = node_ptr.clone();
        }

        self.tail = node_ptr.clone();
    }

    pub fn move_to_front(&mut self, node_ptr: &NodePtr<T>) {
        if self.length <= 1 {
            return;
        }

        // If it's already the head, do nothing
        if Rc::ptr_eq(node_ptr, &self.head) {
            return;
        }

        let (prev, next) = {
            let borrow = node_ptr.borrow();
            let node = borrow.as_ref().unwrap();
            (node.previous.clone(), node.next.clone())
        };

        // If it was the tail
        if Rc::ptr_eq(node_ptr, &self.head) {
            self.tail = prev.clone();
        }

        // Remove from current position
        if let Some(p) = prev.borrow_mut().as_mut() {
            p.next = next.clone();
        }

        if let Some(n) = next.borrow_mut().as_mut() {
            n.previous = prev.clone();
        }

        // Attach to head
        {
            let mut borrow = node_ptr.borrow_mut();
            let node = borrow.as_mut().unwrap();
            node.next = self.head.clone();
            node.previous = Rc::new(RefCell::new(None));
        }

        if let Some(head_node) = self.head.borrow_mut().as_mut() {
            head_node.previous = node_ptr.clone();
        }

        self.head = node_ptr.clone();
    }

    pub fn pop_head(&mut self) -> Option<T> {
        let head = self.head.borrow_mut().take()?;

        self.head = head.next.clone();
        self.length -= 1;

        if let Some(head) = self.head.borrow_mut().as_mut() {
            head.previous = Rc::new(RefCell::new(None));
        }

        head.value
    }

    pub fn pop_tail(&mut self) -> Option<T> {
        let tail = self.tail.borrow_mut().take()?;

        self.tail = tail.previous.clone();
        self.length -= 1;

        if let Some(tail) = self.tail.borrow_mut().as_mut() {
            tail.next = Rc::new(RefCell::new(None));
        }

        tail.value
    }

    pub fn head(&self) -> Option<&T> {
        let value = self.head.borrow();
        let value = value.as_ref()?.value.as_ref()?;

        // see cache safety...
        unsafe {
            let val_ptr = value as *const T;
            Some(&*val_ptr)
        }
    }

    pub fn tail(&self) -> Option<&T> {
        let value = self.tail.borrow();
        let value = value.as_ref()?.value.as_ref()?;

        // see cache safety...
        unsafe {
            let val_ptr = value as *const T;
            Some(&*val_ptr)
        }
    }
}

pub struct LinkedListNode<T> {
    pub previous: NodePtr<T>,
    pub next: NodePtr<T>,
    pub value: Option<T>,
}

pub struct CacheEntry<K, V> {
    key: K,
    value: V,
    age: u64,
}

pub struct TimedLRUCache<K, V> {
    lookup: HashMap<K, NodePtr<CacheEntry<K, V>>>,
    elements: LinkedList<CacheEntry<K, V>>,
    generation: u64,
}

#[derive(Default, Copy, Clone)]
pub struct TimedLRUCacheCleanupParams {
    /// If set, the cache will evict all members that are older than max_age many ticks
    pub max_age: Option<u64>,
    /// If set, the cache will evict the oldest members if it holds more than max_entries many members.
    pub max_entries: Option<usize>,
}

impl<K, V> TimedLRUCache<K, V> {
    pub fn new() -> Self {
        Self {
            lookup: HashMap::new(),
            elements: LinkedList::new(),
            generation: 0,
        }
    }
}

impl<K: Eq + std::hash::Hash + Clone, V> TimedLRUCache<K, V> {
    pub fn insert(&mut self, key: K, value: V) {
        let node = self.elements.push_back(CacheEntry {
            key: key.clone(),
            value,
            age: self.generation,
        });
        self.lookup.insert(key, node);
    }

    pub fn get(&mut self, key: &K) -> Option<&V> {
        let node_ptr = self.lookup.get(key)?;

        self.elements.move_to_back(node_ptr);

        let mut borrow = node_ptr.borrow_mut();
        let cache_entry = borrow.as_mut()?.value.as_mut()?;
        cache_entry.age = self.generation;

        // Safety: We are returning a reference to a value owned by the Cache.
        // The value is inside an Rc<RefCell>, and move_to_back doesn't reallocate the entry itself.
        // As long as the Cache is not mutably borrowed again, this reference remains valid.
        // And since the Option has the same lifetime as the cache reference, it should be fine... I think
        // Maybe I should use pins...
        unsafe {
            let val_ptr = &cache_entry.value as *const V;
            Some(&*val_ptr)
        }
    }

    pub fn tick(&mut self) {
        self.generation = self.generation.wrapping_add(1);
    }

    /// Removes all excess entries and those that are too old. Returns the keys of the removed entries.
    pub fn cleanup(&mut self, cleanup_params: TimedLRUCacheCleanupParams) -> Vec<(K, V)> {
        let mut pruned = vec![];

        if let Some(max_entries) = cleanup_params.max_entries {
            // remove excess elements
            while self.elements.length > max_entries {
                let entry = self.elements.pop_head();
                if let Some(entry) = entry {
                    self.lookup.remove(&entry.key);
                    pruned.push((entry.key, entry.value));
                }
            }
        }

        if let Some(max_age) = cleanup_params.max_age {
            // remove entries that are too old
            let mut head_ptr = self.elements.head();

            while let Some(head) = head_ptr
                && (head.age.saturating_add(max_age) < self.generation
                // if we wrap around for some reason, then head.age will be very far in the future
                || head.age.saturating_sub(max_age) > self.generation)
            {
                let entry = self.elements.pop_head();
                if let Some(entry) = entry {
                    self.lookup.remove(&entry.key);
                    pruned.push((entry.key, entry.value));
                }

                head_ptr = self.elements.head();
            }
        }

        pruned
    }

    pub fn len(&self) -> usize {
        self.elements.length
    }
}
