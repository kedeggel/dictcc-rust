extern crate htmlescape;

use super::ParseResult;

pub fn html_decode(input: &str) -> ParseResult<String> {
    Ok(htmlescape::decode_html(input)?)
}