use std::path::Path;
use std::fs::File;

use super::ParseResult;
use error::ParseDictionaryError;
use super::csv::{get_csv_reader_from_path, incomplete_records_filter};

pub fn parse_test() -> ParseResult<()> {
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