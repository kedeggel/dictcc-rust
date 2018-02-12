extern crate csv;

use dict::grouped::DictQueryResultGrouped;
use error::{DictError, DictResult};
use failure::Backtrace;
use parse::html::HtmlDecodedDictEntry;
use parse::raw_csv::{get_csv_reader_from_path, incomplete_records_filter, RawDictEntry};
use parse::word_ast::{WordNodes, WordNodesDictEntry};
use regex::{Captures, escape, Regex, RegexBuilder};
use std::fmt::{self, Display, Formatter};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::str::FromStr;

pub mod grouped;

/// Result of a translation query
#[derive(Clone, Eq, PartialEq, Debug)]
pub struct DictQueryResult {
    entries: Vec<DictEntry>,
    query_direction: QueryDirection,
}

impl DictQueryResult {
    /// Returns a slice of all entries found in the query.
    pub fn entries(&self) -> &[DictEntry] {
        &self.entries
    }

    /// Converts a `DictQueryResult` into a grouped representation used for structured display of the found entries.
    pub fn into_grouped(self) -> DictQueryResultGrouped {
        DictQueryResultGrouped::from(self)
    }
}

/// Structure that contains all dictionary entries
#[derive(Clone, Eq, PartialEq, Debug)]
pub struct Dict {
    /// List of all dictionary entries
    entries: Vec<DictEntry>,

    // Languages
    languages: DictLanguagePair,
}

impl Dict {
    /// Create a `Dict` from a database at `path`.
    ///
    /// Reads the csv, decodes HTML-encoded characters and parses the dict.cc bracket syntax into a AST.
    pub fn create<P: AsRef<Path>>(path: P) -> DictResult<Self> {
        info!("Using database path: {}", path.as_ref().display());

        let mut reader = get_csv_reader_from_path(&path)?;
        let languages = DictLanguagePair::from_path(&path)?;
        let records = reader
            .deserialize()
            .filter(incomplete_records_filter);

        let mut entries = vec![];

        for record in records {
            let raw_entry: RawDictEntry = record?;
            trace!("raw_entry = {:#?}", raw_entry);
            let html_decoded_entry = HtmlDecodedDictEntry::from(&raw_entry);
            trace!("html_decoded_entry = {:#?}", html_decoded_entry);
            let word_ast = WordNodesDictEntry::from(&html_decoded_entry);
            trace!("word_ast = {:#?}", word_ast);
            if let Ok(entry) = DictEntry::try_from(word_ast) {
                trace!("entry = {:#?}", entry);
                entries.push(entry);
            };
        }
        Ok(Self {
            entries,
            languages,
        })
    }

    /// Returns a slice of all entries in the `Dict`.
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
    pub fn get_language_pair(&self) -> &DictLanguagePair {
        &self.languages
    }

    /// Returns a `DictQuery` builder.
    pub fn query<'a, 'b>(&'a self, query_term: &'b str) -> DictQuery<'a, 'b> {
        DictQuery {
            dict: self,
            query_term,
            query_type: QueryType::Word,
            query_direction: QueryDirection::Bidirectional,
        }
    }
}


/// Builder for a `DictQueryResult`.
#[derive(Debug)]
pub struct DictQuery<'a, 'b> {
    dict: &'a Dict,
    query_term: &'b str,
    query_type: QueryType,
    query_direction: QueryDirection,
}

impl<'a, 'b> DictQuery<'a, 'b> {
    /// Set the query direction.
    pub fn set_direction(&mut self, query_direction: QueryDirection) -> &mut Self {
        self.query_direction = query_direction;
        self
    }

    /// Set the query type.
    pub fn set_type(&mut self, query_type: QueryType) -> &mut Self {
        self.query_type = query_type;
        self
    }

