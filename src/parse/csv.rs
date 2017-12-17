extern crate csv;

use std::path::Path;
use std::fs::File;

use super::ParseResult;
use error::ParseDictionaryError;

pub fn incomplete_records_filter(record: &Result<csv::StringRecord, csv::Error>) -> bool {
    match *record {
        Ok(_) => true,
        Err(ref err) => {
            match *err.kind() {
                csv::ErrorKind::UnequalLengths { .. } => {
                    eprintln!("Drop incomplete entry: {:?}", err);
                    false
                }
                _ => true,
            }
        }
    }
}

pub fn get_csv_reader_from_path<P: AsRef<Path>>(path: P) -> ParseResult<csv::Reader<File>> {
    csv::ReaderBuilder::new()
        .delimiter(b'\t')
        .has_headers(false)
        .quoting(false)
        .comment(Some(b'#'))
        .from_path(&path)
        .map_err(|err| ParseDictionaryError::FileOpen {
            path: format!("{}", path.as_ref().display()),
            cause: err,
        })
}


