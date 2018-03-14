use csv::{DeserializeRecordsIter, Reader};
use dict::DictEntry;
use dict::DictLanguagePair;
use error::DictResult;
use parse::html::HtmlDecodedDictEntry;
use parse::raw_csv::get_csv_reader_from_path;
use parse::raw_csv::incomplete_records_filter;
use parse::raw_csv::RawDictEntry;
use parse::word_ast::WordNodesDictEntry;
use std::fs::File;
use std::path::Path;
use dunce::canonicalize;
use dict::sqlite::DictccMetadata;


#[derive(Debug)]
pub struct DictccReader {
    reader: Reader<File>,
}

impl DictccReader {
    pub fn new<P: AsRef<Path>>(dictcc_db_path: P) -> DictResult<(Self, DictccMetadata)> {
        info!("Using database path: {}", dictcc_db_path.as_ref().display());

        Ok((DictccReader {
            reader: get_csv_reader_from_path(&dictcc_db_path)?,
        }, DictccMetadata::new(DictLanguagePair::from_path(&dictcc_db_path)?,
                               canonicalize(dictcc_db_path)?, )))
    }

    pub fn entries<'r>(&'r mut self) -> Entries<'r> {
        let records = self.reader.deserialize();

        Entries {
            records,
        }
    }

    pub fn raw_entries<'r>(&'r mut self) -> Box<Iterator<Item=DictResult<RawDictEntry>> + 'r> {
        Box::new(self.reader.deserialize()
            .filter(incomplete_records_filter)
            .map(|record| Ok(record?)))
    }
}

#[allow(missing_debug_implementations)]
pub struct Entries<'r> {
    records: DeserializeRecordsIter<'r, File, RawDictEntry>,
}

impl<'r> Iterator for Entries<'r> {
    type Item = DictResult<DictEntry>;

    fn next(&mut self) -> Option<Self::Item> {
        self.records.find(incomplete_records_filter)
            .map(|record| {
                let raw_entry: RawDictEntry = record?;
                trace!("raw_entry = {:#?}", raw_entry);
                let html_decoded_entry = HtmlDecodedDictEntry::from(&raw_entry);
                trace!("html_decoded_entry = {:#?}", html_decoded_entry);
                let word_ast = WordNodesDictEntry::from(&html_decoded_entry);
                trace!("word_ast = {:#?}", word_ast);
                let entry = DictEntry::from(word_ast);
                trace!("entry = {:#?}", entry);
                Ok(entry)
            })
    }
}