    /// Set the query term.
    pub fn set_term<'c>(self, query_term: &'c str) -> DictQuery<'a, 'c> {
        DictQuery {
            dict: self.dict,
            query_term,
            query_type: self.query_type,
            query_direction: self.query_direction,
        }
    }

    /// Sets the query direction based on the given source language.
    ///
    /// Convenience function for `set_query_direction`
    pub fn source_language(&mut self, source_language: &Language) -> DictResult<&mut Self> {
        let query_direction = self.dict.get_language_pair().infer_query_direction(source_language)?;
        self.set_direction(query_direction);
        Ok(self)
    }

    /// Every entry that contains the query-word is a hit (default!)
    ///
    /// Convenience function for `set_query_type`
    pub fn word(&mut self) -> &mut Self {
        self.set_type(QueryType::Word);
        self
    }

    /// Search for exact matches
    ///
    /// Convenience function for `set_query_type`
    pub fn exact(&mut self) -> &mut Self {
        self.set_type(QueryType::Exact);
        self
    }

    /// Search for regex, so the user can specify by himself what he wants to match
    ///
    /// Convenience function for `set_query_type`
    pub fn regex(&mut self) -> &mut Self {
        self.set_type(QueryType::Regex);
        self
    }

    /// Execute the query.
    pub fn execute(&self) -> DictResult<DictQueryResult> {
        let regexp = match self.query_type {
            QueryType::Word => RegexBuilder::new(&format!(r"(^|\s|-){}($|\s|-)", escape(self.query_term))).case_insensitive(true).build()?,
            QueryType::Exact => RegexBuilder::new(&format!(r"^{}$", escape(self.query_term))).case_insensitive(true).build()?,
            QueryType::Regex => RegexBuilder::new(&format!(r"^{}$", self.query_term)).case_insensitive(true).build()?,
        };

        Ok(DictQueryResult {
            entries: self.dict.entries.iter().filter(|entry| {
                match self.query_direction {
                    QueryDirection::ToRight => regexp.is_match(&entry.left_word.indexed_word),
                    QueryDirection::ToLeft => regexp.is_match(&entry.right_word.indexed_word),
                    QueryDirection::Bidirectional => regexp.is_match(&entry.left_word.indexed_word)
                        || regexp.is_match(&entry.right_word.indexed_word),
                }
            }).cloned().collect(),
            query_direction: self.query_direction,
        })
    }
}

/// Different types of queries. Used by `DictQuery`.
#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum QueryType {
    /// Search for exact matches
    Exact,
    /// Every entry that contains the query-word is a hit
    Regex,
    /// Search for regex, so the user can specify by himself what he wants to match
    Word,
}

impl FromStr for QueryType {
    type Err = DictError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use self::QueryType::*;

        Ok(match s.to_lowercase().as_str() {
            "e" | "exact" => Exact,
            "r" | "regex" => Regex,
            "w" | "word" => Word,
            unknown => Err(DictError::UnknownQueryType {
                query_type: unknown.to_string(),
                backtrace: Backtrace::new(),
            })?
        })
    }
}

/// In which direction a query is executed. Used by `DictQuery`.
/// Can be inferred by `DictLanguagePair::infer_query_direction`.
#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum QueryDirection {
    /// Search in the left language, to get results in the right language.
    ToRight,
    /// Search in the right language, to get results in the left language.
    ToLeft,
    /// Search in both languages.
    Bidirectional,
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

impl DictEntry {
    /// Try to convert from WordNodesDictEntry into DictEntry
    pub(crate) fn try_from(word_nodes_dict_entry: WordNodesDictEntry<String>) -> DictResult<Self> {
        let mut classes = Vec::new();
        for class in word_nodes_dict_entry.word_classes.split_whitespace() {
            classes.push(WordClass::try_from(class)?);
        }
        Ok(DictEntry {
            left_word: DictWord::try_from(word_nodes_dict_entry.left_word_nodes)?,
            right_word: DictWord::try_from(word_nodes_dict_entry.right_word_nodes)?,
            word_classes: classes,
        })
    }

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
    indexed_word: String,

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

impl DictWord {
    /// Try to convert from a WordNode into a DictWord
    fn try_from(word_nodes: WordNodes<String>) -> DictResult<Self> {
        Ok(DictWord {
            indexed_word: word_nodes.build_indexed_word(),
            word_count: word_nodes.count_words(),
            word_nodes,
        })
    }

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

/// Lists all available languages
#[derive(Clone, Eq, PartialEq, Debug)]
pub enum Language {
    /// Albanian
    SQ,
    /// Bosnian
    BS,
    /// Bulgarian
    BG,
    /// Croatioan
    HR,
    /// Czech
    CS,
    /// Danish
    DA,
    /// Dutch
    NL,
    /// English
    EN,
    /// Esperanto
    EO,
    /// Finnish
    FI,
    /// French
    FR,
    /// German
    DE,
    /// Greek
    EL,
    /// Hungarian
    HU,
    /// Icelandic
    IS,
    /// Italian
    IT,
    /// Latin
    LA,
    /// Norwegian
    NO,
    /// Polish
    PL,
    /// Portuguese
    PT,
    /// Romanian
    RO,
    /// Russian
    RU,
    /// Serbian
    SR,
    /// Slovak
    SK,
    /// Spanish
    ES,
    /// Swedish
    SV,
    /// Turkish
    TR,
    /// Other language that are not listed explicitly
    Other {
        /// The unknown language code.
        language_code: String
    },
}

impl FromStr for Language {
    type Err = DictError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use self::Language::*;
        if s.len() != 2 {
            return Err(DictError::InvalidLanguageCode { lang: s.to_string(), backtrace: Backtrace::new() });
        }
        Ok(match s.to_uppercase().as_str() {
            "SQ" => SQ,
            "BS" => BS,
            "BG" => BG,
            "HR" => HR,
            "CS" => CS,
            "DA" => DA,
            "NL" => NL,
            "EN" => EN,
            "EO" => EO,
            "FI" => FI,
            "FR" => FR,
            "DE" => DE,
            "EL" => EL,
            "HU" => HU,
            "IS" => IS,
            "IT" => IT,
            "LA" => LA,
            "NO" => NO,
            "PL" => PL,
            "PT" => PT,
            "RO" => RO,
            "RU" => RU,
            "SR" => SR,
            "SK" => SK,
            "ES" => ES,
            "SV" => SV,
            "TR" => TR,
            _ => Other { language_code: s.to_string() }
        })
    }
}

impl Display for Language {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        use self::Language::*;

