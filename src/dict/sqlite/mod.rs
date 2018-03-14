extern crate rusqlite;

use dict::read::DictccDBReader;
use dict::DictEntry;
use parse::html::HtmlDecodedDictEntry;
use parse::raw_csv::RawDictEntry;
use parse::word_ast::WordNodesDictEntry;
use error::DictResult;
use rusqlite::{Connection, Transaction};
use std::path::Path;
use dict::sqlite::query::SqliteDictQuery;
use dict::query::QueryDirection;
use dict::language::DictLanguagePair;

pub mod query;

// TODO: Highlight match
// TODO: replace colored with termcolor and keep tables


#[derive(Debug)]
pub struct SqliteDict {
    conn: Connection,
}

impl SqliteDict {
    pub fn open<P: AsRef<Path>>(sqlite_db_path: P) -> DictResult<SqliteDict> {
        let version = rusqlite::version();
        debug!("version = {:?}", version);

        let conn = Connection::open(sqlite_db_path)?;

        Ok(SqliteDict {
            conn,
        })
    }

    pub fn new<P1: AsRef<Path>, P2: AsRef<Path>>(sqlite_db_path: P1, dictcc_db_path: P2) -> DictResult<SqliteDict> {
        // TODO: save language and other dictcc db meta info in sqlite

        let version = rusqlite::version();
        debug!("version = {:?}", version);

        let conn = Connection::open(sqlite_db_path)?;

        let mut dict_reader = DictccDBReader::new(dictcc_db_path)?;

        let mut sql_dict = SqliteDict {
            conn,
        };

        sql_dict.seed_db(&mut dict_reader)?;

        Ok(sql_dict)
    }

    pub fn query<'a, 'b>(&'a self, query_term: &'b str) -> SqliteDictQuery<'a, 'b> {
        SqliteDictQuery {
            dict: self,
            query_term,
            query_direction: QueryDirection::Bidirectional,
        }
    }

    /// Return the language pair of the dictionary.
    fn language_pair(&self) -> &DictLanguagePair {
        unimplemented!()
    }

    fn seed_db(&mut self, mut reader: &mut DictccDBReader) -> DictResult<()> {
        {
            let tx = self.conn.transaction()?;

            tx.execute_batch(include_str!("sql/pre_seed.sql"))?;

            SqliteDict::insert_entries(&mut reader, &tx)?;

            info!("Committing");

            tx.commit()?;
        }

        info!("Post seed cleanup");
        self.conn.execute_batch(include_str!("sql/post_seed.sql"))?;
        info!("Post seed cleanup finished");

        Ok(())
    }

    fn insert_entries(reader: &mut DictccDBReader, tx: &Transaction) -> DictResult<()> {
        let mut statement = tx.prepare(include_str!("sql/insert_entry.sql"))?;

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
        }

        statement.finalize()?;

        Ok(())
    }
}
