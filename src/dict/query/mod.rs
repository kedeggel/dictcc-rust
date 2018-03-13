extern crate csv;

use dict::VecDict;
use dict::DictEntry;
use dict::Language;
use dict::query::grouped::DictQueryResultGrouped;
use error::{DictError, DictResult};
use failure::Backtrace;
use regex::{escape, RegexBuilder};
use std::str::FromStr;

pub mod grouped;

/// Builder for a `DictQueryResult`.
#[derive(Debug)]
pub struct VecDictQuery<'a, 'b> {
    pub(crate) dict: &'a VecDict,
    pub(crate) query_term: &'b str,
    pub(crate) query_type: QueryType,
    pub(crate) query_direction: QueryDirection,
}

impl<'a, 'b> VecDictQuery<'a, 'b> {

    /// Set the query direction.
    pub fn set_direction(&mut self, query_direction: QueryDirection) -> &mut Self {
        self.query_direction = query_direction;
        self
    }

    /// Set the query type.
    pub fn set_type(&mut self, query_type: QueryType) -> &mut Self {
        self.query_type = query_type;
        self
    }

    /// Set the query term.
    pub fn set_term<'c>(self, query_term: &'c str) -> VecDictQuery<'a, 'c> {
        VecDictQuery {
            dict: self.dict,
            query_term,
            query_type: self.query_type,
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

    /// Every entry that contains the query-word is a hit (default!)
    ///
    /// Convenience function for `set_query_type`
    pub fn word(&mut self) -> &mut Self {
        self.set_type(QueryType::Word);
        self
    }

    /// Search for exact matches
    ///
    /// Convenience function for `set_query_type`
    pub fn exact(&mut self) -> &mut Self {
        self.set_type(QueryType::Exact);
        self
    }

    /// Search for regex, so the user can specify by himself what he wants to match
    ///
    /// Convenience function for `set_query_type`
    pub fn regex(&mut self) -> &mut Self {
        self.set_type(QueryType::Regex);
        self
    }

    /// Execute the query.
    pub fn execute(&self) -> DictResult<DictQueryResult> {
        let regexp = match self.query_type {
            QueryType::Word => RegexBuilder::new(&format!(r"(^|\s|-){}($|\s|-)", escape(self.query_term))).case_insensitive(true).build()?,
            QueryType::Exact => RegexBuilder::new(&format!(r"^{}$", escape(self.query_term))).case_insensitive(true).build()?,
            QueryType::Regex => RegexBuilder::new(&format!(r"^{}$", self.query_term)).case_insensitive(true).build()?,
        };

        Ok(DictQueryResult {
            entries: self.dict.entries.iter().filter(|entry| {
                match self.query_direction {
                    QueryDirection::ToRight => regexp.is_match(&entry.left_word.indexed_word),
                    QueryDirection::ToLeft => regexp.is_match(&entry.right_word.indexed_word),
                    QueryDirection::Bidirectional => regexp.is_match(&entry.left_word.indexed_word)
                        || regexp.is_match(&entry.right_word.indexed_word),
                }
            }).cloned().collect(),
            query_direction: self.query_direction,
        })
    }
}

/// Different types of queries. Used by `DictQuery`.
#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum QueryType {
    /// Search for exact matches
    Exact,
    /// Every entry that contains the query-word is a hit
    Regex,
    /// Search for regex, so the user can specify by himself what he wants to match
    Word,
}

impl FromStr for QueryType {
    type Err = DictError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use self::QueryType::*;

        Ok(match s.to_lowercase().as_str() {
            "e" | "exact" => Exact,
            "r" | "regex" => Regex,
            "w" | "word" => Word,
            unknown => Err(DictError::UnknownQueryType {
                query_type: unknown.to_string(),
                backtrace: Backtrace::new(),
            })?
        })
    }
}

/// In which direction a query is executed. Used by `DictQuery`.
/// Can be inferred by `DictLanguagePair::infer_query_direction`.
#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum QueryDirection {
    /// Search in the left language, to get results in the right language.
    ToRight,
    /// Search in the right language, to get results in the left language.
    ToLeft,
    /// Search in both languages.
    Bidirectional,
}

/// Result of a translation query
#[derive(Clone, Eq, PartialEq, Debug)]
pub struct DictQueryResult {
    pub(crate) entries: Vec<DictEntry>,
    pub(crate) query_direction: QueryDirection,
}

impl DictQueryResult {
    /// Returns a slice of all entries found in the query.
    pub fn entries(&self) -> &[DictEntry] {
        &self.entries
    }

    /// Converts a `DictQueryResult` into a grouped representation used for structured display of the found entries.
    pub fn into_grouped(self) -> DictQueryResultGrouped {
        DictQueryResultGrouped::from(self)
    }
}