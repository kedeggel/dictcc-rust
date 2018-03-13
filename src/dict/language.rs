extern crate csv;

use error::{DictError, DictResult};
use failure::Backtrace;
use query::QueryDirection;
use regex::{Captures, Regex};
use std::fmt::{self, Display, Formatter};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::str::FromStr;


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

    pub(crate) fn from_path<P: AsRef<Path>>(path: P) -> DictResult<DictLanguagePair> {
        let file = File::open(&path).map_err(|err| DictError::FileOpen {
            path: format!("{}", path.as_ref().display()),
            cause: csv::Error::from(err),
            backtrace: Backtrace::new(),
        })?;

        let mut header = String::new();
        let _ = BufReader::new(file).read_line(&mut header).map_err(|err| DictError::FileOpen {
            path: format!("{}", path.as_ref().display()),
            cause: csv::Error::from(err),
            backtrace: Backtrace::new(),
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