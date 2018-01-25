extern crate nom;

use error::{DictResult, DictError};
use parse::html::HtmlDecodedDictEntry;
use nom::GetInput;
use std::string::ToString;
use std::borrow::Borrow;

/// Parsing AST node
#[derive(Debug, PartialEq, Eq)]
pub enum WordNode<T: Borrow<str>> {
    /// text at root
    Word(T),
    /// abbreviations/acronyms
    Angle(Vec<T>),
    /// optional parts
    Round(T),
    /// visible comments
    Square(T),
    /// gender tags
    Curly(T),
}

impl<'a, 'b> From<&'a WordNode<&'b str>> for WordNode<String> {
    fn from(str_node: &'a WordNode<&'b str>) -> Self {
        use self::WordNode::*;
        use std::string::ToString;

        match *str_node {
            Word(ref s) => Word(s.to_string()),
            Angle(ref vec_s) => Angle(vec_s.iter().map(ToString::to_string).collect()),
            Round(ref s) => Round(s.to_string()),
            Square(ref s) => Square(s.to_string()),
            Curly(ref s) => Curly(s.to_string()),
        }
    }
}

impl<'a> WordNode<&'a str> {
    /// Performs the conversion from str into WordNode
    pub fn try_from(word: &'a str) -> DictResult<Vec<Self>> {
        let nom_res: nom::IResult<_, _> = entry(word);

        match nom_res.remaining_input() {
            Some("") | None => {}
            Some(remaining_input) => {
                return Err(DictError::WordASTRemainingInput {
                    word: word.to_string(),
                    remaining_input: remaining_input.to_string(),
                });
            }
        }

        nom_res.to_full_result().map_err(|err| {
            DictError::WordASTParse { cause: err, word: word.to_string() }
        }).into()
    }

    pub fn with_fallback_from(s: &'a str) -> Vec<Self> {
        match WordNode::try_from(s) {
            Ok(node) => node,
            Err(err) => {
                info!("Using WordNode fallback: {}", err);

                vec![WordNode::Word(s)]
            }
        }
    }
}

impl<T: Borrow<str>> WordNode<T> {
    // FIXME: handle multiple comments/acronyms/gender blocks consistently

    pub fn build_comment_string(ast: &[Self]) -> String {
        use self::WordNode::*;

        ast.iter()
            .filter_map(|node| {
                match *node {
                    Square(ref s) => Some(s.borrow().to_string()),
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
            .flat_map(|foo| foo.iter().map(|s| s.borrow().to_string()))
            .collect()
    }

    pub fn build_gender_tag_string(ast: &[Self]) -> Option<String> {
        use self::WordNode::*;

        ast.iter().filter_map(|node| {
            match *node {
                Curly(ref s) => Some(s.borrow().to_string()),
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

    pub fn build_indexed_word(ast: &[Self]) -> String {
        use self::WordNode::*;

        ast.iter().filter_map(|node| {
            match *node {
                Word(ref s) | Round(ref s) => {
                    Some(s.borrow().to_string().to_lowercase())
                }
                _ => None,
            }
        }).collect::<Vec<_>>().join(" ")
    }

    pub fn count_words(ast: &[Self]) -> u8 {
        use self::WordNode::*;

        ast.iter().filter(|node| {
            match *node {
                &Word(_) | &Round(_) => true,
                _ => false,
            }
        }).count() as u8
    }
}

use std::fmt;

impl<'a, T: Borrow<str>> fmt::Display for WordNode<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::WordNode::*;

        match *self {
            Word(ref s) => {
                write!(f, "{}", s.borrow())
            }
            Angle(ref vec_s) => {
                write!(f, "<{}>", vec_s.join(", "))
            }
            Round(ref s) => {
                write!(f, "({})", s.borrow())
            }
            Square(ref s) => {
                write!(f, "[{}]", s.borrow())
            }
            Curly(ref s) => {
                write!(f, "{{{}}}", s.borrow())
            }
        }
    }
}

/// Word Abstract-Syntax-Tree
#[derive(Debug, PartialEq, Eq)]
pub struct ASTDictEntry<T: Borrow<str>> {
    /// Source word, parsed into WordNodes
    pub source: Vec<WordNode<T>>,
    /// Target word, parsed into WordNodes
    pub translation: Vec<WordNode<T>>,
    /// Simple str representation of word classes
    pub word_classes: T,
}

impl<'a, T: Borrow<str>> ASTDictEntry<T> {
    /// Try to convert from HtmlDecodedDictEntry into ASTDictEntry
    pub fn try_from(entry: &'a HtmlDecodedDictEntry) -> DictResult<ASTDictEntry<&'a str>> {
        Ok(ASTDictEntry {
            source: WordNode::try_from(&entry.source)?,
            translation: WordNode::try_from(&entry.translation)?,
            word_classes: &entry.word_classes,
        })
    }
}

impl<'a> From<&'a HtmlDecodedDictEntry> for ASTDictEntry<&'a str> {
    /// Perform conversion from HtmlDecodedDictEntry into ASTDictEntry.
    /// If word can't be parsed, a fallback representation of the word is returned.
    fn from(entry: &'a HtmlDecodedDictEntry) -> ASTDictEntry<&'a str> {
        ASTDictEntry {
            source: WordNode::with_fallback_from(&entry.source),
            translation: WordNode::with_fallback_from(&entry.translation),
            word_classes: &entry.word_classes,
        }
    }
}


named!(csv<&str, Vec<&str> >, separated_list_complete!(
    tag_s!(", "),
    alt_complete!( take_until_s!(", ") | is_not_s!("") )
));

named!(word<&str, &str>, is_not_s!("([{< ") );
named!(angle_br<&str,Vec<&str> >,  flat_map!(delimited!(tag_s!("<"), is_not_s!("<>"), tag_s!(">")), csv));
named!(round_br<&str,&str>,  delimited!(tag_s!("("), is_not_s!("()"), tag_s!(")")));
named!(square_br<&str,&str>, delimited!(tag_s!("["), is_not_s!("[]"), tag_s!("]")));
named!(curly_br<&str,&str>,  delimited!(tag_s!("{"), is_not_s!("{}"), tag_s!("}")));

named!(word_fragment<&str, WordNode<&str>>,   map!(word,      WordNode::Word));
named!(angle_fragment<&str, WordNode<&str>>,  map!(angle_br,  WordNode::Angle));
named!(round_fragment<&str, WordNode<&str>>,  map!(round_br,  WordNode::Round));
named!(square_fragment<&str, WordNode<&str>>, map!(square_br, WordNode::Square));
named!(curly_fragment<&str, WordNode<&str>>,  map!(curly_br,  WordNode::Curly));

named!(entry_fragment<&str,WordNode<&str>>, alt!(
    word_fragment |
    angle_fragment |
    round_fragment |
    square_fragment |
    curly_fragment
));

named!(entry<&str, Vec<WordNode<&str>> >, many1!( ws!( entry_fragment ) ));

#[cfg(test)]
mod tests {
    use super::*;
    use nom::IResult::*;

    #[test]
    fn test_entry_parser() {
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
