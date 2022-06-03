#[macro_use]
extern crate lalrpop_util;

lalrpop_mod!(#[allow(clippy::all)] pub grammar);

pub mod ann;
pub mod colosseum;
pub mod exprs;
pub mod parser;
pub mod types;
