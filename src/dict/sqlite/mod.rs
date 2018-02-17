extern crate rusqlite;

use dict::read::DictccDBReader;
use dict::DictEntry;
use parse::html::HtmlDecodedDictEntry;
use parse::raw_csv::RawDictEntry;
use parse::word_ast::WordNodesDictEntry;
use error::DictResult;
use rusqlite::{Connection, Transaction};
use std::path::Path;
use dict::query::DictQueryResult;
use dict::query::QueryDirection;
use error::DictError;

// TODO: Dict/DictQuery trait
// TODO: Highlight match
// TODO: replace colored with termcolor and keep tables

#[derive(Debug)]
pub(crate) struct EntryQueryRow {
    pub(crate) left_indexed_word: String,
    pub(crate) right_indexed_word: String,
    pub(crate) left_word: String,
    pub(crate) right_word: String,
    pub(crate) word_classes: String,
    pub(crate) highlight_left_indexed_word: String,
    pub(crate) highlight_right_indexed_word: String,
}

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

    pub fn query(&mut self, query_term: &str, query_direction: QueryDirection) -> DictResult<DictQueryResult> {
        let mut stmt = self.conn.prepare(include_str!("sql/query_entry.sql"))?;

        // TODO: query builder
        // TODO: query direction
        // TODO: query types
        // FIXME: query term SQL-injection
        let rows = stmt.query_map(&[&query_term], |row| {
            EntryQueryRow {
                left_indexed_word: row.get(0),
                right_indexed_word: row.get(1),
                left_word: row.get(2),
                right_word: row.get(3),
                word_classes: row.get(4),
                highlight_left_indexed_word: row.get(5),
                highlight_right_indexed_word: row.get(6),
            }
        })?.map(|res| res.map_err(DictError::from)).collect::<DictResult<Vec<EntryQueryRow>>>()?;

        let entries = rows.iter()
            .map(|entry_query_row| {
                let html_decoded_entry = HtmlDecodedDictEntry::from(entry_query_row);
                let word_ast = WordNodesDictEntry::from(&html_decoded_entry);
                let entry = DictEntry::from(word_ast);
                entry
            }).collect();

        Ok(DictQueryResult {
            entries,
            query_direction,
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