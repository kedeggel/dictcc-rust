extern crate dictcc;
extern crate failure;
extern crate rusqlite;

use dictcc::read::DictReader;
use dictcc::parse::html::HtmlDecodedDictEntry;
use dictcc::parse::raw_csv::RawDictEntry;
use dictcc::DictEntry;
use failure::Error;
use rusqlite::{Connection, Transaction};
use dictcc::parse::word_ast::WordNodesDictEntry;
use dictcc::sqlite::SqliteDict;

fn main() {
    let version = rusqlite::version();
    eprintln!("version = {:?}", version);

    if let Err(err) = run() {
        eprintln!("{}", err);
        eprintln!("err = {:?}", err);
    }
}

fn run() -> Result<(), Error> {
    SqliteDict::new("database/sqlite/test.db", "database/dictcc_DE-EN.txt")?;

    Ok(())
}
