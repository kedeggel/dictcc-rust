extern crate dictcc;
extern crate structopt;
#[macro_use]
extern crate structopt_derive;
extern crate app_dirs;
extern crate toml;
#[macro_use]
extern crate failure;
#[macro_use]
extern crate serde_derive;
extern crate colored;

use app_dirs::*;
use dictcc::dict::{Dict, Language, QueryType};
use error::{DictCliError, DictCliResult};
use std::fs::{canonicalize, File};
use std::io::BufReader;
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use structopt::StructOpt;

pub mod error;

const APP_INFO: AppInfo = AppInfo { name: "dictcc-rust", author: "DeggelmannAndLengler" };
const CONFIG_NAME: &'static str = "config.toml";

// TODO: Limit output length? less cross platform/rust pager

#[derive(StructOpt, Debug, Clone)]
#[structopt(name = "dictcc", about = "Offline Translator powered by the database of dict.cc")]
struct Cli {
    /// Path to the dict.cc database file. If not specified, the last used path is used instead.
    /// If there never was a path specified, an error is shown.
    #[structopt(short = "d", long = "database", parse(from_os_str))]
    database_path: Option<PathBuf>,

    /// Activates the interactive mode.
    #[structopt(short = "i", long = "interactive")]
    interactive_mode: bool,

    /// Disable colored output.
    #[structopt(short = "c", long = "no-color")]
    no_color: bool,

    /// In which language the query is written. If not specified, the query is bidirectional.
    #[structopt(short = "l", long = "language")]
    language: Option<Language>,

    /// "w" | "word" - Matches on a word in an entry.
    /// "e" | "exact" - Must match the complete entry.
    /// "r" | "regex" - Matches using the regex provided by the user.
    #[structopt(short = "t", long = "type", default_value = "Word")]
    query_type: QueryType,

    /// The query to be translated.
    #[structopt(required_unless = "interactive_mode")]
    query: Option<String>,
}

fn main() {
    let cli: Cli = Cli::from_args();

    println!("{:?}", cli);

    if let Err(err) = run_cli(cli) {
        eprintln!("{}", err);
        std::process::exit(1);
    }
}

#[derive(Deserialize, Serialize, Debug)]
struct Config {
    last_database_path: PathBuf,
}

impl Config {
    fn new<P: AsRef<Path>>(last_database_path: P) -> DictCliResult<Self> {
        Ok(Config {
            last_database_path: Config::canonicalize_path(last_database_path)?,
        })
    }

    fn get_config_path() -> DictCliResult<PathBuf> {
        let mut user_config_path = app_root(AppDataType::UserConfig, &APP_INFO)?;

        user_config_path.push(CONFIG_NAME);

        Ok(user_config_path)
    }

    fn read() -> DictCliResult<Option<Config>> {
        let config_path = Config::get_config_path()?;

        if config_path.exists() {
            let file = File::open(config_path)?;
            let mut buf_reader = BufReader::new(file);
            let mut contents = String::new();

            buf_reader.read_to_string(&mut contents)?;

            let config: Config = toml::from_str(&contents)?;
            Ok(Some(config))
        } else {
            Ok(None)
        }
    }

    fn canonicalize_path<P: AsRef<Path>>(path: P) -> DictCliResult<PathBuf> {
        Ok(canonicalize(path)?)
    }

    fn set_last_database_path<P: AsRef<Path>>(&mut self, path: P) -> DictCliResult<()> {
        self.last_database_path = Config::canonicalize_path(path)?;

        Ok(())
    }

    fn write(&self) -> DictCliResult<()> {
        let mut file = File::create(Config::get_config_path()?)?;

        let config_string = toml::to_string(self)?;

        file.write_all(config_string.as_bytes())?;

        Ok(())
    }

    fn update_with_cli<'a>(cli: &Cli) -> DictCliResult<Config> {
        let opt_config = Config::read()?;

        if let Some(mut config) = opt_config {
            if let Some(ref database_path) = cli.database_path {
                // Update Config
                config.set_last_database_path(database_path)?;

                config.write()?;

                Ok(config)
            } else {
                // Read Config
                Ok(config)
            }
        } else {
            if let Some(ref database_path) = cli.database_path {
                // Create Config
                let config = Config::new(database_path)?;

                config.write()?;

                Ok(config)
            } else {
                Err(DictCliError::NoDatabasePath)
            }
        }
    }
}

fn run_cli(cli: Cli) -> DictCliResult<()> {
    let config = Config::update_with_cli(&cli)?;

    if cli.no_color {
        colored::control::set_override(false)
    }

    let mut cloned_cli = cli.clone();
    let dict = Dict::create(config.last_database_path)?;

    if cloned_cli.query.is_some() {
        run_query(&cloned_cli, &dict)?;
    }
    if cloned_cli.interactive_mode {
        loop {
            if !update_cli_interactive(&mut cloned_cli)? {
                break;
            }
            run_query(&cloned_cli, &dict)?;
        }
    }
    Ok(())
}

fn update_cli_interactive(cli: &mut Cli) -> DictCliResult<bool> {
    println!("Enter query language (if empty, the query is bidirectional):");
    let mut tmp_lang = String::new();
    ::std::io::stdin().read_line(&mut tmp_lang)?;
    tmp_lang = tmp_lang.trim_right_matches(|c| c == '\n' || c == '\r').to_string();
    cli.language = if tmp_lang == "" {
        None
    } else {
        Some(Language::from_str(&tmp_lang)?)
    };

    println!("Enter query type (\"w(ord)\" [default], \"e(xact)\", \"r(egex)\"):");
    let mut tmp_type = String::new();
    ::std::io::stdin().read_line(&mut tmp_type)?;
    tmp_type = tmp_type.trim_right_matches(|c| c == '\n' || c == '\r').to_string();
    cli.query_type = if tmp_type == "" {
        QueryType::Word
    } else {
        QueryType::from_str(&tmp_type)?
    };

    println!("Enter query:");
    let mut query_term = String::new();
    ::std::io::stdin().read_line(&mut query_term)?;
    query_term = query_term.trim_right_matches(|c| c == '\n' || c == '\r').to_string();
    if query_term == "" {
        return Ok(false);
    }
    cli.query = Some(query_term);
    Ok(true)
}


fn run_query(cli: &Cli, dict: &Dict) -> DictCliResult<()> {
    let mut query = dict.query(cli.query.as_ref().unwrap());

    if let Some(ref language) = cli.language {
        query.source_language(language)?;
    }

    query.set_type(cli.query_type);

    let query_result = query.execute()?;

    if query_result.get_results().is_empty() {
        println!("Sorry, no translations found!");
    } else {
        let query_result_grouped = query_result.into_grouped();

        println!("{}", query_result_grouped);
    }

    Ok(())
}