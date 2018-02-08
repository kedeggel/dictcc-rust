//! Parsing and AST of the bracket syntax of dict.cc,
//! see [Guidelines](https://contribute.dict.cc/guidelines/)

extern crate nom;

use error::{DictError, DictResult};
use nom::GetInput;
use parse::html::HtmlDecodedDictEntry;
use std::borrow::Borrow;
use std::fmt;
use std::ops::Deref;
use std::string::ToString;

/// Parsing AST node
#[derive(Clone, Debug, PartialEq, Eq)]
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
            Word(s) => Word(s.to_string()),
            Angle(ref vec_s) => Angle(vec_s.iter().map(ToString::to_string).collect()),
            Round(s) => Round(s.to_string()),
            Square(s) => Square(s.to_string()),
            Curly(s) => Curly(s.to_string()),
        }
    }
}


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

impl<'a, T: Borrow<str>> WordNode<T> {
    fn to_colored_string(&self) -> String {
        use self::WordNode::*;

        use colored::Colorize;

        match *self {
            ref node @ Word(_) => {
                node.to_string()
            }
            ref node @ Angle(_) => {
                node.to_string().bright_red().to_string()
            }
            ref node @ Round(_) => {
                node.to_string().bright_green().to_string()
            }
            ref node @ Square(_) => {
                node.to_string().bright_blue().to_string()
            }
            ref node @ Curly(_) => {
                node.to_string().bright_cyan().to_string()
            }
        }
    }
}


/// "Newtype" struct of a `Vec<WordNode<T>>`.
/// Provides useful methods for extraction of parts of the word.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WordNodes<T: Borrow<str>> {
    nodes: Vec<WordNode<T>>
}

impl<'a, T: Borrow<str>> Deref for WordNodes<T> {
    type Target = [WordNode<T>];

    fn deref(&self) -> &Self::Target {
        &self.nodes
    }
}

impl<'a, 'b> From<&'a WordNodes<&'b str>> for WordNodes<String> {
    fn from(str_nodes: &'a WordNodes<&'b str>) -> Self {
        WordNodes {
            nodes: str_nodes.iter().map(|node| node.into()).collect()
        }
    }
}

impl<T: Borrow<str>> From<Vec<WordNode<T>>> for WordNodes<T> {
    fn from(nodes: Vec<WordNode<T>>) -> Self {
        WordNodes {
            nodes
        }
    }
}

impl<'a, T: Borrow<str>> fmt::Display for WordNodes<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let string = self.nodes.iter()
            .map(|word_node| word_node.to_string())
            .collect::<Vec<_>>()
            .join(" ");

        f.write_str(&string)
    }
}


impl<'a> WordNodes<&'a str> {
    /// Performs the conversion from str into WordNodes
    pub fn try_from(word: &'a str) -> DictResult<Self> {
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

        nom_res.to_full_result()
            .map(WordNodes::from)
            .map_err(|err| {
                DictError::WordASTParse { cause: err, word: word.to_string() }
            })
    }

    fn with_fallback_from(s: &'a str) -> Self {
        match WordNodes::try_from(s) {
            Ok(node) => node,
            Err(err) => {
                info!("Using WordNode fallback: {}", err);

                WordNodes {
                    nodes: vec![WordNode::Word(s)],
                }
            }
        }
    }
}

impl<T: Borrow<str>> WordNodes<T> {
    pub(crate) fn build_comments(&self) -> Vec<String> {
        use self::WordNode::*;

        self.nodes.iter()
            .filter_map(|node| {
                match *node {
                    Square(ref s) => Some(s.borrow().to_string()),
                    _ => None,
                }
            }).collect()
    }

    pub(crate) fn build_acronyms(&self) -> Vec<String> {
        use self::WordNode::*;

        self.nodes.iter()
            .filter_map(|node| {
                match *node {
                    Angle(ref vec_str) => Some(vec_str),
                    _ => None,
                }
            })
            .flat_map(|acronym_vec| acronym_vec.iter().map(|s| s.borrow().to_string()))
            .collect()
    }

