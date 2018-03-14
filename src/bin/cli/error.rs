extern crate failure;
extern crate app_dirs;
extern crate dictcc;
extern crate simplelog;

use dictcc::error::DictError;
use std::io;

pub type DictCliResult<T> = ::std::result::Result<T, DictCliError>;

#[derive(Debug, Fail)]
pub enum DictCliError {
    #[fail(display = "No dictionary has been added.")]
    NoDictionary,

    #[fail(display = "Invalid dictionary identifier: {}", _0)]
    InvalidDictId(String),

    #[fail(display = "{}", _0)]
    DictError(#[cause] DictError),

    #[fail(display = "{}", _0)]
    TermLogError(#[cause] simplelog::TermLogError),

    #[fail(display = "{}", _0)]
    AppDirsError(#[cause] app_dirs::AppDirsError),

    #[fail(display = "{}", _0)]
    Io(#[cause] io::Error),
}

impl From<app_dirs::AppDirsError> for DictCliError {
    fn from(err: app_dirs::AppDirsError) -> Self {
        DictCliError::AppDirsError(err)
    }
}

impl From<DictError> for DictCliError {
    fn from(err: DictError) -> Self {
        DictCliError::DictError(err)
    }
}

impl From<io::Error> for DictCliError {
    fn from(err: io::Error) -> Self {
        DictCliError::Io(err)
    }
}

impl From<simplelog::TermLogError> for DictCliError {
    fn from(err: simplelog::TermLogError) -> Self {
        DictCliError::TermLogError(err)
    }
}
