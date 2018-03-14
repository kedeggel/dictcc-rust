extern crate csv;

use error::{DictError, DictResult};
use failure::Backtrace;
use parse::word_ast::{WordNodes, WordNodesDictEntry};
use query::VecDictQuery;
use query::QueryDirection;
use query::QueryType;
use read::DictccReader;
use std::fmt::{self, Display, Formatter};
use std::path::Path;
use std::str::FromStr;

pub mod query;
pub mod read;
mod language;

pub use self::language::*;
use dict::sqlite::DictccMetadata;

#[cfg(feature = "sqlite")]
pub mod sqlite;

/// Structure that contains all dictionary entries
#[derive(Clone, Eq, PartialEq, Debug)]
pub struct VecDict {
    /// List of all dictionary entries
    entries: Vec<DictEntry>,

    // Languages
    languages: DictLanguagePair,
}

impl VecDict {
    /// Create a `VecDict` from a database at `path`.
    ///
    /// Reads the csv, decodes HTML-encoded characters and parses the dict.cc bracket syntax into a AST.
    pub fn create<P: AsRef<Path>>(path: P) -> DictResult<Self> {
        let (mut dict_reader, DictccMetadata {
            languages,
            ..
        }) = DictccReader::new(path)?;

        let entries: DictResult<Vec<DictEntry>> = dict_reader.entries().collect();

        Ok(Self {
            entries: entries?,
            languages,
        })
    }

    /// Returns a slice of all entries in the `VecDict`.
    pub fn get_entries(&self) -> &[DictEntry] {
        &self.entries
    }

    /// Return the left column's language of the dictionary file
    pub fn get_left_language(&self) -> &Language {
        &self.languages.left_language
    }

    /// Return the right column's language of the dictionary file
    pub fn get_right_language(&self) -> &Language {
        &self.languages.right_language
    }

    /// Return the language pair of the dictionary.
    fn language_pair(&self) -> &DictLanguagePair {
        &self.languages
    }

    /// Returns a `DictQuery` builder.
    pub fn query<'a, 'b>(&'a self, query_term: &'b str) -> VecDictQuery<'a, 'b> {
        VecDictQuery {
            dict: self,
            query_term,
            query_type: QueryType::Word,
            query_direction: QueryDirection::Bidirectional,
        }
    }
}


/// Structure that holds the word pair and it's class
#[derive(Clone, Eq, PartialEq, Debug)]
pub struct DictEntry {
    /// The word on the left side.
    pub left_word: DictWord,
    /// The word on the right side.
    pub right_word: DictWord,
    /// List of word classes (`noun`, `verb`, `adj`, etc.).
    pub word_classes: Vec<WordClass>,
}

impl From<WordNodesDictEntry<String>> for DictEntry {
    fn from(word_nodes_dict_entry: WordNodesDictEntry<String>) -> Self {
        DictEntry {
            left_word: DictWord::from(word_nodes_dict_entry.left_word_nodes),
            right_word: DictWord::from(word_nodes_dict_entry.right_word_nodes),
            word_classes: WordClass::with_fallback_from(&word_nodes_dict_entry.word_classes),
        }
    }
}

impl DictEntry {
    fn get_max_word_count(&self) -> u8 {
        use std::cmp::max;

        let source_word_count = self.left_word.word_count;
        let translation_word_count = self.right_word.word_count;

        max(source_word_count, translation_word_count)
    }
}

/// Structure that contains all fields of a dictionary entry from dict.cc
#[derive(Clone, Eq, PartialEq, Debug)]
pub struct DictWord {
    /// The word without the brackets of optional parts and in lowercase.
    /// Is used for searching and sorting.
    ///
    ///  Syntax:
    /// `(a) Foo` -> `a foo`
    pub indexed_word: String,

    /// The AST (abstract syntax tree) of the complete word.
    pub word_nodes: WordNodes<String>,

    /// The number of space separated words in this `DictWord`
    pub word_count: u8,
}

impl Display for DictWord {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.write_str(&self.word_nodes.to_string())
    }
}