        match *self {
            SQ => write!(f, "Albanian"),
            BS => write!(f, "Bosnian"),
            BG => write!(f, "Bulgarian"),
            HR => write!(f, "Croatioan"),
            CS => write!(f, "Czech"),
            DA => write!(f, "Danish"),
            NL => write!(f, "Dutch"),
            EN => write!(f, "English"),
            EO => write!(f, "Esperanto"),
            FI => write!(f, "Finnish"),
            FR => write!(f, "French"),
            DE => write!(f, "German"),
            EL => write!(f, "Greek"),
            HU => write!(f, "Hungarian"),
            IS => write!(f, "Icelandic"),
            IT => write!(f, "Italian"),
            LA => write!(f, "Latin"),
            NO => write!(f, "Norwegian"),
            PL => write!(f, "Polish"),
            PT => write!(f, "Portuguese"),
            RO => write!(f, "Romanian"),
            RU => write!(f, "Russian"),
            SR => write!(f, "Serbian"),
            SK => write!(f, "Slovak"),
            ES => write!(f, "Spanish"),
            SV => write!(f, "Swedish"),
            TR => write!(f, "Turkish"),
            Other { ref language_code } => write!(f, "Other Language: {}", language_code),
        }
    }
}

/// A pair of two languages. Identifies the languages of a single translation database file.
#[derive(Clone, Eq, PartialEq, Debug)]
pub struct DictLanguagePair {
    /// The left language in the database.
    pub left_language: Language,
    /// The right language in the database.
    pub right_language: Language,
}

impl DictLanguagePair {
    /// Infers the `QueryDirection` based on a given language.
    ///
    /// # Errors
    ///
    /// Returns `DictError::InvalidSourceLanguage`
    /// if `source_language` is not one of the two languages in `DictLanguagePair`.
    pub fn infer_query_direction(&self, source_language: &Language) -> DictResult<QueryDirection> {
        if *source_language == self.left_language {
            Ok(QueryDirection::ToRight)
        } else if *source_language == self.right_language {
            Ok(QueryDirection::ToLeft)
        } else {
            Err(DictError::InvalidSourceLanguage {
                source_language: source_language.clone(),
                backtrace: Backtrace::new(),
            })
        }
    }

    fn from_path<P: AsRef<Path>>(path: P) -> DictResult<DictLanguagePair> {
        let file = File::open(&path).map_err(|err| DictError::FileOpen {
            path: format!("{}", path.as_ref().display()),
            cause: csv::Error::from(err),
        })?;

        let mut header = String::new();
        let _ = BufReader::new(file).read_line(&mut header).map_err(|err| DictError::FileOpen {
            path: format!("{}", path.as_ref().display()),
            cause: csv::Error::from(err),
        })?;

        // Since the regex cannot be changed, unwrap is ok here
        let re = Regex::new("([A-Z]{2})-([A-Z]{2})").unwrap();
        let captures = |s| re.captures(s);
        let groups = match header.lines().next().and_then(captures) {
            Some(mat) => mat,
            None => return Err(DictError::LanguageCodeNotFound { backtrace: Backtrace::new() })
        };

        fn get_lang(idx: usize, captures: &Captures) -> DictResult<Language> {
            Language::from_str(captures.get(idx).
                ok_or(DictError::LanguageCodeNotFound { backtrace: Backtrace::new() })?.as_str())
        }

        Ok(DictLanguagePair {
            left_language: get_lang(1, &groups)?,
            right_language: get_lang(2, &groups)?,
        })
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
