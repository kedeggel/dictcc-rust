use dict::DictEntry;
use parse::html::HtmlDecodedDictEntry;
use parse::word_ast::WordNodesDictEntry;
use error::DictResult;
use dict::query::DictQueryResult;
use dict::query::QueryDirection;
use error::DictError;
use dict::sqlite::SqliteDict;
use dict::language::Language;

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

/// Builder for a `DictQueryResult`.
#[derive(Debug)]
pub struct SqliteDictQuery<'a, 'b> {
    pub(crate) dict: &'a SqliteDict,
    pub(crate) query_term: &'b str,
    pub(crate) query_direction: QueryDirection,
}

impl<'a, 'b> SqliteDictQuery<'a, 'b> {
    /// Set the query direction.
    pub fn set_direction(&mut self, query_direction: QueryDirection) -> &mut Self {
        self.query_direction = query_direction;
        self
    }

    /// Set the query term.
    pub fn set_term<'c>(self, query_term: &'c str) -> SqliteDictQuery<'a, 'c> {
        SqliteDictQuery {
            dict: self.dict,
            query_term,
            query_direction: self.query_direction,
        }
    }

    /// Sets the query direction based on the given source language.
    ///
    /// Convenience function for `set_query_direction`
    pub fn source_language(&mut self, source_language: &Language) -> DictResult<&mut Self> {
        let query_direction = self.dict.language_pair().infer_query_direction(source_language)?;
        self.set_direction(query_direction);
        Ok(self)
    }

    pub fn execute(&self) -> DictResult<DictQueryResult> {
        let mut stmt = self.dict.conn.prepare(include_str!("sql/query_entry.sql"))?;

        let query_term = self.query_term;

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
        })?
            .map(|res| res.map_err(DictError::from))
            .collect::<DictResult<Vec<EntryQueryRow>>>()?;

        let entries = rows.iter()
            .map(|entry_query_row| {
                let html_decoded_entry = HtmlDecodedDictEntry::from(entry_query_row);
                let word_ast = WordNodesDictEntry::from(&html_decoded_entry);
                let entry = DictEntry::from(word_ast);
                entry
            }).collect();

        Ok(DictQueryResult {
            entries,
            query_direction: self.query_direction,
        })
    }
}