pub mod html;
pub mod raw_csv;
pub mod debug;
pub mod brackets;

use std;
use error::ParseDictionaryError;

pub type ParseResult<T> = std::result::Result<T, ParseDictionaryError>;

