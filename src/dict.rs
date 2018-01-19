use std::str::FromStr;
use std::fmt::{self, Display, Formatter};
use std::path::{PathBuf, Path};

use error::{DictError, DictResult};
use failure::Backtrace;
use parse::raw_csv::{get_csv_reader_from_path, incomplete_records_filter, RawDictEntry};
use parse::html::HtmlDecodedDictEntry;
use parse::word_ast::{WordNode, WordAST};
use regex::{escape, RegexBuilder};

/// Result of a translation query
#[derive(Clone, Eq, PartialEq, Debug)]
pub struct DictQueryResult {
    entries: Vec<DictEntry>
}

impl DictQueryResult {
    pub fn get_results(&self) -> &Vec<DictEntry> {
        &self.entries
    }
}

impl Display for DictQueryResult {
    fn fmt(&self, _f: &mut Formatter) -> fmt::Result {
        unimplemented!()
    }
}

/// A configurable builder for a dictionary
#[derive(Clone, Eq, PartialEq, Debug)]
pub struct DictBuilder {
    /// Path where the dictionary database is stored
    path: Option<PathBuf>
}

impl DictBuilder {
    /// Create a new DictBuilder
    pub fn new() -> Self {
        DictBuilder {
            path: None,
        }
    }

    /// Configure the path where the dictionary database is stored
    pub fn path<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.path = Some(path.as_ref().to_owned());
        self
    }

    /// Consume the builder and create the dictionary
    pub fn build(self) -> DictResult<Dict> {
        if self.path.is_none() {
            return Err(DictError::NoPathSet);
        }

        let mut reader = get_csv_reader_from_path(self.path.unwrap())?;

        let records = reader
            .deserialize()
            .filter(incomplete_records_filter);

        let mut entries = vec![];

        for record in records {
            let raw_entry: RawDictEntry = record?;
            let html_decoded_entry = HtmlDecodedDictEntry::from(&raw_entry);
            let word_ast = WordAST::from(&html_decoded_entry);
            if let Ok(entry) = DictEntry::try_from(&word_ast) {
                entries.push(entry);
            };
        }
        Ok(Dict {
            entries
        })
    }
}

/// Structure that contains all dictionary entries
#[derive(Clone, Eq, PartialEq, Debug)]
pub struct Dict {
    /// List of all dictionary entries
    entries: Vec<DictEntry>,

}

impl Dict {
    pub fn get_entries(&self) -> &Vec<DictEntry> {
        &self.entries
    }
}

pub struct DictQuery<'a> {
    dict: &'a Dict,
    query_type: QueryType,
    query_direction: QueryDirection,
}

impl<'a> DictQuery<'a> {
    pub fn new(dict: &'a Dict) -> Self {
        Self {
            dict,
            query_type: QueryType::Word,
            query_direction: QueryDirection::Bidirectional,
        }
    }

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
            QueryType::Word => RegexBuilder::new(&format!(r"^{}$", escape(query))).case_insensitive(true).build().unwrap(),
            QueryType::Exact => RegexBuilder::new(&format!(r"^{}($|\s|-)", escape(query))).case_insensitive(true).build().unwrap(),
            QueryType::Regex => RegexBuilder::new(&format!(r"^{}$", query)).case_insensitive(true).build().unwrap(),
        };

        DictQueryResult {
            entries: self.dict.entries.to_owned().into_iter().filter(|entry| {
                println!("{:?}", self.query_direction);
                match self.query_direction {
                    QueryDirection::ToRight => regexp.is_match(&entry.source.plain_word.to_lowercase()),
                    QueryDirection::ToLeft => regexp.is_match(&entry.translation.plain_word.to_lowercase()),
                    QueryDirection::Bidirectional => regexp.is_match(&entry.source.plain_word.to_lowercase())
                        || regexp.is_match(&entry.translation.plain_word.to_lowercase()),
                }
            }).collect()
        }
    }
}

enum QueryType {
    Exact,
    Regex,
    Word,
}

#[derive(Debug)]
pub enum QueryDirection {
    ToRight,
    ToLeft,
    Bidirectional,
}

/// Structure that holds the word pair and it'S class
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
    /// Try to convert from WordAST into DictEntry
    pub fn try_from(ast: &WordAST) -> DictResult<Self> {
        let mut classes = Vec::new();
        for class in ast.word_classes.split_whitespace() {
            classes.push(WordClass::try_from(class)?);
        }
        Ok(DictEntry {
            source: DictWord::try_from(&ast.source)?,
            translation: DictWord::try_from(&ast.translation)?,
            word_classes: classes,
        })
    }
}

/// Structure that contains all fields of a dictionary entry from dict.cc
#[derive(Clone, Eq, PartialEq, Debug)]
pub struct DictWord {
// FIXME: evaluate where the best place for the language tag is (space constraints and internal representation?)
//pub language: Language,

    /// Syntax:
    /// `<foo>`
    /// `<foo, bar>`
    ///
// Indexing
// sorting: false
// keyword: true
    pub acronyms: Vec<String>,
    /// Syntax:
    /// `{f}`
    /// `{m}`
    /// `{n}`
    /// `{pl}`
    /// `{sg}`
// Indexing
// sorting: false
// keyword: false
    pub gender: Option<Gender>,
    /// Syntax:
    /// `[foo]`
    ///
// Indexing
// sorting: false
// keyword: false
    pub comment: String,
    /// The word with optional parts
    ///
    /// Syntax:
    /// `(a) foo`
    ///
    /// sorting: true
    /// keyword: true
    pub complete_word: String,
    /// The word without optional parts
    ///
    ///  Syntax:
    /// `foo`
    pub plain_word: String,
}

impl DictWord {
    /// Try to convert from a WordNode into a DictWord
    fn try_from<'a>(ast: &[WordNode<'a>]) -> DictResult<Self> {
        let gender = match WordNode::build_gender_tag_string(&ast) {
            Some(gender_string) => Some(gender_string.parse()?),
            None => None,
        };

        Ok(DictWord {
            acronyms: WordNode::build_acronyms_vec(&ast),
            gender,
            comment: WordNode::build_comment_string(&ast),
            complete_word: WordNode::build_word_with_optional_parts(&ast),
            plain_word: WordNode::build_word_without_optional_parts(&ast),
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
#[derive(Clone, Eq, PartialEq, Debug)]
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
