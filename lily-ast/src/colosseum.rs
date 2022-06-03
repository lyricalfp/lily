use bumpalo::Bump;
use lily_interner::{Interned, InternedString, Interner, StringInterner};

use crate::{
    ann::Ann,
    exprs::{Expr, ExprKind},
    types::{ApplicationKind, QuantifierKind, Type, TypeKind, VariableKind},
};

#[derive(Debug)]
pub struct Colosseum<'a> {
    ann_interner: Interner<'a, Ann>,
    string_interner: StringInterner<'a>,
    expr_kind_interner: Interner<'a, ExprKind<'a>>,
    type_kind_interner: Interner<'a, TypeKind<'a>>,
}

impl<'a> Colosseum<'a> {
    pub fn new(arena: &'a Bump) -> Self {
        Self {
            ann_interner: Interner::new(arena),
            string_interner: StringInterner::new(arena),
            expr_kind_interner: Interner::new(arena),
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
    fn intern_expr_kind(&mut self, value: ExprKind<'a>) -> Interned<'a, ExprKind<'a>> {
        self.expr_kind_interner.intern(value)
    }

    #[inline]
    fn intern_type_kind(&mut self, value: TypeKind<'a>) -> Interned<'a, TypeKind<'a>> {
        self.type_kind_interner.intern(value)
    }

    #[inline]
    pub fn make_expr(&mut self, ann: Ann, expr_kind: ExprKind<'a>) -> Expr<'a> {
        let ann = self.intern_ann(ann);
        let expr_kind = self.intern_expr_kind(expr_kind);
        Expr(ann, expr_kind)
    }

    #[inline]
    pub fn make_type(&mut self, ann: Ann, type_kind: TypeKind<'a>) -> Type<'a> {
        let ann = self.intern_ann(ann);
        let type_kind = self.intern_type_kind(type_kind);
        Type(ann, type_kind)
    }

    pub fn make_type_application(
        &mut self,
        ann: Ann,
        function: Type<'a>,
        argument: Type<'a>,
    ) -> Type<'a> {
        self.make_type(
            ann,
            TypeKind::Application(ApplicationKind::TypeApplication, function, argument),
        )
    }

    pub fn make_kind_application(
        &mut self,
        ann: Ann,
        function: Type<'a>,
        argument: Type<'a>,
    ) -> Type<'a> {
        self.make_type(
            ann,
            TypeKind::Application(ApplicationKind::KindApplication, function, argument),
        )
    }

    pub fn make_constructor(&mut self, ann: Ann, name: &'a str) -> Type<'a> {
        let name = self.intern_string(name);
        self.make_type(ann, TypeKind::Constructor(name))
    }

    pub fn make_forall(
        &mut self,
        ann: Ann,
        name: &'a str,
        knd: Option<Type<'a>>,
        typ: Type<'a>,
    ) -> Type<'a> {
        let name = self.intern_string(name);
        self.make_type(
            ann,
            TypeKind::Quantifier(QuantifierKind::Universal(name), knd, typ),
        )
    }

    pub fn make_exists(
        &mut self,
        ann: Ann,
        name: &'a str,
        knd: Option<Type<'a>>,
        typ: Type<'a>,
    ) -> Type<'a> {
        let name = self.intern_string(name);
        self.make_type(
            ann,
            TypeKind::Quantifier(QuantifierKind::Existential(name), knd, typ),
        )
    }

    pub fn make_function(&mut self, ann: Ann, argument: Type<'a>, result: Type<'a>) -> Type<'a> {
        self.make_type(ann, TypeKind::Function(argument, result))
    }

    pub fn make_kinded(&mut self, ann: Ann, typ: Type<'a>, knd: Type<'a>) -> Type<'a> {
        self.make_type(ann, TypeKind::Kinded(typ, knd))
    }

    pub fn make_skolem(&mut self, ann: Ann, name: &'a str) -> Type<'a> {
        let name = self.intern_string(name);
        self.make_type(ann, TypeKind::Variable(VariableKind::Skolem(name)))
    }

    pub fn make_variable(&mut self, ann: Ann, name: &'a str) -> Type<'a> {
        let name = self.intern_string(name);
        self.make_type(ann, TypeKind::Variable(VariableKind::Syntactic(name)))
    }

    pub fn make_unification(&mut self, ann: Ann, name: u32) -> Type<'a> {
        self.make_type(ann, TypeKind::Variable(VariableKind::Unification(name)))
    }
}

#[cfg(test)]
mod tests {
    use crate::ann::Ann;

    use super::Colosseum;
    use bumpalo::Bump;

    #[test]
    fn it_works_as_intended() {
        let arena = Bump::new();
        let mut colosseum = Colosseum::new(&arena);

        // `forall (a : Type) . a -> a`
        let k = colosseum.make_constructor(Ann::FromCompiler, "Type");
        let v = colosseum.make_variable(Ann::FromCompiler, "a");
        let f = colosseum.make_function(Ann::FromCompiler, v, v);
        let _ = colosseum.make_forall(Ann::FromCompiler, "a", Some(k), f);

        let allocated_0 = arena.allocated_bytes();

        for _ in 1..100 {
            let k = colosseum.make_constructor(Ann::FromCompiler, "Type");
            let v = colosseum.make_variable(Ann::FromCompiler, "a");
            let f = colosseum.make_function(Ann::FromCompiler, v, v);
            let _ = colosseum.make_forall(Ann::FromCompiler, "a", Some(k), f);
        }

        let allocated_1 = arena.allocated_bytes();

        assert_eq!(allocated_0, allocated_1);
    }
}
