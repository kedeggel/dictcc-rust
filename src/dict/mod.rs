use std::str::FromStr;
use std::fmt::{self, Display, Formatter};
use std::path::Path;

use error::{DictError, DictResult};
use failure::Backtrace;
use parse::raw_csv::{get_csv_reader_from_path, incomplete_records_filter, RawDictEntry};
use parse::html::HtmlDecodedDictEntry;
use parse::word_ast::WordNodesDictEntry;
use regex::{escape, RegexBuilder};

use dict::grouped::DictQueryResultGrouped;
use parse::word_ast::WordNodes;

mod grouped;

/// Result of a translation query
#[derive(Clone, Eq, PartialEq, Debug)]
pub struct DictQueryResult {
    entries: Vec<DictEntry>,
}

impl DictQueryResult {
    pub fn get_results(&self) -> &[DictEntry] {
        &self.entries
    }

    pub fn into_grouped(self) -> DictQueryResultGrouped {
        self.into()
    }
}

/// Structure that contains all dictionary entries
#[derive(Clone, Eq, PartialEq, Debug)]
pub struct Dict {
    /// List of all dictionary entries
    entries: Vec<DictEntry>,
}

impl Dict {
    pub fn create<P: AsRef<Path>>(path: P) -> DictResult<Self> {
        let mut reader = get_csv_reader_from_path(path)?;

        let records = reader
            .deserialize()
            .filter(incomplete_records_filter);

        let mut entries = vec![];

        for record in records {
            let raw_entry: RawDictEntry = record?;
            let html_decoded_entry = HtmlDecodedDictEntry::from(&raw_entry);
            let word_ast = WordNodesDictEntry::from(&html_decoded_entry);
            if let Ok(entry) = DictEntry::try_from(word_ast) {
                entries.push(entry);
            };
        }
        Ok(Self {
            entries
        })
    }

    pub fn get_entries(&self) -> &[DictEntry] {
        &self.entries
    }

    pub fn query(&self) -> DictQuery {
        DictQuery {
            dict: &self,
            query_type: QueryType::Word,
            query_direction: QueryDirection::Bidirectional,
        }
    }
}

pub struct DictQuery<'a> {
    dict: &'a Dict,
    query_type: QueryType,
    query_direction: QueryDirection,
}

impl<'a> DictQuery<'a> {
    pub fn set_query_direction(&mut self, query_direction: QueryDirection) -> &Self {
        self.query_direction = query_direction;
        self
    }

    /// Every entry that contains the query-word is a hit (default!)
    pub fn word(&mut self) -> &Self {
        self.query_type = QueryType::Word;
        self
    }

    /// Search for exact matches
    pub fn exact(&mut self) -> &Self {
        self.query_type = QueryType::Exact;
        self
    }

    /// Search for regex, so the user can specify by himself what he wants to match
    pub fn regex(&mut self) -> &Self {
        self.query_type = QueryType::Regex;
        self
    }

    pub fn query(&self, query: &str) -> DictQueryResult {
        let regexp = match self.query_type {
            // TODO: remove unwrap
            QueryType::Word => RegexBuilder::new(&format!(r"(^|\s|-){}($|\s|-)", escape(query))).case_insensitive(true).build().unwrap(),
            QueryType::Exact => RegexBuilder::new(&format!(r"^{}$", escape(query))).case_insensitive(true).build().unwrap(),
            QueryType::Regex => RegexBuilder::new(&format!(r"^{}$", query)).case_insensitive(true).build().unwrap(),
        };

        DictQueryResult {
            entries: self.dict.entries.iter().filter(|entry| {
                match self.query_direction {
                    QueryDirection::ToRight => regexp.is_match(&entry.source.indexed_word),
                    QueryDirection::ToLeft => regexp.is_match(&entry.translation.indexed_word),
                    QueryDirection::Bidirectional => regexp.is_match(&entry.source.indexed_word)
                        || regexp.is_match(&entry.translation.indexed_word),
                }
            }).cloned().collect(),
        }
    }
}

