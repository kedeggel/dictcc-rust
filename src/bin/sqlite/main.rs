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

fn main() {
    let version = rusqlite::version();
    eprintln!("version = {:?}", version);

    if let Err(err) = run() {
        eprintln!("{}", err);
        eprintln!("err = {:?}", err);
    }
}

fn run() -> Result<(), Error> {
    let mut reader = DictReader::new("database/dictcc_DE-EN.txt")?;

    let mut conn = Connection::open("database/sqlite/test.db")?;

    seed_db(&mut reader, &mut conn)?;

    Ok(())
}

fn seed_db(mut reader: &mut DictReader<&str>, conn: &mut Connection) -> Result<(), Error> {
    {
        let tx = conn.transaction()?;

        tx.execute_batch(include_str!("sql/pre_seed.sql"))?;

        insert_entries(&mut reader, &tx)?;

        println!("Committing");

        tx.commit()?;
    }

    println!("Post seed cleanup");
    conn.execute_batch(include_str!("sql/post_seed.sql"))?;
    println!("Post seed cleanup finished");

    Ok(())
}

fn insert_entries(reader: &mut DictReader<&str>, tx: &Transaction) -> Result<(), Error> {
    let mut statement = tx.prepare(include_str!("sql/insert_entry.sql"))?;
    let mut count = 1;

    for raw_entry_res in reader.raw_entries() {
        let raw_entry: RawDictEntry = raw_entry_res?;
        let html_decoded_entry = HtmlDecodedDictEntry::from(&raw_entry);
        let word_ast = WordNodesDictEntry::from(&html_decoded_entry);
        let entry = DictEntry::from(word_ast);

        statement.execute(&[
            &entry.left_word.indexed_word,
            &entry.right_word.indexed_word,
            &html_decoded_entry.left_word,
            &html_decoded_entry.right_word,
            &html_decoded_entry.word_classes
        ])?;

        count += 1;

        if count % 1000 == 0 {
            eprintln!("count = {:?}", count);
        }
    }

    statement.finalize()?;

    Ok(())
}
