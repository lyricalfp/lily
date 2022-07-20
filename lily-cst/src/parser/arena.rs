use std::{
    fmt::{Debug, Display},
    hash::Hash,
    marker::PhantomData,
    ops::Deref,
};

use bumpalo::Bump;
use rustc_hash::FxHashSet;

use super::types::ExpressionK;

mod private {
    #[derive(Debug, Clone, Copy)]
    pub struct Zst;
}

#[derive(Default)]
pub struct Coliseum<'a> {
    expressions: Bump,
    strings: Bump,
    marker: PhantomData<&'a ()>,
}

impl<'a> Coliseum<'a> {
    #[inline]
    pub fn alloc_expression(&self, value: ExpressionK<'a>) -> &ExpressionK<'a> {
        self.expressions.alloc(value)
    }

    #[inline]
    pub fn alloc_string(&self, value: &str) -> &str {
        self.strings.alloc_str(value)
    }

    #[inline]
    pub fn allocated_bytes(&self) -> usize {
        self.expressions.allocated_bytes() + self.strings.allocated_bytes()
    }
}

pub struct Interner<'a> {
    coliseum: &'a Coliseum<'a>,
    expressions: FxHashSet<&'a ExpressionK<'a>>,
    strings: FxHashSet<&'a str>,
}

impl<'a> Interner<'a> {
    pub fn new(coliseum: &'a Coliseum<'a>) -> Self {
        Self {
            coliseum,
            expressions: FxHashSet::default(),
            strings: FxHashSet::default(),
        }
    }

    pub fn intern_expression(&mut self, value: ExpressionK<'a>) -> Interned<'a, ExpressionK<'a>> {
        if let Some(&value) = self.expressions.get(&value) {
            Interned::new(value)
        } else {
            let value = self.coliseum.alloc_expression(value);
            self.expressions.insert(value);
            Interned::new(value)
        }
    }

    pub fn intern_string(&mut self, value: &'a str) -> Symbol<'a> {
        if let Some(&value) = self.strings.get(value) {
            Symbol::new(value)
        } else {
            let value = self.coliseum.alloc_string(value);
            self.strings.insert(value);
            Symbol::new(value)
        }
    }
}

pub struct Interned<'a, T>(pub &'a T, pub private::Zst);

impl<'a, T> Interned<'a, T> {
    fn new(value: &'a T) -> Self {
        Self(value, private::Zst)
    }
}

impl<'a, T> Debug for Interned<'a, T>
where
    T: Debug,
{
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.debug_tuple("Interned").field(&self.0).finish()
    }
}

impl<'a, T> Display for Interned<'a, T>
where
    T: Display,
{
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(formatter)
    }
}

impl<'a, T> Clone for Interned<'a, T>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        Self(<&T>::clone(&self.0), private::Zst)
    }
}

impl<'a, T> Copy for Interned<'a, T> where T: Copy {}

impl<'a, T> PartialEq for Interned<'a, T> {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self.0, other.0)
    }
}

impl<'a, T> Eq for Interned<'a, T> {}

impl<'a, T> Hash for Interned<'a, T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        std::ptr::hash(self.0, state)
    }
}

pub struct Symbol<'a>(pub &'a str, pub private::Zst);

impl<'a> Symbol<'a> {
    fn new(value: &'a str) -> Self {
        Self(value, private::Zst)
    }
}

impl<'a> Debug for Symbol<'a> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.debug_tuple("Symbol").field(&self.0).finish()
    }
}

impl<'a> Display for Symbol<'a> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.0)
    }
}

impl<'a> Clone for Symbol<'a> {
    fn clone(&self) -> Self {
        Self(<&str>::clone(&self.0), private::Zst)
    }
}

impl<'a> Copy for Symbol<'a> {}

impl<'a> PartialEq for Symbol<'a> {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self.0, other.0)
    }
}

impl<'a> Eq for Symbol<'a> {}

impl<'a> Hash for Symbol<'a> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        std::ptr::hash(self.0, state)
    }
}

impl<'a> Deref for Symbol<'a> {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::types::ExpressionK;

    use super::{Coliseum, Interner};

    #[test]
    fn string_interning() {
        let coliseum = Coliseum::default();
        let mut interner = Interner::new(&coliseum);
        let v1 = interner.intern_string("hello");
        let v2 = interner.intern_string("hello");
        assert_eq!(v1, v2)
    }

    #[test]
    fn expression_interning() {
        let coliseum = Coliseum::default();
        let mut interner = Interner::new(&coliseum);
        let v1 = interner.intern_string("hello");
        let v2 = interner.intern_string("hello");
        let k1 = interner.intern_expression(ExpressionK::Variable(v1));
        let k2 = interner.intern_expression(ExpressionK::Variable(v2));
        assert_eq!(v1, v2);
        assert_eq!(k1, k2);
    }
}
