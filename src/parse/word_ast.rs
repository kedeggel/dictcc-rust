extern crate nom;

use error::DictError;
use error::DictResult;
use parse::html::HtmlDecodedDictEntry;

/// Parsing AST node
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum WordNode<'a> {
    /// text at root
    Word(&'a str),
    /// abbreviations/acronyms
    Angle(Vec<&'a str>),
    /// optional parts
    Round(&'a str),
    /// visible comments
    Square(&'a str),
    /// gender tags
    Curly(&'a str),
}

impl<'a> WordNode<'a> {
    pub fn try_from(s: &str) -> DictResult<Vec<WordNode>> {
        let nom_res: nom::IResult<_, _> = entry(s);
        Ok(nom_res.to_full_result().map_err(|err| {
            DictError::WordASTParse { cause: err, word: s.to_string() }
        })?)
    }

    pub fn with_fallback_from(s: &str) -> Vec<WordNode> {
        match WordNode::try_from(s) {
            Ok(node) => node,
            Err(err) => {
                info!("Using WordNode fallback: {}", err);

                vec![WordNode::Word(s)]
            }
        }
    }

    // FIXME: handle multiple comments/acronyms/gender blocks consistently

    pub fn build_comment_string(ast: &[Self]) -> String {
        use self::WordNode::*;

        ast.iter()
            .filter_map(|node| {
                match *node {
                    Square(ref s) => Some(s.to_string()),
                    _ => None,
                }
            }).collect()
    }

    pub fn build_acronyms_vec(ast: &[Self]) -> Vec<String> {
        use self::WordNode::*;

        ast.iter()
            .filter_map(|node| {
                match *node {
                    Angle(ref vec_str) => Some(vec_str),
                    _ => None,
                }
            })
            .flat_map(|foo| foo.iter().map(|s| s.to_string()))
            .collect()
    }

    pub fn build_gender_tag_string(ast: &[Self]) -> Option<String> {
        use self::WordNode::*;

        ast.iter().filter_map(|node| {
            match *node {
                Curly(ref s) => Some(s.to_string()),
                _ => None,
            }
        }).next()
    }

    pub fn build_word_with_optional_parts(ast: &[Self]) -> String {
        use self::WordNode::*;

        ast.iter().filter_map(|node| {
            match *node {
                ref node @ Word(_) => {
                    Some(node.to_string())
                }
                ref node @ Round(_) => {
                    Some(node.to_string())
                }
                _ => None,
            }
        }).collect::<Vec<_>>().join(" ")
    }

    pub fn build_word_without_optional_parts(ast: &[Self]) -> String {
        use self::WordNode::*;

        ast.iter().filter_map(|node| {
            match *node {
                ref node @ Word(_) => {
                    Some(node.to_string())
                }
                _ => None,
            }
        }).collect::<Vec<_>>().join(" ")
    }
}

use std::fmt;

impl<'a> fmt::Display for WordNode<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::WordNode::*;

        match *self {
            Word(ref s) => {
                write!(f, "{}", s)
            }
            Angle(ref vec_s) => {
                write!(f, "<{}>", vec_s.join(", "))
            }
            Round(ref s) => {
                write!(f, "({})", s)
            }
            Square(ref s) => {
                write!(f, "[{}]", s)
            }
            Curly(ref s) => {
                write!(f, "{{{}}}", s)
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct WordAST<'a> {
    pub source: Vec<WordNode<'a>>,
    pub translation: Vec<WordNode<'a>>,
    pub word_class: &'a str,
}

impl<'a> WordAST<'a> {
    pub fn try_from(entry: &'a HtmlDecodedDictEntry) -> DictResult<WordAST<'a>> {
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
    fn test_word_node() {
        let input = "(optional) word {f} [comment] <foo, bar, baz>";
        let expected = vec![
            WordNode::Round("optional"),
            WordNode::Word("word"),
            WordNode::Curly("f"),
            WordNode::Square("comment"),
            WordNode::Angle(vec![
                "foo",
                "bar",
                "baz"
            ]),
        ];

        assert_eq!(Done("", expected), entry(input));
    }

    #[test]
    fn test_word_node_display() {
        assert_eq!("foo", WordNode::Word("foo").to_string());
        assert_eq!("<>", WordNode::Angle(vec![]).to_string());
        assert_eq!("<foo>", WordNode::Angle(vec!["foo"]).to_string());
        assert_eq!("<foo, bar>", WordNode::Angle(vec!["foo", "bar"]).to_string());
        assert_eq!("(foo)", WordNode::Round("foo").to_string());
        assert_eq!("[foo]", WordNode::Square("foo").to_string());
        assert_eq!("{foo}", WordNode::Curly("foo").to_string());
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
