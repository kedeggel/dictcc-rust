extern crate htmlescape;

use parse::raw_csv::RawDictEntry;

// TODO: use &str to avoid cloning
#[derive(Debug)]
pub struct HtmlDecodedDictEntry {
    pub source: String,
    pub translation: String,
    pub word_classes: String,
}

impl<'a> From<&'a RawDictEntry> for HtmlDecodedDictEntry {
    fn from(raw: &RawDictEntry) -> Self {
        HtmlDecodedDictEntry {
            source: html_decode_with_fallback(&raw.source),
            translation: html_decode_with_fallback(&raw.translation),
            word_classes: raw.word_classes.clone(),
        }
    }
}

fn html_decode_with_fallback(input: &str) -> String {
    match htmlescape::decode_html(input) {
        Ok(decoded) => decoded,
        Err(err) => {
            info!("Using HTML-Decode fallback for {}: {:?}", input, err);
            input.to_string()
        }
    }

}
