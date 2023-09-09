use crate::core::{Font, Size};
use crate::text;

use rustc_hash::{FxHashMap, FxHashSet};
use std::collections::hash_map;
use std::hash::{BuildHasher, Hash, Hasher};

#[allow(missing_debug_implementations)]
#[derive(Default)]
pub struct Cache {
    entries: FxHashMap<KeyHash, Entry>,
    aliases: FxHashMap<KeyHash, KeyHash>,
    recently_used: FxHashSet<KeyHash>,
    hasher: HashBuilder,
}

#[cfg(not(target_arch = "wasm32"))]
type HashBuilder = twox_hash::RandomXxHashBuilder64;

#[cfg(target_arch = "wasm32")]
type HashBuilder = std::hash::BuildHasherDefault<twox_hash::XxHash64>;

impl Cache {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get(&self, key: &KeyHash) -> Option<&Entry> {
        self.entries.get(key)
    }

    pub fn allocate(
        &mut self,
        font_system: &mut cosmic_text::FontSystem,
        key: Key<'_>,
    ) -> (KeyHash, &mut Entry) {
        let hash = key.hash(self.hasher.build_hasher());

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
                key.bounds.width,
                key.bounds.height.max(key.line_height),
            );
            buffer.set_text(
                font_system,
                key.content,
                text::to_attributes(key.font),
                text::to_shaping(key.shaping),
            );

            let bounds = text::measure(&buffer);
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
                        Key { bounds, ..key }.hash(self.hasher.build_hasher()),
                        hash,
                    );
                }
            }
        }

        let _ = self.recently_used.insert(hash);

        (hash, self.entries.get_mut(&hash).unwrap())
    }

    pub fn trim(&mut self) {
        self.entries
            .retain(|key, _| self.recently_used.contains(key));

        self.aliases
            .retain(|_, value| self.recently_used.contains(value));

        self.recently_used.clear();
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Key<'a> {
    pub content: &'a str,
    pub size: f32,
    pub line_height: f32,
    pub font: Font,
    pub bounds: Size,
    pub shaping: text::Shaping,
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

        hasher.finish()
    }
}

pub type KeyHash = u64;

#[allow(missing_debug_implementations)]
pub struct Entry {
    pub buffer: cosmic_text::Buffer,
    pub min_bounds: Size,
}
