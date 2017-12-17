extern crate htmlescape;

use super::ParseResult;
use super::raw_csv::RawDictEntry;

#[derive(Debug)]
pub struct HtmlDecodedDictEntry {
    pub source: String,
    pub translation: String,
    pub word_class: String,
}

impl HtmlDecodedDictEntry {
    pub fn try_from(raw: &RawDictEntry) -> ParseResult<HtmlDecodedDictEntry> {
        Ok(HtmlDecodedDictEntry {
            source: html_decode(&raw.source)?,
            translation: html_decode(&raw.translation)?,
            word_class: raw.word_class.clone(),
        })
    }
}

fn html_decode(input: &str) -> ParseResult<String> {
    Ok(htmlescape::decode_html(input)?)
}