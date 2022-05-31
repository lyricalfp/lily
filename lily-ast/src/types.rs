use crate::ann::Ann;
use lily_interner::{Interned, InternedString};

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum ApplicationKind {
    TypeApplication,
    KindApplication,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum QuantifierKind<'a> {
    Universal(InternedString<'a>),
    Existential(InternedString<'a>),
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum VariableKind<'a> {
    Skolem(InternedString<'a>),
    Syntactic(InternedString<'a>),
    Unification(u32),
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum TypeKind<'a> {
    Application(ApplicationKind, Type<'a>, Type<'a>),
    Constructor(InternedString<'a>),
    Function(Type<'a>, Type<'a>),
    Kinded(Type<'a>, Type<'a>),
    Quantifier(QuantifierKind<'a>, Option<Type<'a>>, Type<'a>),
    Variable(VariableKind<'a>),
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct Type<'a>(pub Interned<'a, Ann>, pub Interned<'a, TypeKind<'a>>);
