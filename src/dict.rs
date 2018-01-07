use std::str::FromStr;

use error::{DictError, DictResult};
use failure::Backtrace;

use parse::word_ast::{WordNode, WordAST};

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct DictEntry {
    pub source: DictWord,
    pub translation: DictWord,
    // TODO: Vec<WordClass> + parsing
    pub word_class: Option<WordClass>,
}

impl DictEntry {
    pub fn try_from(ast: &WordAST) -> DictResult<Self> {
        Ok(DictEntry {
            source: DictWord::try_from(&ast.source)?,
            translation: DictWord::try_from(&ast.translation)?,
            word_class: WordClass::try_from(ast.word_class)?,
        })
    }
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct DictWord {
    // FIXME: evaluate where the best place for the language tag is (space constraints and internal representation?)
    //pub language: Language,

    /// # Syntax:
    /// `<foo>`
    /// `<foo, bar>`
    ///
    /// # Indexing
    /// sorting: false
    /// keyword: true
    pub acronyms: Vec<String>,

    /// # Syntax:
    /// `{f}`
    /// `{m}`
    /// `{n}`
    /// `{pl}`
    /// `{sg}`
    ///
    /// # Indexing
    /// sorting: false
    /// keyword: false
    pub gender: Option<Gender>,

    /// # Syntax:
    /// `[foo]`
    ///
    /// # Indexing
    /// sorting: false
    /// keyword: false
    pub comment: String,

    /// The word with optional parts
    ///
    /// # Syntax:
    /// `(a) foo`
    ///
    /// sorting: true
    /// keyword: true
    pub complete_word: String,

    /// The word without optional parts
    ///
    /// # Syntax:
    /// `foo`
    pub plain_word: String,
}

impl DictWord {
    fn try_from<'a>(ast: &[WordNode<'a>]) -> Result<Self, DictError> {
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

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum Language {
    DE,
    EN,
    // ...
    Other { language_code: String },
}

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
    Noun
}

impl WordClass {
    pub fn try_from(s: &str) -> DictResult<Option<Self>> {
        match s {
            "" => Ok(None),
            s => Ok(Some(s.parse()?))
        }
    }
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