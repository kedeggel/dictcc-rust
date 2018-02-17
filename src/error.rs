//! Error handling.

extern crate csv;
extern crate failure;
extern crate htmlescape;
extern crate nom;
extern crate regex;

#[cfg(feature = "sqlite")]
extern crate rusqlite;

use dict::Language;
use failure::Backtrace;
use std::io;

/// Type alias for Result with preconfigured error type `DictError`.
pub type DictResult<T> = ::std::result::Result<T, DictError>;

/// All errors that can be returned by this crate.
#[allow(missing_docs)]
#[derive(Debug, Fail)]
pub enum DictError {
    #[fail(display = "Unknown gender name: {}", name)]
    UnknownGender {
        name: String,
        backtrace: Backtrace,
    },
    #[fail(display = "Unknown word class: {}", word_class)]
    UnknownWordClass {
        word_class: String,
        backtrace: Backtrace,
    },
    #[fail(display = "Unknown query type: {}", query_type)]
    UnknownQueryType {
        query_type: String,
        backtrace: Backtrace,
    },
    #[fail(display = "Invalid language code: {}", lang)]
    InvalidLanguageCode {
        lang: String,
        backtrace: Backtrace,
    },
    #[fail(display = "Invalid source language: {}", source_language)]
    InvalidSourceLanguage {
        source_language: Language,
        backtrace: Backtrace,
    },

    #[fail(display = "Could not completely parse {:?}: remaining input: {:?}", word, remaining_input)]
    WordASTRemainingInput {
        word: String,
        remaining_input: String,
        backtrace: Backtrace,
    },

    #[fail(display = "Language code not found")]
    LanguageCodeNotFound {
        backtrace: Backtrace,
    },

    #[fail(display = "Could not open dictionary file at {:?}: {}", path, cause)]
    FileOpen {
        path: String,
        #[cause] cause: csv::Error,
        backtrace: Backtrace,
    },

    #[fail(display = "{}", _0)]
    Io(#[cause] io::Error, Backtrace),

    #[cfg(feature = "sqlite")]
    #[fail(display = "Error while interacting with SQLite database: {}", _0)]
    Rusqlite(#[cause] rusqlite::Error, Backtrace),

    #[fail(display = "Incomplete entry in dictionary: {}", _0)]
    IncompleteEntry(#[cause] csv::Error, Backtrace),

    #[fail(display = "Could not parse csv: {}", _0)]
    CsvParse(#[cause] csv::Error, Backtrace),

    #[fail(display = "Could not decode HTML character references: {:?}", _0)]
    HtmlDecode(htmlescape::DecodeErr, Backtrace),

    #[fail(display = "Could not parse {:?}: {:?}", word, cause)]
    WordASTParse {
        word: String,
        cause: nom::IError,
        backtrace: Backtrace,
    },

    #[fail(display = "{}", _0)]
    Regex(#[cause] regex::Error, Backtrace),
}

#[cfg(feature = "sqlite")]
impl From<rusqlite::Error> for DictError {
    fn from(err: rusqlite::Error) -> Self {
        DictError::Rusqlite(err, Backtrace::new())
    }
}

impl From<io::Error> for DictError {
    fn from(err: io::Error) -> Self {
        DictError::Io(err, Backtrace::new())
    }
}

impl From<csv::Error> for DictError {
    fn from(err: csv::Error) -> Self {
        DictError::CsvParse(err, Backtrace::new())
    }
}

impl From<htmlescape::DecodeErr> for DictError {
    fn from(err: htmlescape::DecodeErr) -> Self {
        DictError::HtmlDecode(err, Backtrace::new())
    }
}

impl From<regex::Error> for DictError {
    fn from(err: regex::Error) -> Self {
        DictError::Regex(err, Backtrace::new())
    }
}
