pub trait Layer {
    type Cache;

    fn new() -> Self;

    fn clear(&mut self);
}

pub struct Recorder<T: Layer> {
    layers: Vec<T>,
    caches: Vec<T::Cache>,
    stack: Vec<Kind>,
    current: usize,
}

enum Kind {
    Fresh(usize),
    Cache(usize),
}

impl<T: Layer> Recorder<T> {
    pub fn new() -> Self {
        Self {
            layers: vec![Layer::new()],
            caches: Vec::new(),
            stack: Vec::new(),
            current: 0,
        }
    }

    pub fn fill_quad(&mut self) {}

    pub fn push_cache(&mut self, cache: T::Cache) {
        self.caches.push(cache);
    }

    pub fn clear(&mut self) {
        self.caches.clear();
        self.stack.clear();

        for mut layer in self.layers {
            layer.clear();
        }

        self.current = 0;
    }
}
