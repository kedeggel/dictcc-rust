extern crate csv;

use std::error::Error;
use std::path::Path;
use std::fs::File;

fn records_filter(record: &Result<csv::StringRecord, csv::Error>) -> bool {
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


fn get_csv_reader_from_path<P: AsRef<Path>>(path: P) -> csv::Result<csv::Reader<File>> {
    csv::ReaderBuilder::new()
        .delimiter(b'\t')
        .has_headers(false)
        .quoting(false)
        .comment(Some(b'#'))
        .from_path(path)
}

pub fn parse_test() -> Result<(), Box<Error>> {
    get_csv_reader_from_path("../database/dictcc_DE-EN.txt")?
        .records()
        .filter(records_filter)
        .enumerate()
        .for_each(|(i, record)| {
            if i % 10000 == 0 {
                eprintln!("record = {:?}", record);
            }
        });

    Ok(())
}