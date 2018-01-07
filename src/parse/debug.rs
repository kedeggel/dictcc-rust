extern crate csv;
extern crate failure;

use error::DictResult;
use parse::raw_csv::{get_csv_reader_from_path, incomplete_records_filter};
use parse::html::HtmlDecodedDictEntry;
use parse::word_ast::WordAST;
use parse::raw_csv::RawDictEntry;
use dict::DictEntry;

pub fn parse_test() -> DictResult<()> {
    let mut reader = get_csv_reader_from_path("../database/dictcc_DE-EN.txt")?;

    let records = reader
        .deserialize()
        .filter(incomplete_records_filter)
        .enumerate();

    let mut error_counter = 0;

    for (i, record) in records {
        let raw_entry: RawDictEntry = record?;

        let html_decoded_entry = HtmlDecodedDictEntry::from(&raw_entry);

        let word_ast = WordAST::from(&html_decoded_entry);

        match DictEntry::try_from(&word_ast) {
            Ok(dict_entry) => {
                if  i == 0 {
                    eprintln!("i = {:?}", i);
                    eprintln!("raw_entry = {:?}", raw_entry);
                    eprintln!("html_decoded_entry = {:?}", html_decoded_entry);
                    eprintln!("word_ast = {:?}", word_ast);
                    eprintln!("dict_entry = {:?}", dict_entry);
                }
            },
            Err(err) => {
                error_counter += 1;
                eprintln!("Error {}: {},\n index: {}, word_ast {:?}", error_counter, err, i, word_ast);
            }
        };



    }


    // pause for memory consumption monitoring
//    ::std::io::stdin().read_line(&mut String::new()).unwrap();

    Ok(())
}
