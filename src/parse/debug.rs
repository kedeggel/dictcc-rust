extern crate csv;

use parse::ParseResult;

use parse::raw_csv::{get_csv_reader_from_path, incomplete_records_filter};
use parse::html::HtmlDecodedDictEntry;
use parse::word_ast::WordAST;

use parse::raw_csv::RawDictEntry;

pub fn parse_test() -> ParseResult<()> {
    let mut reader = get_csv_reader_from_path("../database/dictcc_DE-EN.txt")?;

    let records = reader
        .deserialize()
        .filter(incomplete_records_filter)
        .enumerate().take(10);

    for (i, record) in records {
        let raw_entry: RawDictEntry = record?;

        let html_decoded_entry = HtmlDecodedDictEntry::try_from(&raw_entry)?;

        eprintln!("html_decoded_entry = {:?}", html_decoded_entry);

        let word_ast = WordAST::try_from(&html_decoded_entry)?;

        eprintln!("word_ast = {:?}", word_ast);

        if i % 10000 == 0 {}
    }

    Ok(())
}