#[macro_use]
mod macros;
mod tests;

pub mod parser;

pub use parser::{ParseError, Parser};
