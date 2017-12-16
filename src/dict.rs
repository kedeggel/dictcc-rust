use std::str::FromStr;

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

    // TODO: evaluate if needed
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
    type Err = ();

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
            // FIXME error handling
            _ => return Err(()),
        })
    }
}