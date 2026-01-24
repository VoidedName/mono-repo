pub mod cache;
pub use cache::*;

pub mod dependency_graph;
// pub use dependency_graph::*;

pub mod option {
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
}

pub mod float {
    pub trait NaNTo: Sized {
        fn nan_to(&self, value: Self) -> Self;
        fn replace_nan_with(&mut self, value: Self) -> &mut Self {
            *self = self.nan_to(value);
            self
        }
    }

    impl NaNTo for f32 {
        fn nan_to(&self, value: f32) -> f32 {
            if self.is_nan() { value } else { *self }
        }
    }

    impl NaNTo for f64 {
        fn nan_to(&self, value: f64) -> f64 {
            if self.is_nan() { value } else { *self }
        }
    }
}

pub mod result {
    pub trait MonoResult<T> {
        fn value_ref(&self) -> &T;
        fn value(self) -> T;
    }

    impl<T> MonoResult<T> for Result<T, T> {
        fn value_ref(&self) -> &T {
            match self {
                Ok(v) | Err(v) => v,
            }
        }

        fn value(self) -> T {
            match self {
                Ok(v) | Err(v) => v,
            }
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
