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

use app_dirs::*;
use dictcc::dict::Dict;
use dictcc::dict::Language;
use dictcc::dict::QueryType;
use error::DictCliResult;
use std::fs::{canonicalize, File};
use std::io::BufReader;
use std::io::prelude::*;
use std::iter::once;
use std::path::PathBuf;
use structopt::StructOpt;
use std::path::Path;
use error::DictCliError;

pub mod error;

const APP_INFO: AppInfo = AppInfo { name: "dictcc-rust", author: "DeggelmannAndLengler" };
const CONFIG_NAME: &'static str = "config.toml";

// TODO: Caching of database parameter (App Dirs) @ Matze

// TODO: Print something if result is empty @ Matze

// TODO: Interactive mode? (-i interactive) only query

// TODO: Limit output length? less cross platform/rust pager

// TODO: Disable color flag @ Matze

#[derive(StructOpt, Debug)]
#[structopt(name = "dictcc", about = "Translator powered by the translation database of dict.cc")]
struct Cli {
    /// Path to the dict.cc database file. If not specified, the last used path is selected instead.
    /// If there was never a path specified, an error is shown.
    #[structopt(short = "d", long = "database", parse(from_os_str))]
    database_path: Option<PathBuf>,

    /// In which language the query is written. If not specified, the query is bidirectional.
    #[structopt(short = "l", long = "language")]
    language: Option<Language>,

    /// "w" | "word" - Matches on a word in an entry.
    /// "e" | "exact" - Must match the complete entry.
    /// "r" | "regex" - Matches using the regex provided by the user.
    #[structopt(short = "t", long = "type", default_value = "Word")]
    query_type: QueryType,

    /// First query term.
    query: String,

    /// Rest of the query.
    query_rest: Vec<String>,
}

fn main() {
    let cli = Cli::from_args();
    println!("{:?}", cli);

    if let Err(err) = run_query(cli) {
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


fn run_query(cli: Cli) -> DictCliResult<()> {
    let config = Config::update_with_cli(&cli)?;

    let dict = Dict::create(config.last_database_path)?;

    let query_term = once(cli.query.as_str())
        .chain(cli.query_rest.iter().map(String::as_str))
        .collect::<Vec<_>>()
        .join(" ");

    let mut query = dict.query(&query_term);

    if let Some(language) = cli.language {
        query.source_language(&language)?;
    }

    query.set_type(cli.query_type);

    let query_result = query.execute()?;

    let query_result_grouped = query_result.into_grouped();

    println!("{}", query_result_grouped);

    Ok(())
}