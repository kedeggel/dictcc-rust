extern crate htmlescape;

use parse::raw_csv::RawDictEntry;

#[derive(Debug)]
pub struct HtmlDecodedDictEntry {
    pub left_word: String,
    pub right_word: String,
    pub word_classes: String,
}

impl<'a> From<&'a RawDictEntry> for HtmlDecodedDictEntry {
    fn from(raw: &RawDictEntry) -> Self {
        HtmlDecodedDictEntry {
            left_word: html_decode_with_fallback(&raw.left_word),
            right_word: html_decode_with_fallback(&raw.right_word),
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
