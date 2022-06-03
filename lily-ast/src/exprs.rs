use std::collections::BTreeMap;

use lily_interner::{Interned, InternedString};

use crate::{ann::Ann, types::Type};

use self::extra::LilyF64;

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum LiteralKind<'a, T> {
    Char(char),
    String(InternedString<'a>),
    Int(i32),
    Float(LilyF64),
    Array(Vec<T>),
    Object(BTreeMap<InternedString<'a>, Expr<'a>>),
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum ApplicationArgument<'a> {
    ExprArgument(Expr<'a>),
    TypeArgument(Type<'a>),
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum ExprKind<'a> {
    Annotation(Expr<'a>, Type<'a>),
    Application(Expr<'a>, ApplicationArgument<'a>),
    IfThenElse(Expr<'a>, Expr<'a>, Expr<'a>),
    Lambda(InternedString<'a>, Expr<'a>),
    LetBinding(InternedString<'a>, Option<Type<'a>>, Expr<'a>),
    Literal(LiteralKind<'a, Expr<'a>>),
    Variable(InternedString<'a>),
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Expr<'a>(pub Interned<'a, Ann>, pub Interned<'a, ExprKind<'a>>);

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum BinderKind<'a> {
    Null,
    Literal(LiteralKind<'a, Binder<'a>>),
    Variable(InternedString<'a>),
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Binder<'a>(pub Interned<'a, Ann>, pub Interned<'a, BinderKind<'a>>);

/* Extraneous Utilities */

pub mod extra {
    use std::hash::{Hash, Hasher};

    #[derive(Debug)]
    pub struct LilyF64(f64);

    impl PartialEq for LilyF64 {
        fn eq(&self, other: &Self) -> bool {
            let _ = self.0.sqrt();
            self.0.to_bits() == other.0.to_bits()
        }
    }

    impl Eq for LilyF64 {}

    impl Hash for LilyF64 {
        fn hash<H: Hasher>(&self, state: &mut H) {
            self.0.to_bits().hash(state);
        }
    }

    impl From<f64> for LilyF64 {
        fn from(value: f64) -> Self {
            Self(value)
        }
    }
}
