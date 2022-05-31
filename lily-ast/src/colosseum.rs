use bumpalo::Bump;
use lily_interner::{Interned, InternedString, Interner, StringInterner};

use crate::{
    ann::Ann,
    types::{ApplicationKind, QuantifierKind, Type, TypeKind, VariableKind},
};

pub struct Colosseum<'a> {
    ann_interner: Interner<'a, Ann>,
    string_interner: StringInterner<'a>,
    type_kind_interner: Interner<'a, TypeKind<'a>>,
}

impl<'a> Colosseum<'a> {
    pub fn new(arena: &'a Bump) -> Self {
        Self {
            ann_interner: Interner::new(arena),
            string_interner: StringInterner::new(arena),
            type_kind_interner: Interner::new(arena),
        }
    }

    #[inline]
    fn intern_ann(&mut self, value: Ann) -> Interned<'a, Ann> {
        self.ann_interner.intern(value)
    }

    #[inline]
    fn intern_string(&mut self, value: &'a str) -> InternedString<'a> {
        self.string_interner.intern(value)
    }

    #[inline]
    fn intern_type_kind(&mut self, value: TypeKind<'a>) -> Interned<'a, TypeKind<'a>> {
        self.type_kind_interner.intern(value)
    }

    #[inline]
    pub fn make_type(&mut self, ann: Ann, type_kind: TypeKind<'a>) -> Type<'a> {
        let ann = self.intern_ann(ann);
        let type_kind = self.intern_type_kind(type_kind);
        Type(ann, type_kind)
    }

    #[inline]
    pub fn make_compiler_type(&mut self, type_kind: TypeKind<'a>) -> Type<'a> {
        self.make_type(Ann::FromCompiler, type_kind)
    }

    pub fn make_compiler_type_application(
        &mut self,
        function: Type<'a>,
        argument: Type<'a>,
    ) -> Type<'a> {
        self.make_compiler_type(TypeKind::Application(
            ApplicationKind::TypeApplication,
            function,
            argument,
        ))
    }

    pub fn make_compiler_kind_application(
        &mut self,
        function: Type<'a>,
        argument: Type<'a>,
    ) -> Type<'a> {
        self.make_compiler_type(TypeKind::Application(
            ApplicationKind::KindApplication,
            function,
            argument,
        ))
    }

    pub fn make_compiler_constructor(&mut self, name: &'a str) -> Type<'a> {
        let name = self.intern_string(name);
        self.make_compiler_type(TypeKind::Constructor(name))
    }

    pub fn make_compiler_forall(
        &mut self,
        name: &'a str,
        knd: Option<Type<'a>>,
        typ: Type<'a>,
    ) -> Type<'a> {
        let name = self.intern_string(name);
        self.make_compiler_type(TypeKind::Quantifier(
            QuantifierKind::Universal(name),
            knd,
            typ,
        ))
    }

    pub fn make_compiler_exists(
        &mut self,
        name: &'a str,
        knd: Option<Type<'a>>,
        typ: Type<'a>,
    ) -> Type<'a> {
        let name = self.intern_string(name);
        self.make_compiler_type(TypeKind::Quantifier(
            QuantifierKind::Existential(name),
            knd,
            typ,
        ))
    }

    pub fn make_compiler_function(&mut self, argument: Type<'a>, result: Type<'a>) -> Type<'a> {
        self.make_compiler_type(TypeKind::Function(argument, result))
    }

    pub fn make_compiler_kinded(&mut self, typ: Type<'a>, knd: Type<'a>) -> Type<'a> {
        self.make_compiler_type(TypeKind::Kinded(typ, knd))
    }

    pub fn make_compiler_skolem(&mut self, name: &'a str) -> Type<'a> {
        let name = self.intern_string(name);
        self.make_compiler_type(TypeKind::Variable(VariableKind::Skolem(name)))
    }

    pub fn make_compiler_variable(&mut self, name: &'a str) -> Type<'a> {
        let name = self.intern_string(name);
        self.make_compiler_type(TypeKind::Variable(VariableKind::Syntactic(name)))
    }

    pub fn make_compiler_unification(&mut self, name: u32) -> Type<'a> {
        self.make_compiler_type(TypeKind::Variable(VariableKind::Unification(name)))
    }
}

#[cfg(test)]
mod tests {
    use super::Colosseum;
    use bumpalo::Bump;

    #[test]
    fn it_works_as_intended() {
        let arena = Bump::new();
        let mut colosseum = Colosseum::new(&arena);

        // `forall (a : Type) . a -> a`
        let k = colosseum.make_compiler_constructor("Type");
        let v = colosseum.make_compiler_variable("a");
        let f = colosseum.make_compiler_function(v, v);
        let _ = colosseum.make_compiler_forall("a", Some(k), f);

        let allocated_0 = arena.allocated_bytes();

        for _ in 1..100 {
            let k = colosseum.make_compiler_constructor("Type");
            let v = colosseum.make_compiler_variable("a");
            let f = colosseum.make_compiler_function(v, v);
            let _ = colosseum.make_compiler_forall("a", Some(k), f);
        }

        let allocated_1 = arena.allocated_bytes();

        assert_eq!(allocated_0, allocated_1);
    }
}
