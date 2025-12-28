pub trait UpdateOption<T> {
    /// Replaces the value with the result of the update function.
    fn update<F>(&mut self, update: F)
    where
        F: FnOnce(T) -> T;

    /// Replaces the value with the result of the update functions inner value.
    fn flat_update<F>(&mut self, update: F)
    where
        F: FnOnce(T) -> Option<T>;
}

impl<T> UpdateOption<T> for Option<T> {
    fn update<F>(&mut self, update: F)
    where
        F: FnOnce(T) -> T,
    {
        self.flat_update(|t| Some(update(t)));
    }

    fn flat_update<F>(&mut self, update: F)
    where
        F: FnOnce(T) -> Option<T>,
    {
        if self.is_some() {
            *self = update(self.take().unwrap());
        }
    }
}

pub mod string {
    pub trait CharIndex {
        fn byte_pos_for_char_index(&self, index: usize) -> Option<usize>;
    }

    pub trait InsertAtCharIndex: CharIndex {
        fn insert_at_char_index(&mut self, index: usize, c: char);
        fn insert_str_at_char_index(&mut self, index: usize, c: &str);
    }

    pub trait RemoveAtCharIndex: CharIndex {
        fn remove_at_char_index(&mut self, index: usize);
    }

    impl CharIndex for String {
        fn byte_pos_for_char_index(&self, index: usize) -> Option<usize> {
            self.char_indices()
                .enumerate()
                .find_map(|(idx, (byte_pos, _))| if idx == index { Some(byte_pos) } else { None })
        }
    }

    impl InsertAtCharIndex for String {
        fn insert_at_char_index(&mut self, index: usize, c: char) {
            let index = self.byte_pos_for_char_index(index).unwrap_or(self.len());

            self.insert(index, c);
        }

        fn insert_str_at_char_index(&mut self, index: usize, c: &str) {
            let index = self.byte_pos_for_char_index(index).unwrap_or(self.len());

            self.insert_str(index, c);
        }
    }

    impl RemoveAtCharIndex for String {
        fn remove_at_char_index(&mut self, index: usize) {
            let index = self.byte_pos_for_char_index(index).unwrap_or(self.len());

            self.remove(index);
        }
    }
}
