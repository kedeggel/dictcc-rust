extern crate nom;

use error::ParseDictionaryError;

use parse::ParseResult;
use parse::html::HtmlDecodedDictEntry;

/// Parsing AST node
#[derive(Debug, PartialEq, Eq)]
pub enum WordNode<'a> {
    /// text at root
    Word(&'a str),
    /// abbreviations/acronyms
    ///
    /// sorting: false
    /// keyword: true
    /// multiple: true
    Angle(Vec<&'a str>),
    /// optional parts
    ///
    /// sorting: true
    /// keyword: true
    /// multiple: false
    Round(&'a str),
    /// visible comments
    ///
    /// sorting: false
    /// keyword: false
    /// multiple: false
    Square(&'a str),
    /// gender tags
    Curly(&'a str),
}

impl<'a> WordNode<'a> {
    pub fn try_from(s: &str) -> ParseResult<Vec<WordNode>> {
        let nom_res: nom::IResult<_, _> = entry(s);
        Ok(nom_res.to_full_result().map_err(|err| {
            ParseDictionaryError::WordASTParse { cause: err, word: s.to_string() }
        })?)
    }

    fn with_fallback_from(s: &str) -> Vec<WordNode> {
        match WordNode::try_from(s) {
            Ok(node) => node,
            Err(err) => {
                info!("Using WordNode fallback: {}", err);

                vec![WordNode::Word(s)]
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct WordAST<'a> {
    pub source: Vec<WordNode<'a>>,
    pub translation: Vec<WordNode<'a>>,
    pub word_class: &'a str,
}

impl<'a> WordAST<'a> {
    pub fn try_from(entry: &'a HtmlDecodedDictEntry) -> ParseResult<WordAST<'a>> {
        Ok(WordAST {
            source: WordNode::try_from(&entry.source)?,
            translation: WordNode::try_from(&entry.translation)?,
            word_class: &entry.word_class,
        })
    }
}

impl<'a> From<&'a HtmlDecodedDictEntry> for WordAST<'a> {
    fn from(entry: &'a HtmlDecodedDictEntry) -> WordAST<'a> {
        WordAST {
            source: WordNode::with_fallback_from(&entry.source),
            translation: WordNode::with_fallback_from(&entry.translation),
            word_class: &entry.word_class,
        }
    }
}


named!(csv<&str, Vec<&str> >, separated_list_complete!(
    tag_s!(", "),
    alt_complete!( take_until_s!(", ") | is_not_s!("") )
));

named!(word<&str, &str>, is_not_s!("([{< ") );
named!(angle_br<&str,Vec<&str> >,  flat_map!(delimited!(tag_s!("<"), is_not_s!(">"), tag_s!(">")), csv));
named!(round_br<&str,&str>,  delimited!(tag_s!("("), is_not_s!(")"), tag_s!(")")));
named!(square_br<&str,&str>, delimited!(tag_s!("["), is_not_s!("]"), tag_s!("]")));
named!(curly_br<&str,&str>,  delimited!(tag_s!("{"), is_not_s!("}"), tag_s!("}")));

named!(word_fragment<&str, WordNode>,   map!(word,      WordNode::Word));
named!(angle_fragment<&str, WordNode>,  map!(angle_br,  WordNode::Angle));
named!(round_fragment<&str, WordNode>,  map!(round_br,  WordNode::Round));
named!(square_fragment<&str, WordNode>, map!(square_br, WordNode::Square));
named!(curly_fragment<&str, WordNode>,  map!(curly_br,  WordNode::Curly));

named!(entry_fragment<&str,WordNode>, alt!(
    word_fragment |
    angle_fragment |
    round_fragment |
    square_fragment |
    curly_fragment
));

named!(entry<&str, Vec<WordNode> >, many1!( ws!( entry_fragment ) ));

#[cfg(test)]
mod tests {
    use super::*;
    use nom::IResult::*;

    #[test]
    fn test_entry() {
        let input = "(comment) word {f} [Foo.] <foo, bar, baz>";
        let expected = vec![
            WordNode::Round("comment"),
            WordNode::Word("word"),
            WordNode::Curly("f"),
            WordNode::Square("Foo."),
            WordNode::Angle(vec![
                "foo",
                "bar",
                "baz"
            ]),
        ];

        assert_eq!(Done("", expected), entry(input));
    }

    #[test]
    fn test_csv() {
        let data = vec![
            ("a", vec!["a"]),
            ("a, b", vec!["a", "b"]),
            ("foo", vec!["foo"]),
            ("foo, bar", vec!["foo", "bar"]),
            ("foo, bar, baz", vec!["foo", "bar", "baz"]),
            ("a, bar", vec!["a", "bar"]),
            ("foo, b", vec!["foo", "b"]),
            ("f,oo, ba,r, baz,", vec!["f,oo", "ba,r", "baz,"])
        ];

        for (input, expected) in data {
            assert_eq!(Done("", expected), csv(input));
        }
    }

    #[test]
    fn test_angle_br() {
        let data = vec![
            ("<a>", vec!["a"]),
            ("<a, b>", vec!["a", "b"]),
            ("<foo>", vec!["foo"]),
            ("<foo, bar>", vec!["foo", "bar"]),
            ("<foo, bar, baz>", vec!["foo", "bar", "baz"]),
            ("<a, bar>", vec!["a", "bar"]),
            ("<foo, b>", vec!["foo", "b"]),
            ("<f,oo, ba,r, baz,>", vec!["f,oo", "ba,r", "baz,"])
        ];

        for (input, expected) in data {
            assert_eq!(Done("", expected), angle_br(input));
        }
    }

    #[test]
    fn test_round_br() {
        assert_eq!(Done("", "foo"), round_br("(foo)"));
    }

    #[test]
    fn test_square_br() {
        assert_eq!(Done("", "foo"), square_br("[foo]"));
    }

    #[test]
    fn test_curly_br() {
        assert_eq!(Done("", "foo"), curly_br("{foo}"));
    }
}
