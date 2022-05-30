use std::{
    hash::{Hash, Hasher},
    ptr,
};

use bumpalo::Bump;
use rustc_hash::FxHashMap;

mod private {
    #[derive(Debug, Clone, Copy)]
    pub struct PrivateZst;
}

#[derive(Debug)]
pub struct Interned<'a, T>(&'a T, private::PrivateZst);

impl<'a, T> Interned<'a, T> {
    fn new(t: &'a T) -> Self {
        Self(t, private::PrivateZst)
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

#[cfg(test)]
mod tests {
    use bumpalo::Bump;

    use crate::Interner;

    type StringInterner<'a> = Interner<'a, &'a str>;

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
    }
}
