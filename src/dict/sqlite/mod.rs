extern crate rusqlite;

use dict::read::DictReader;
use dict::DictEntry;
use parse::html::HtmlDecodedDictEntry;
use parse::raw_csv::RawDictEntry;
use parse::word_ast::WordNodesDictEntry;
use error::DictResult;
use rusqlite::{Connection, Transaction};
use std::path::Path;

#[derive(Debug)]
pub struct SqliteDict {
    conn: Connection,
}

impl SqliteDict {
    pub fn open<P: AsRef<Path>>(sqlite_db_path: P) -> DictResult<SqliteDict> {
        let conn = Connection::open(sqlite_db_path)?;

        Ok(SqliteDict {
            conn,
        })
    }

    pub fn new<P1: AsRef<Path>, P2: AsRef<Path>>(sqlite_db_path: P1, dictcc_db_path: P2) -> DictResult<SqliteDict> {
        let conn = Connection::open(sqlite_db_path)?;

        //TODO

        Ok(SqliteDict {
            conn,
        })
    }

    fn seed_db(&mut self, mut reader: &mut DictReader) -> DictResult<()> {
        {
            let tx = self.conn.transaction()?;

            tx.execute_batch(include_str!("sql/pre_seed.sql"))?;

            SqliteDict::insert_entries(&mut reader, &tx)?;

            println!("Committing");

            tx.commit()?;
        }

        println!("Post seed cleanup");
        self.conn.execute_batch(include_str!("sql/post_seed.sql"))?;
        println!("Post seed cleanup finished");

        Ok(())
    }

    fn insert_entries(reader: &mut DictReader, tx: &Transaction) -> DictResult<()> {
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

}