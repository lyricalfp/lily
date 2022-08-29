use smol_str::SmolStr;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct LesserPattern {
    pub begin: usize,
    pub end: usize,
    pub kind: LesserPatternK
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum LesserPatternK {
    Null,
    Variable(SmolStr),
}
