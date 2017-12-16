extern crate csv;
extern crate failure;

use std::io;

use failure::Backtrace;

#[derive(Debug, Fail)]
pub enum DictError {
    // Own errors

    #[fail(display = "Unknown gender name: {}", name)]
    UnknownGender {
        name: String,
        // TODO: consistent backtrace handling
        backtrace: Backtrace,
    },
    #[fail(display = "Unknown word class: {}", word_class)]
    UnknownWordClass {
        word_class: String,
        backtrace: Backtrace,
    },
    #[fail(display = "Could not parse dictionary: {}", _0)]
    ParseDictionary(#[cause] ParseDictionaryError),

    // Foreign errors to simply pass through
    #[fail(display = "{}", _0)]
    Io(#[cause] io::Error),
}


// Error type for module parse
#[derive(Debug, Fail)]
pub enum ParseDictionaryError {
    #[fail(display = "Could not open dictionary file at \"{}\": {}", path, cause)]
    FileOpen {
        path: String,
        #[cause] cause: csv::Error,
    },

    #[fail(display = "Incomplete entry in dictionary: {}", _0)]
    IncompleteEntry(#[cause] csv::Error),

    #[fail(display = "Could not parse csv: {}", _0)]
    CsvParsing(#[cause] csv::Error)
}

impl From<csv::Error> for ParseDictionaryError {
    fn from(err: csv::Error) -> Self {
        ParseDictionaryError::CsvParsing(err)
    }
}