impl From<WordNodes<String>> for DictWord {
    fn from(word_nodes: WordNodes<String>) -> Self {
        DictWord {
            indexed_word: word_nodes.build_indexed_word(),
            word_count: word_nodes.count_words(),
            word_nodes,
        }
    }
}

impl DictWord {
    fn to_colored_string(&self) -> String {
        self.word_nodes.to_colored_string()
    }

    // TODO: make a searchable keyword
    /// Syntax:
    /// `<foo>`
    /// `<foo, bar>`
    ///
    /// Indexing:
    /// not for sorting, but a keyword
    ///
    pub fn acronyms(&self) -> Vec<String> {
        self.word_nodes.build_acronyms()
    }

    /// Syntax:
    /// `[foo]`
    ///
    /// Indexing:
    /// not for sorting and not a keyword
    ///
    pub fn comments(&self) -> Vec<String> {
        self.word_nodes.build_comments()
    }

    /// Syntax:
    /// `{f}`
    /// `{m}`
    /// `{n}`
    /// `{pl}`
    /// `{sg}`
    ///
    /// Indexing:
    /// not for sorting and not a keyword
    ///
    pub fn genders(&self) -> Vec<String> {
        self.word_nodes.build_genders()
    }

    /// The word with optional parts
    ///
    /// Syntax:
    /// `(a) foo`
    ///
    /// Indexing:
    /// for sorting and a keyword
    ///
    pub fn word_with_optional_parts(&self) -> String {
        self.word_nodes.build_word_with_optional_parts()
    }

    /// The word with optional parts
    ///
    /// Syntax:
    /// `foo`
    ///
    pub fn plain_word(&self) -> String {
        self.word_nodes.build_plain_word()
    }
}


/// Lists all available genders
#[allow(missing_docs)]
#[derive(Clone, Eq, PartialEq, Debug)]
pub enum Gender {
    Feminine,
    Masculine,
    Neuter,
    Plural,
    Singular,
}

impl FromStr for Gender {
    type Err = DictError;

    /// Performs the fault-tolerant conversion from str into a Gender
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use self::Gender::*;

        Ok(match s.trim_right_matches('.') {
            "f" => Feminine,
            "m" => Masculine,
            "n" => Neuter,
            "pl" => Plural,
            "sg" => Singular,
            unknown => Err(DictError::UnknownGender { name: unknown.to_string(), backtrace: Backtrace::new() })?
        })
    }
}


/// Lists all available `WordClasses`
#[allow(missing_docs)]
#[derive(Clone, Copy, Eq, PartialEq, Debug, Ord, PartialOrd)]
pub enum WordClass {
    Adjective,
    Adverb,
    Past,
    Verb,
    PresentParticiple,
    Preposition,
    Conjunction,
    Pronoun,
    Prefix,
    Suffix,
    Noun,
}

impl WordClass {
    fn with_fallback_from(word_classes: &str) -> Vec<Self> {
        word_classes.split_whitespace().filter_map(|word_class_str| {
            match WordClass::try_from(word_class_str) {
                Ok(word_class) => Some(word_class),
                Err(err) => {
                    info!("Using WordClass fallback: {}", err);
                    None
                }
            }
        }).collect()
    }

    pub(crate) fn try_from(s: &str) -> DictResult<Self> {
        Ok(s.parse()?)
    }
}

impl FromStr for WordClass {
    type Err = DictError;

    /// Performs the fault-tolerant conversion from str into a WordClass
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use self::WordClass::*;

        Ok(match s.trim_right_matches('.') {
            "adj" => Adjective,
            "adv" => Adverb,
            "past-p" => Past,
            "verb" => Verb,
            "pres-p" => PresentParticiple,
            "prep" => Preposition,
            "conj" => Conjunction,
            "pron" => Pronoun,
            "prefix" => Prefix,
            "suffix" => Suffix,
            "noun" => Noun,
            unknown => return Err(DictError::UnknownWordClass { word_class: unknown.to_string(), backtrace: Backtrace::new() }),
        })
    }
}
