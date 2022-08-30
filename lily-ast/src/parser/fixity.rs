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
