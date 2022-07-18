use std::{
    fmt::{self, Debug, Display},
    hash::Hash,
    ops::Deref,
    ptr,
};

use bumpalo::Bump;
use rustc_hash::FxHashMap;

mod private {
    #[derive(Debug, Clone, Copy)]
    pub struct Zst;
}

pub struct Interned<'a, T>(pub &'a T, pub private::Zst);

impl<'a, T> Interned<'a, T> {
    pub fn new(value: &'a T) -> Self {
        Self(value, private::Zst)
    }
}

impl<T> Debug for Interned<'_, T>
where
    T: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Interned").field(&self.0).finish()
    }
}

impl<T> PartialEq for Interned<'_, T> {
    fn eq(&self, other: &Self) -> bool {
        ptr::eq(self.0, other.0)
    }
}

impl<T> Clone for Interned<'_, T> {
    fn clone(&self) -> Self {
        Self(self.0.clone(), private::Zst)
    }
}

impl<T> Copy for Interned<'_, T> {}

impl<T> Eq for Interned<'_, T> {}

impl<T> Hash for Interned<'_, T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        ptr::hash(self.0, state)
    }
}

impl<'a, T> Deref for Interned<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.0
    }
}

#[derive(Debug)]
pub struct Interner<'a, T> {
    indices: FxHashMap<&'a T, usize>,
    entries: Vec<&'a T>,
    arena: &'a Bump,
}

impl<'a, T> Interner<'a, T> {
    pub fn new(arena: &'a Bump) -> Self {
        Self {
            indices: FxHashMap::default(),
            entries: Vec::default(),
            arena,
        }
    }
}

impl<'a, T> Interner<'a, T>
where
    T: Eq + Hash,
{
    pub fn intern(&mut self, value: T) -> Interned<'a, T> {
        if let Some(&index) = self.indices.get(&value) {
            Interned::new(self.entries[index])
        } else {
            let value = self.arena.alloc(value);
            let index = self.entries.len();

            self.indices.insert(value, index);
            self.entries.push(value);

            Interned::new(value)
        }
    }
}

pub struct InternedString<'a>(pub &'a str, pub private::Zst);

impl<'a> InternedString<'a> {
    fn new(value: &'a str) -> Self {
        Self(value, private::Zst)
    }
}

impl Debug for InternedString<'_> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_tuple("InternedString")
            .field(&self.0)
            .finish()
    }
}

impl Display for InternedString<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.0)
    }
}

impl PartialEq for InternedString<'_> {
    fn eq(&self, other: &Self) -> bool {
        ptr::eq(self.0, other.0)
    }
}

impl Clone for InternedString<'_> {
    fn clone(&self) -> Self {
        Self(self.0, private::Zst)
    }
}

impl Copy for InternedString<'_> {}

impl Eq for InternedString<'_> {}

impl Hash for InternedString<'_> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        ptr::hash(self.0, state)
    }
}

#[derive(Debug)]
pub struct StringInterner<'a> {
    indices: FxHashMap<&'a str, usize>,
    entries: Vec<&'a str>,
    arena: &'a Bump,
}

impl<'a> StringInterner<'a> {
    pub fn new(arena: &'a Bump) -> Self {
        Self {
            indices: FxHashMap::default(),
            entries: Vec::default(),
            arena,
        }
    }

    pub fn intern(&mut self, value: &'a str) -> InternedString<'a> {
        if let Some(&index) = self.indices.get(&value) {
            InternedString::new(self.entries[index])
        } else {
            let value = self.arena.alloc_str(value);
            let index = self.entries.len();

            self.indices.insert(value, index);
            self.entries.push(value);

            InternedString::new(value)
        }
    }
}
