extern crate csv;

use std::path::Path;
use std::fs::File;

use error::ParseDictionaryError;

fn incomplete_records_filter(record: &Result<csv::StringRecord, csv::Error>) -> bool {
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

fn get_csv_reader_from_path<P: AsRef<Path>>(path: P) -> Result<csv::Reader<File>, ParseDictionaryError> {
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

pub fn parse_test() -> Result<(), ParseDictionaryError> {
    let mut reader = get_csv_reader_from_path("../database/dictcc_DE-EN.txt")?;

    let records = reader
        .records()
        .filter(incomplete_records_filter)
        .enumerate();

    for (i, record) in records {
        let record = record?;

        if i % 10000 == 0 {
            eprintln!("record = {:?}", record);
        }
    }

    Ok(())
}
