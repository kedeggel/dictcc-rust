use std::str::FromStr;

use error::{DictError};
use failure::Backtrace;

pub struct DictEntry {
    pub source: DictWord,
    pub translation: DictWord,
    pub word_class: WordClass,
}

pub struct DictWord {
    pub language: Language,

    /// # Syntax:
    /// `<foo>`
    /// `<foo, bar>`
    pub acronyms: Vec<DictWord>,

    /// # Syntax:
    /// `{f}`
    /// `{m}`
    /// `{n}`
    /// `{pl}`
    /// `{sg}`
    pub gender: Option<Gender>,

    /// The word with comments
    pub word: String,

    /// The word stripped of comments for sorting
    pub word_without_comments: String,
}

pub enum Language {
    DE,
    EN,
    // ...
    Other { language_code: String },
}

pub enum Gender {
    Feminine,
    Masculine,
    Neuter,
    Plural,
    Singular,
}

impl FromStr for Gender {
    type Err = DictError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use self::Gender::*;

        Ok(match s {
            "f" => Feminine,
            "m" => Masculine,
            "n" => Neuter,
            "pl" => Plural,
            "sg" => Singular,
            unknown => Err(DictError::UnknownGender { name: unknown.to_string(), backtrace: Backtrace::new() })?
        })
    }
}


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
    Undefined,
}

impl FromStr for WordClass {
    type Err = DictError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use self::WordClass::*;

        Ok(match s {
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