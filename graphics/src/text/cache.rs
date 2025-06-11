//! Cache text.
use crate::core::{Font, Size};
use crate::text;

use rustc_hash::{FxHashMap, FxHashSet, FxHasher};
use std::collections::hash_map;
use std::hash::{Hash, Hasher};

/// A store of recently used sections of text.
#[derive(Debug, Default)]
pub struct Cache {
    entries: FxHashMap<KeyHash, Entry>,
    aliases: FxHashMap<KeyHash, KeyHash>,
    recently_used: FxHashSet<KeyHash>,
}

impl Cache {
    /// Creates a new empty [`Cache`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Gets the text [`Entry`] with the given [`KeyHash`].
    pub fn get(&self, key: &KeyHash) -> Option<&Entry> {
        self.entries.get(key)
    }

    /// Allocates a text [`Entry`] if it is not already present in the [`Cache`].
    pub fn allocate(
        &mut self,
        font_system: &mut cosmic_text::FontSystem,
        key: Key<'_>,
    ) -> (KeyHash, &mut Entry) {
        let hash = key.hash(FxHasher::default());

        if let Some(hash) = self.aliases.get(&hash) {
            let _ = self.recently_used.insert(*hash);

            return (*hash, self.entries.get_mut(hash).unwrap());
        }

        if let hash_map::Entry::Vacant(entry) = self.entries.entry(hash) {
            let metrics = cosmic_text::Metrics::new(
                key.size,
                key.line_height.max(f32::MIN_POSITIVE),
            );
            let mut buffer = cosmic_text::Buffer::new(font_system, metrics);

            buffer.set_size(
                font_system,
                Some(key.bounds.width),
                Some(key.bounds.height.max(key.line_height)),
            );
            buffer.set_text(
                font_system,
                key.content,
                &text::to_attributes(key.font),
                text::to_shaping(key.shaping),
            );

            let bounds = text::align(&mut buffer, font_system, key.align_x);

            let _ = entry.insert(Entry {
                buffer,
                min_bounds: bounds,
            });

            for bounds in [
                bounds,
                Size {
                    width: key.bounds.width,
                    ..bounds
                },
            ] {
                if key.bounds != bounds {
                    let _ = self.aliases.insert(
                        Key { bounds, ..key }.hash(FxHasher::default()),
                        hash,
                    );
                }
            }
        }

        let _ = self.recently_used.insert(hash);

        (hash, self.entries.get_mut(&hash).unwrap())
    }

    /// Trims the [`Cache`].
    ///
    /// This will clear the sections of text that have not been used since the last `trim`.
    pub fn trim(&mut self) {
        self.entries
            .retain(|key, _| self.recently_used.contains(key));

        self.aliases
            .retain(|_, value| self.recently_used.contains(value));

        self.recently_used.clear();
    }
}

/// A cache key representing a section of text.
#[derive(Debug, Clone, Copy)]
pub struct Key<'a> {
    /// The content of the text.
    pub content: &'a str,
    /// The size of the text.
    pub size: f32,
    /// The line height of the text.
    pub line_height: f32,
    /// The [`Font`] of the text.
    pub font: Font,
    /// The bounds of the text.
    pub bounds: Size,
    /// The shaping strategy of the text.
    pub shaping: text::Shaping,
    /// The alignment of the text.
    pub align_x: text::Alignment,
}

impl Key<'_> {
    fn hash<H: Hasher>(self, mut hasher: H) -> KeyHash {
        self.content.hash(&mut hasher);
        self.size.to_bits().hash(&mut hasher);
        self.line_height.to_bits().hash(&mut hasher);
        self.font.hash(&mut hasher);
        self.bounds.width.to_bits().hash(&mut hasher);
        self.bounds.height.to_bits().hash(&mut hasher);
        self.shaping.hash(&mut hasher);
        self.align_x.hash(&mut hasher);

        hasher.finish()
    }
}

/// The hash of a [`Key`].
pub type KeyHash = u64;

/// A cache entry.
#[derive(Debug)]
pub struct Entry {
    /// The buffer of text, ready for drawing.
    pub buffer: cosmic_text::Buffer,
    /// The minimum bounds of the text.
    pub min_bounds: Size,
}
