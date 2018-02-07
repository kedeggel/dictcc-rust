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

pub mod parse;
pub mod dict;
pub mod error;
