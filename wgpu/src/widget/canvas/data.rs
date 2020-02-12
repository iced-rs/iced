#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Data<T> {
    raw: T,
    version: usize,
}

impl<T> Data<T> {
    pub fn new(data: T) -> Self {
        Data {
            raw: data,
            version: 0,
        }
    }

    pub fn update(&mut self, f: impl FnOnce(&mut T)) {
        f(&mut self.raw);

        self.version += 1;
    }
}