enum QueryType {
    Exact,
    Regex,
    Word,
}

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum QueryDirection {
    ToRight,
    ToLeft,
    Bidirectional,
}

/// Structure that holds the word pair and it's class
#[derive(Clone, Eq, PartialEq, Debug)]
pub struct DictEntry {
    /// Source word
    pub source: DictWord,
    /// Target word
    pub translation: DictWord,
    /// List of word classes (`noun`, `verb`, `adj`, etc.)
    pub word_classes: Vec<WordClass>,
}

impl DictEntry {
    /// Try to convert from WordNodesDictEntry into DictEntry
    pub fn try_from(word_nodes_dict_entry: WordNodesDictEntry<String>) -> DictResult<Self> {
        let mut classes = Vec::new();
        for class in word_nodes_dict_entry.word_classes.split_whitespace() {
            classes.push(WordClass::try_from(class)?);
        }
        Ok(DictEntry {
            source: DictWord::try_from(word_nodes_dict_entry.source)?,
            translation: DictWord::try_from(word_nodes_dict_entry.translation)?,
            word_classes: classes,
        })
    }

    fn get_max_word_count(&self) -> u8 {
        use std::cmp::max;

        let source_word_count = self.source.word_count;
        let translation_word_count = self.translation.word_count;

        max(source_word_count, translation_word_count)
    }
}

/// Structure that contains all fields of a dictionary entry from dict.cc
#[derive(Clone, Eq, PartialEq, Debug)]
pub struct DictWord {
// FIXME: evaluate where the best place for the language tag is (space constraints and internal representation?)
//pub language: Language,

    // TODO: make a searchable keyword
    /// Syntax:
    /// `<foo>`
    /// `<foo, bar>`
    ///
    /// Indexing:
    /// not for sorting, but a keyword
    ///
//    pub acronyms: Vec<String>,
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
//    pub gender: Option<Gender>,
    /// Syntax:
    /// `[foo]`
    ///
    /// Indexing:
    /// not for sorting and not a keyword
    ///
//    pub comment: String,
    /// The word with optional parts
    ///
    /// Syntax:
    /// `(a) foo`
    ///
    /// Indexing:
    /// for sorting and a keyword
    ///
//    pub word: String,
    /// The word without the brackets of optional parts and in lowercase.
    /// Is used for searching and sorting.
    ///
    ///  Syntax:
    /// `(a) Foo` -> `a foo`
    indexed_word: String,

    // TODO:
    // API for comment/acronyms/gender and remove existing fields
    // Display
    /// The AST (abstract syntax tree) of the complete word.
    word_nodes: WordNodes<String>,

    /// The number of space separated words in this `DictWord`
    pub word_count: u8,
}

impl Display for DictWord {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.write_str(&self.word_nodes.to_string())
    }
}

impl DictWord {
    /// Try to convert from a WordNode into a DictWord
    fn try_from(word_nodes: WordNodes<String>) -> DictResult<Self> {
        Ok(DictWord {
            indexed_word: word_nodes.build_indexed_word(),
            word_count: word_nodes.count_words(),
            word_nodes,
        })
    }
}

/// Lists all available languages
#[derive(Clone, Eq, PartialEq, Debug)]
pub enum Language {
    /// German
    DE,
    /// English
    EN,
    // TODO: List all available languages
    /// Other language that are not listed explicitly
    Other { language_code: String },
}

impl FromStr for Language {
    type Err = DictError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use self::Language::*;
        if s.len() != 2 {
            return Err(DictError::InvalidLanguageCode { lang: s.to_string(), backtrace: Backtrace::new() });
        }
        Ok(match s {
            "DE" => DE,
            "EN" => EN,
            // ...
            _ => Other { language_code: s.to_string() }
        })
    }
}

/// Lists all available genders
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


/// Lists all available WordClasses
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
    pub fn try_from(s: &str) -> DictResult<Self> {
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
