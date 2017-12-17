pub mod html;
pub mod csv;
pub mod debug;

use std;
use error::ParseDictionaryError;

pub type ParseResult<T> = std::result::Result<T, ParseDictionaryError>;

