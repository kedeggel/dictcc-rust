extern crate csv;
extern crate failure;
extern crate htmlescape;
extern crate nom;

use std::io;

use failure::{Backtrace, Context};

pub type DictResult<T> = ::std::result::Result<T, DictError>;

#[derive(Debug, Fail)]
pub enum DictError {
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

    #[fail(display = "{}", _0)]
    Io(#[cause] io::Error),

    #[fail(display = "Could not open dictionary file at {:?}: {}", path, cause)]
    FileOpen {
        path: String,
        #[cause] cause: csv::Error,
    },

    #[fail(display = "Incomplete entry in dictionary: {}", _0)]
    IncompleteEntry(#[cause] csv::Error),

    #[fail(display = "Could not parse csv: {}", _0)]
    CsvParse(#[cause] csv::Error),

    #[fail(display = "Could not decode HTML character references: {:?}", _0)]
    HtmlDecode(htmlescape::DecodeErr),

    #[fail(display = "Could not parse {:?}: {:?}", word, cause)]
    WordASTParse {
        word: String,
        cause: nom::IError,
    },

    #[fail(display = "Could not completely parse {:?}: remaining input: {:?}", word, remaining_input)]
    WordASTRemainingInput {
        word: String,
        remaining_input: String,
    },

    #[fail(display = "Parse error with context: {:?}", _0)]
    Context(#[cause] Context<String>),
}

impl From<csv::Error> for DictError {
    fn from(err: csv::Error) -> Self {
        DictError::CsvParse(err)
    }
}

impl From<htmlescape::DecodeErr> for DictError {
    fn from(err: htmlescape::DecodeErr) -> Self {
        DictError::HtmlDecode(err)
    }
}

impl From<Context<String>> for DictError {
    fn from(context: Context<String>) -> Self {
        DictError::Context(context)
    }
}
