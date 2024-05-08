use crate::cache::{self, Cached};
use crate::core::Size;
use crate::geometry::{self, Frame};

pub use cache::Group;

/// A simple cache that stores generated geometry to avoid recomputation.
///
/// A [`Cache`] will not redraw its geometry unless the dimensions of its layer
/// change or it is explicitly cleared.
pub struct Cache<Renderer>
where
    Renderer: geometry::Renderer,
{
    raw: crate::Cache<Data<<Renderer::Geometry as Cached>::Cache>>,
}

#[derive(Debug, Clone)]
struct Data<T> {
    bounds: Size,
    geometry: T,
}

impl<Renderer> Cache<Renderer>
where
    Renderer: geometry::Renderer,
{
    /// Creates a new empty [`Cache`].
    pub fn new() -> Self {
        Cache {
            raw: cache::Cache::new(),
        }
    }

    /// Creates a new empty [`Cache`] with the given [`Group`].
    ///
    /// Caches within the same group may reuse internal rendering storage.
    ///
    /// You should generally group caches that are likely to change
    /// together.
    pub fn with_group(group: Group) -> Self {
        Cache {
            raw: crate::Cache::with_group(group),
        }
    }

    /// Clears the [`Cache`], forcing a redraw the next time it is used.
    pub fn clear(&self) {
        self.raw.clear();
    }

    /// Draws geometry using the provided closure and stores it in the
    /// [`Cache`].
    ///
    /// The closure will only be called when
    /// - the bounds have changed since the previous draw call.
    /// - the [`Cache`] is empty or has been explicitly cleared.
    ///
    /// Otherwise, the previously stored geometry will be returned. The
    /// [`Cache`] is not cleared in this case. In other words, it will keep
    /// returning the stored geometry if needed.
    pub fn draw(
        &self,
        renderer: &Renderer,
        bounds: Size,
        draw_fn: impl FnOnce(&mut Frame<Renderer>),
    ) -> Renderer::Geometry {
        use std::ops::Deref;

        let state = self.raw.state();

        let previous = match state.borrow().deref() {
            cache::State::Empty { previous } => {
                previous.as_ref().map(|data| data.geometry.clone())
            }
            cache::State::Filled { current } => {
                if current.bounds == bounds {
                    return Cached::load(&current.geometry);
                }

                Some(current.geometry.clone())
            }
        };

        let mut frame = Frame::new(renderer, bounds);
        draw_fn(&mut frame);

        let geometry = frame.into_geometry().cache(self.raw.group(), previous);
        let result = Cached::load(&geometry);

        *state.borrow_mut() = cache::State::Filled {
            current: Data { bounds, geometry },
        };

        result
    }
}

impl<Renderer> std::fmt::Debug for Cache<Renderer>
where
    Renderer: geometry::Renderer,
    <Renderer::Geometry as Cached>::Cache: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", &self.raw)
    }
}

impl<Renderer> Default for Cache<Renderer>
where
    Renderer: geometry::Renderer,
{
    fn default() -> Self {
        Self::new()
    }
}
