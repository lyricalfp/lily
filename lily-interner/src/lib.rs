use std::{
    fmt::Debug,
    hash::{Hash, Hasher},
    ops::Deref,
    ptr,
};

use bumpalo::Bump;
use rustc_hash::FxHashMap;

mod private {
    #[derive(Debug, Clone, Copy)]
    pub struct PrivateZst;
}

pub struct Interned<'a, T>(pub &'a T, pub private::PrivateZst);

impl<'a, T> Interned<'a, T> {
    fn new(t: &'a T) -> Self {
        Self(t, private::PrivateZst)
    }
}

impl<T> Debug for Interned<'_, T>
where
    T: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Interned").field(&self.0).finish()
    }
}

impl<T> PartialEq for Interned<'_, T> {
    fn eq(&self, other: &Self) -> bool {
        ptr::eq(self.0, other.0)
    }
}

impl<T> Eq for Interned<'_, T> {}

impl<T> Clone for Interned<'_, T> {
    fn clone(&self) -> Self {
        Self(self.0, private::PrivateZst)
    }
}

impl<T> Copy for Interned<'_, T> {}

impl<T> Hash for Interned<'_, T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        ptr::hash(self.0, state)
    }
}

impl<'a, T> Deref for Interned<'a, T> {
    type Target = T;

    fn deref(&self) -> &'a Self::Target {
        self.0
    }
}

#[derive(Debug)]
pub struct Interner<'a, T> {
    map: FxHashMap<&'a T, usize>,
    vec: Vec<&'a T>,
    arena: &'a Bump,
}

impl<'a, T> Interner<'a, T>
where
    T: Eq + Hash,
{
    pub fn new(arena: &'a Bump) -> Self {
        Self {
            map: FxHashMap::default(),
            vec: Vec::default(),
            arena,
        }
    }

    pub fn intern(&mut self, value: T) -> Interned<'a, T> {
        if let Some(&index) = self.map.get(&value) {
            Interned::new(self.vec[index])
        } else {
            let value = self.arena.alloc(value);
            let index = self.vec.len();

            self.map.insert(value, index);
            self.vec.push(value);

            Interned::new(value)
        }
    }
}

pub struct InternedString<'a>(pub &'a str, pub private::PrivateZst);

impl<'a> InternedString<'a> {
    fn new(t: &'a str) -> Self {
        Self(t, private::PrivateZst)
    }
}

impl Debug for InternedString<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("InternedString").field(&self.0).finish()
    }
}

impl PartialEq for InternedString<'_> {
    fn eq(&self, other: &Self) -> bool {
        ptr::eq(self.0, other.0)
    }
}

impl Eq for InternedString<'_> {}

impl Clone for InternedString<'_> {
    fn clone(&self) -> Self {
        Self(self.0, private::PrivateZst)
    }
}

impl Copy for InternedString<'_> {}

impl Hash for InternedString<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        ptr::hash(self.0, state)
    }
}

#[derive(Debug)]
pub struct StringInterner<'a> {
    map: FxHashMap<&'a str, usize>,
    vec: Vec<&'a str>,
    arena: &'a Bump,
}

impl<'a> StringInterner<'a> {
    pub fn new(arena: &'a Bump) -> Self {
        Self {
            map: FxHashMap::default(),
            vec: Vec::default(),
            arena,
        }
    }

    pub fn intern(&mut self, value: &'a str) -> InternedString<'a> {
        if let Some(&index) = self.map.get(&value) {
            InternedString::new(self.vec[index])
        } else {
            let value = self.arena.alloc_str(value);
            let index = self.vec.len();

            self.map.insert(value, index);
            self.vec.push(value);

            InternedString::new(value)
        }
    }
}

#[cfg(test)]
mod tests {
    use bumpalo::Bump;

    use crate::{Interner, StringInterner};

    #[test]
    pub fn it_works_as_intended() {
        let arena_0 = Bump::new();
        let mut interner_0 = StringInterner::new(&arena_0);

        let hello_a_0 = interner_0.intern("hello");
        let hello_b_0 = interner_0.intern("hello");

        assert_eq!(hello_a_0, hello_b_0);

        let arena_1 = Bump::new();
        let mut interner_1 = StringInterner::new(&arena_1);

        let hello_a_1 = interner_1.intern("hello");
        let hello_b_1 = interner_1.intern("hello");

        assert_ne!(hello_a_0, hello_a_1);
        assert_ne!(hello_a_0, hello_b_1);
        assert_eq!(hello_a_1, hello_b_1);

        let arena_2 = Bump::new();
        let mut interner_2 = Interner::new(&arena_2);

        let x = interner_2.intern(42);
        let y = interner_2.intern(21 + 21);

        assert_eq!(x, y)
    }
}
