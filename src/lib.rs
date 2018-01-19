extern crate csv;
#[macro_use]
extern crate failure;
extern crate htmlescape;
extern crate serde;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate nom;
extern crate env_logger;
#[macro_use]
extern crate log;
extern crate regex;

pub mod parse;
pub mod dict;
mod error;
