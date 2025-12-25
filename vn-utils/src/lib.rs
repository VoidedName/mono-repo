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
