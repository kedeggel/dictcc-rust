extern crate csv;
#[macro_use]
extern crate failure;
extern crate htmlescape;
extern crate serde;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate nom;

pub mod parse;
pub mod dict;
mod error;
