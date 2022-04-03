//! String interning.

use bimap::BiHashMap;
use std::ops::Index;

/// Lightweight unique key for a string.
#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
pub struct StringKey(usize);

impl StringKey {
    /// The key for the empty string. The same across all caches.
    pub const EMPTY: Self = StringKey(0);
}

#[derive(Debug)]
pub struct StringCache {
    strings: BiHashMap<StringKey, String>,
}

impl StringCache {
    /// Creates a new `StringCache`.
    pub fn new() -> Self {
        let mut strings = BiHashMap::new();
        strings.insert(StringKey::EMPTY, String::new());
        Self { strings }
    }

    /// Retrieve the key for the given string. The key is unique to this cache.
    pub fn intern(&mut self, str: &str) -> StringKey {
        self.strings.get_by_right(str).copied().unwrap_or_else(|| {
            let key = StringKey(self.strings.len());
            self.strings.insert(key, str.to_owned());
            key
        })
    }

    /// Retrieve the string associated with the given key.
    pub fn get(&self, key: StringKey) -> Option<&str> {
        self.strings.get_by_left(&key).map(String::as_str)
    }
}

impl Index<StringKey> for StringCache {
    type Output = str;

    fn index(&self, index: StringKey) -> &Self::Output {
        self.get(index).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let mut cache = StringCache::new();
        let a = cache.intern("a");
        let b = cache.intern("b");
        let c = cache.intern("c");

        assert_eq!("a", &cache[a]);
        assert_eq!("b", &cache[b]);
        assert_eq!("c", &cache[c]);
    }
}
