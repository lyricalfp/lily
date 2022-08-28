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

pub type FixityMap = FxHashMap<SmolStr, Fixity>;
