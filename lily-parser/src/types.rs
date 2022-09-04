use rustc_hash::FxHashMap;
use smol_str::SmolStr;

#[derive(Debug)]
pub enum Associativity {
    Infixl,
    Infixr,
}

#[derive(Debug)]
pub struct Fixity {
    pub begin: usize,
    pub end: usize,
    pub associativity: Associativity,
    pub binding_power: u8,
    pub identifier: SmolStr,
}

impl Fixity {
    pub fn as_pair(&self) -> (u8, u8) {
        match self.associativity {
            Associativity::Infixl => (self.binding_power, self.binding_power + 1),
            Associativity::Infixr => (self.binding_power + 1, self.binding_power),
        }
    }
}

pub type FixityMap = FxHashMap<SmolStr, Fixity>;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct LesserPattern {
    pub begin: usize,
    pub end: usize,
    pub kind: LesserPatternK,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum LesserPatternK {
    Null,
    Variable(SmolStr),
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct GreaterPattern {
    pub begin: usize,
    pub end: usize,
    pub kind: GreaterPatternK,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum GreaterPatternK {
    Application(Box<GreaterPattern>, Vec<GreaterPattern>),
    BinaryOperator(Box<GreaterPattern>, SmolStr, Box<GreaterPattern>),
    Constructor(SmolStr),
    Integer(SmolStr),
    Null,
    Parenthesized(Box<GreaterPattern>),
    Variable(SmolStr),
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Expression {
    pub begin: usize,
    pub end: usize,
    pub kind: ExpressionK,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ExpressionK {
    Application(Box<Expression>, Vec<Expression>),
    BinaryOperator(Box<Expression>, SmolStr, Box<Expression>),
    Constructor(SmolStr),
    DoBlock(Vec<DoStatement>),
    IfThenElse(Box<Expression>, Box<Expression>, Box<Expression>),
    Integer(SmolStr),
    Float(SmolStr),
    Parenthesized(Box<Expression>),
    Variable(SmolStr),
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct DoStatement {
    pub begin: usize,
    pub end: usize,
    pub kind: DoStatementK,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum DoStatementK {
    BindExpression(LesserPattern, Expression),
    DiscardExpression(Expression),
    LetStatement(Vec<Declaration>),
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Declaration {
    pub begin: usize,
    pub end: usize,
    pub kind: DeclarationK,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum DeclarationK {
    ValueDeclaration(SmolStr, Vec<LesserPattern>, Expression),
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Module {
    pub declarations: Vec<Declaration>,
}
