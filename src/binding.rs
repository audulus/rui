/// Reads or writes a value owned by a source-of-truth.
pub trait Binding<S>: Clone + Copy + 'static {
    fn with<T, F: FnOnce(&S) -> T>(&self, f: F) -> T;
    fn with_mut<T, F: FnOnce(&mut S) -> T>(&self, f: F) -> T;

    fn get(&self) -> S
    where
        S: Clone,
    {
        self.with(|s| s.clone())
    }

    fn set(&self, value: S) {
        self.with_mut(move |s| *s = value);
    }
}

