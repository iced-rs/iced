//! Generate events asynchronously for you application.

/// An event subscription.
pub struct Subscription<Hasher, Input, Output> {
    recipes: Vec<Box<dyn Recipe<Hasher, Input, Output = Output>>>,
}

impl<H, I, O> Subscription<H, I, O>
where
    H: std::hash::Hasher,
{
    pub fn none() -> Self {
        Self {
            recipes: Vec::new(),
        }
    }

    pub fn from_recipe(
        recipe: impl Recipe<H, I, Output = O> + 'static,
    ) -> Self {
        Self {
            recipes: vec![Box::new(recipe)],
        }
    }

    pub fn batch(
        subscriptions: impl Iterator<Item = Subscription<H, I, O>>,
    ) -> Self {
        Self {
            recipes: subscriptions
                .flat_map(|subscription| subscription.recipes)
                .collect(),
        }
    }

    pub fn recipes(self) -> Vec<Box<dyn Recipe<H, I, Output = O>>> {
        self.recipes
    }

    pub fn map<A>(
        mut self,
        f: impl Fn(O) -> A + Send + Sync + 'static,
    ) -> Subscription<H, I, A>
    where
        H: 'static,
        I: 'static,
        O: 'static,
        A: 'static,
    {
        let function = std::sync::Arc::new(f);

        Subscription {
            recipes: self
                .recipes
                .drain(..)
                .map(|recipe| {
                    Box::new(Map::new(recipe, function.clone()))
                        as Box<dyn Recipe<H, I, Output = A>>
                })
                .collect(),
        }
    }
}

impl<I, O, H> std::fmt::Debug for Subscription<I, O, H> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Subscription").finish()
    }
}

/// The connection of an event subscription.
pub trait Recipe<Hasher: std::hash::Hasher, Input> {
    type Output;

    fn hash(&self, state: &mut Hasher);

    fn stream(
        &self,
        input: Input,
    ) -> futures::stream::BoxStream<'static, Self::Output>;
}

struct Map<Hasher, Input, A, B> {
    recipe: Box<dyn Recipe<Hasher, Input, Output = A>>,
    mapper: std::sync::Arc<dyn Fn(A) -> B + Send + Sync>,
}

impl<H, I, A, B> Map<H, I, A, B> {
    fn new(
        recipe: Box<dyn Recipe<H, I, Output = A>>,
        mapper: std::sync::Arc<dyn Fn(A) -> B + Send + Sync + 'static>,
    ) -> Self {
        Map { recipe, mapper }
    }
}

impl<H, I, A, B> Recipe<H, I> for Map<H, I, A, B>
where
    A: 'static,
    B: 'static,
    H: std::hash::Hasher,
{
    type Output = B;

    fn hash(&self, state: &mut H) {
        use std::hash::Hash;

        std::any::TypeId::of::<B>().hash(state);
        self.recipe.hash(state);
    }

    fn stream(
        &self,
        input: I,
    ) -> futures::stream::BoxStream<'static, Self::Output> {
        use futures::StreamExt;

        let mapper = self.mapper.clone();

        self.recipe
            .stream(input)
            .map(move |element| mapper(element))
            .boxed()
    }
}
