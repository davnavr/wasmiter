/// Allows [string interning](https://en.wikipedia.org/wiki/String_interning).
///
/// String interning allows for reduced memory usage, especially when parsing certain section of a
/// WebAssembly module that contain duplicate strings. An example of this is the
/// [*import section*](https://webassembly.github.io/spec/core/binary/modules.html#import-section),
/// where module names may be duplicated.
pub trait StringPool {
    /// Type for an interned string.
    type Interned: AsRef<str>;

    /// Interns the given string.
    fn get(&mut self, s: &str) -> Self::Interned;
}

#[cfg(feature = "std")]
impl<T, S> StringPool for std::collections::HashSet<T, S>
where
    T: AsRef<str> + std::borrow::Borrow<str> + Eq + std::hash::Hash + Clone + for<'a> From<&'a str>,
    S: std::hash::BuildHasher,
{
    type Interned = T;

    fn get(&mut self, s: &str) -> Self::Interned {
        if let Some(existing) = std::collections::HashSet::get(self, s) {
            existing.clone()
        } else {
            let interned = T::from(s);
            self.insert(interned.clone());
            interned
        }
    }
}
