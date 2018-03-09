extern crate dictcc;
extern crate failure;
extern crate log;
extern crate rusqlite;
extern crate simplelog;

use dictcc::sqlite::SqliteDict;
use failure::Error;
use simplelog::{LevelFilter, TermLogger};
use dictcc::query::QueryDirection;

fn main() {
    if let Err(err) = run() {
        eprintln!("{}", err);
        eprintln!("err = {:?}", err);
    }
}

fn run() -> Result<(), Error> {
    TermLogger::init(LevelFilter::Trace, simplelog::Config::default())?;

    //let mut dict = SqliteDict::new("database/sqlite/test.db", "database/dictcc_DE-EN.txt")?;
    let mut dict = SqliteDict::open("database/sqlite/test.db")?;

    let query_result = dict.query("house", QueryDirection::Bidirectional)?;

    println!("{}", query_result.into_grouped());

    Ok(())
}