    pub(crate) fn build_genders(&self) -> Vec<String> {
        use self::WordNode::*;

        self.nodes.iter().filter_map(|node| {
            match *node {
                Curly(ref s) => Some(s.borrow().to_string()),
                _ => None,
            }
        }).collect()
    }

    pub(crate) fn build_word_with_optional_parts(&self) -> String {
        use self::WordNode::*;

        self.nodes.iter().filter_map(|node| {
            match *node {
                ref node @ Word(_) |
                ref node @ Round(_) => {
                    Some(node.to_string())
                }
                _ => None,
            }
        }).collect::<Vec<_>>().join(" ")
    }

    pub(crate) fn build_plain_word(&self) -> String {
        use self::WordNode::*;

        self.nodes.iter().filter_map(|node| {
            match *node {
                Word(ref s) => {
                    Some(s.borrow().to_string())
                }
                _ => None,
            }
        }).collect::<Vec<_>>().join(" ")
    }

    pub(crate) fn build_indexed_word(&self) -> String {
        use self::WordNode::*;

        self.nodes.iter().filter_map(|node| {
            match *node {
                Word(ref s) | Round(ref s) => {
                    Some(s.borrow().to_string().to_lowercase())
                }
                _ => None,
            }
        }).collect::<Vec<_>>().join(" ")
    }

    pub(crate) fn count_words(&self) -> u8 {
        use self::WordNode::*;

        self.nodes.iter().filter(|&node| {
            match *node {
                Word(_) | Round(_) => true,
                _ => false,
            }
        }).count() as u8
    }

    pub(crate) fn to_colored_string(&self) -> String {
        self.nodes.iter()
            .map(|word_node| word_node.to_colored_string())
            .collect::<Vec<_>>()
            .join(" ")
    }
}

/// Word Abstract-Syntax-Tree
#[derive(Debug, PartialEq, Eq)]
pub struct WordNodesDictEntry<T: Borrow<str>> {
    /// Source word, parsed into WordNodes
    pub source: WordNodes<T>,
    /// Target word, parsed into WordNodes
    pub translation: WordNodes<T>,
    /// Simple str representation of word classes
    pub word_classes: T,
}

impl<'a, T: Borrow<str>> WordNodesDictEntry<T> {
    /// Try to convert from HtmlDecodedDictEntry into WordNodesDictEntry
    pub fn try_from(entry: &'a HtmlDecodedDictEntry) -> DictResult<WordNodesDictEntry<&'a str>> {
        Ok(WordNodesDictEntry {
            source: WordNodes::try_from(&entry.source)?,
            translation: WordNodes::try_from(&entry.translation)?,
            word_classes: &entry.word_classes,
        })
    }
}

impl<'a> From<&'a HtmlDecodedDictEntry> for WordNodesDictEntry<&'a str> {
    /// Perform conversion from HtmlDecodedDictEntry into WordNodesDictEntry<&str>.
    /// If word can't be parsed, a fallback representation of the word is used.
    fn from(entry: &'a HtmlDecodedDictEntry) -> WordNodesDictEntry<&'a str> {
        WordNodesDictEntry {
            source: WordNodes::with_fallback_from(&entry.source),
            translation: WordNodes::with_fallback_from(&entry.translation),
            word_classes: &entry.word_classes,
        }
    }
}

impl<'a> From<&'a HtmlDecodedDictEntry> for WordNodesDictEntry<String> {
    /// Perform conversion from HtmlDecodedDictEntry into WordNodesDictEntry<String>.
    /// If word can't be parsed, a fallback representation of the word is used.
    fn from(entry: &'a HtmlDecodedDictEntry) -> WordNodesDictEntry<String> {
        WordNodesDictEntry {
            source: WordNodes::from(&WordNodes::with_fallback_from(&entry.source)),
            translation: WordNodes::from(&WordNodes::with_fallback_from(&entry.translation)),
            word_classes: entry.word_classes.to_string(),
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
    use nom::IResult::*;
    use super::*;

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
        assert_eq!("<>", WordNode::Angle::<&str>(vec![]).to_string());
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
