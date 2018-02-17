//#![warn(missing_docs)]
#![warn(missing_debug_implementations)]

//! Rust API for reading and querying the dict.cc offline translation database.

extern crate csv;
#[macro_use]
extern crate failure;
extern crate htmlescape;
extern crate serde;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate nom;
#[macro_use]
extern crate log;
extern crate regex;
#[macro_use]
extern crate prettytable;
extern crate itertools;
extern crate colored;

#[cfg(feature = "sqlite")]
extern crate rusqlite;


pub mod parse;
pub mod error;
mod dict;

pub use dict::*;