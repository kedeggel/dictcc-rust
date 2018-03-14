extern crate rusqlite;

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
use std::path::PathBuf;
use dict::read::DictccReader;
use error::DictError;

pub mod query;

// TODO: Highlight match
// TODO: replace colored with termcolor and keep tables
// TODO: remove Dict_ Prefix

#[derive(Debug)]
pub struct SqliteController {
    conn: Connection,
}

impl SqliteController {
    pub fn new<P: AsRef<Path>>(sqlite_db_path: P) -> DictResult<Self> {
        let version = rusqlite::version();
        debug!("version = {:?}", version);

        let conn = Connection::open(sqlite_db_path)?;

        let mut controller = SqliteController {
            conn,
        };

        controller.init_dictionaries_if_not_exists()?;

        Ok(controller)
    }

    fn init_dictionaries_if_not_exists(&mut self) -> DictResult<()> {
        let tx = self.conn.transaction()?;

        let exists = SqliteController::dictionaries_exists(&tx)?;

        if !exists {
            tx.execute_batch(include_str!("sql/index/create.sql"))?;
            tx.commit()?;

            Ok(())
        } else {
            Ok(())
        }
    }

    fn dictionaries_exists(tx: &Transaction) -> DictResult<bool> {
        let mut stmt_exists = tx.prepare(include_str!("sql/index/exists.sql"))?;

        Ok(stmt_exists.exists(&[])?)
    }


    pub fn get_dict<'a>(&'a mut self, dict_id: &DictId) -> DictResult<SqliteDict<'a>> {
        let metadata = DictccMetadata::select(dict_id, &self.conn)?;

        Ok(SqliteDict {
            conn: &mut self.conn,
            metadata,
        })
    }

    pub fn add_dict<'a, P: AsRef<Path>>(&'a mut self, dictcc_db_path: P) -> DictResult<SqliteDict<'a>> {
        // TODO: handle existing dictionary

        let (mut dict_reader, metadata) = DictccReader::new(dictcc_db_path)?;

        let mut sql_dict = SqliteDict {
            conn: &mut self.conn,
            metadata,
        };

        sql_dict.seed(&mut dict_reader)?;

        Ok(sql_dict)
    }

    pub fn list_dicts(&self) -> DictResult<Vec<DictccMetadata>> {
        let mut stmt = self.conn.prepare(include_str!("sql/index/select_all.sql"))?;

        let res = {
            let rows = stmt.query_map(&[], |row| {
                let tpl: (String, String, String, String) = (row.get(0), row.get(1), row.get(2), row.get(3));

                tpl
            })?;

            rows
                .map(|res| res
                    .map_err(DictError::from)
                    .and_then(|(dict_id, left_language, right_language, path)| {
                        Ok(DictccMetadata {
                            languages: DictLanguagePair {
                                left_language: left_language.parse()?,
                                right_language: right_language.parse()?,
                            },
                            path: PathBuf::from(path),
                            dict_id:DictId {
                                id: dict_id,
                            },
                        })
                    })
                )
                .collect()
        };

        res
    }

    pub fn delete(&self, dict_id: &DictId) -> DictResult<()> {
        // TODO: implement

        unimplemented!()
    }
}

#[derive(Debug)]
pub struct DictccMetadata {
    pub languages: DictLanguagePair,
    pub path: PathBuf,
    pub dict_id: DictId,
    // TODO Date
}

impl DictccMetadata {
    pub(crate) fn new(languages: DictLanguagePair, path: PathBuf) -> DictccMetadata {
        DictccMetadata {
            dict_id: DictId::from(&languages),
            languages,
            path,
        }
    }

    fn insert(&self, tx: &Transaction) -> DictResult<()> {
        tx.execute(include_str!("sql/index/insert.sql"), &[
            &self.dict_id.id,
            &self.languages.left_language.language_code(),
            &self.languages.right_language.language_code(),
            self.path.to_string_lossy().to_mut()])?;

        Ok(())
    }

    fn select(dict_id: &DictId, conn: &Connection) -> DictResult<DictccMetadata> {
        // TODO: what to do when DictId is not found?

        let (dict_id, left_language, right_language, path): (String, String, String, String) =
            conn.query_row(include_str!("sql/index/query.sql"), &[&dict_id.id], |row| {
                (row.get(0), row.get(1), row.get(2), row.get(3))
            })?;

        Ok(DictccMetadata {
            languages: DictLanguagePair {
                left_language: left_language.parse()?,
                right_language: right_language.parse()?,
            },
            path: PathBuf::from(path),
            dict_id: DictId {
                id: dict_id,
            },
        })
    }
}

#[derive(Debug)]
pub struct DictId {
    id: String
}

impl DictId {
    fn replace(&self, sql: &str) -> String {
        sql.replace(":dict_id", &self.id)
    }
}

impl<'a> From<&'a DictLanguagePair> for DictId {
    fn from(languages: &'a DictLanguagePair) -> Self {
        DictId {
            id: format!("entries_{}", languages),
        }
    }
}

#[derive(Debug)]
pub struct SqliteDict<'conn> {
    conn: &'conn mut Connection,
    metadata: DictccMetadata,
}

impl<'conn> SqliteDict<'conn> {
    pub fn query<'a, 'b>(&'a mut self, query_term: &'b str) -> SqliteDictQuery<'a, 'b, 'conn> {
        SqliteDictQuery {
            dict: self,
            query_term,
            query_direction: QueryDirection::Bidirectional,
        }
    }

    /// Return the language pair of the dictionary.
    fn language_pair(&self) -> &DictLanguagePair {
        &self.metadata.languages
    }

    fn seed(&mut self, mut reader: &mut DictccReader) -> DictResult<()> {
        {
            let tx = self.conn.transaction()?;

            self.metadata.insert(&tx)?;

            tx.execute_batch(&self.metadata.dict_id.replace(include_str!("sql/entries/create.sql")))?;

            SqliteDict::insert_entries(&mut reader, &tx, &self.metadata.dict_id)?;

            info!("Committing");

            tx.commit()?;
        }

        info!("Post seed cleanup");
        self.conn.execute_batch(&self.metadata.dict_id.replace(include_str!("sql/entries/post_seed.sql")))?;
        info!("Post seed cleanup finished");

        Ok(())
    }

    fn insert_entries(reader: &mut DictccReader, tx: &Transaction, dict_id: &DictId) -> DictResult<()> {
        let mut statement = tx.prepare(&dict_id.replace(include_str!("sql/entries/insert.sql")))?;

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
