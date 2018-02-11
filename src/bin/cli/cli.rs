extern crate colored;

use config::Config;
use dictcc::{Dict, Language, QueryType};
use error::DictCliResult;
#[cfg(unix)]
use pager::Pager;
use simplelog::{self, LevelFilter, TermLogger};
use std::default::Default;
use std::io;
use std::io::prelude::*;
use std::path::PathBuf;
use std::str::FromStr;

#[derive(StructOpt, Debug, Clone)]
#[structopt(name = "dictcc", about = "Offline Translator powered by the database of dict.cc")]
pub struct Cli {
    /// Path to the dict.cc database file. If not specified, the last used path is used instead.
    /// If there never was a path specified, an error is shown.
    #[structopt(short = "d", long = "database", parse(from_os_str))]
    pub database_path: Option<PathBuf>,

    /// Activates the interactive mode.
    #[structopt(short = "i", long = "interactive")]
    pub interactive_mode: bool,

    /// Disable colored output.
    #[structopt(short = "c", long = "no-color")]
    pub no_color: bool,

    /// Verbose mode (-v, -vv, -vvv, etc.)
    #[structopt(short = "v", long = "verbose")]
    pub verbose: u64,

    /// In which language the query is written. If not specified, the query is bidirectional.
    #[structopt(short = "l", long = "language")]
    pub language: Option<Language>,

    /// "w" | "word" - Matches on a word in an entry.
    /// "e" | "exact" - Must match the complete entry.
    /// "r" | "regex" - Matches using the regex provided by the user.
    #[structopt(short = "t", long = "type", default_value = "Word")]
    pub query_type: QueryType,

    /// The query to be translated.
    #[structopt(required_unless = "interactive_mode")]
    pub query: Option<String>,
}


pub fn run_cli(cli: Cli) -> DictCliResult<()> {
    init_log(&cli)?;

    debug!("cli = {:?}", cli);

    let config = Config::update_with_cli(&cli)?;

    debug!("config = {:?}", config);

    if cli.no_color {
        colored::control::set_override(false)
    }

    info!("Using database path: {}", config.get_database_path().display());
    let dict = Dict::create(config.get_database_path())?;

    let mut cli = cli;

    if cli.query.is_some() {
        run_query(&cli, &dict)?;
    }
    if cli.interactive_mode {
        loop {
            if !update_cli_interactive(&mut cli)? {
                break;
            }
            run_query(&cli, &dict)?;
        }
    }
    Ok(())
}

fn init_log(cli: &Cli) -> DictCliResult<()> {
    let filter = match cli.verbose {
        0 => LevelFilter::Error,
        1 => LevelFilter::Warn,
        2 => LevelFilter::Info,
        3 => LevelFilter::Debug,
        _ => LevelFilter::Trace,
    };

    TermLogger::init(filter, simplelog::Config::default())?;

    Ok(())
}

fn update_cli_interactive(cli: &mut Cli) -> DictCliResult<bool> {
    fn read_stdin_line() -> DictCliResult<String> {
        let mut line = String::new();
        ::std::io::stdin().read_line(&mut line)?;
        Ok(line.trim_right_matches(|c| c == '\n' || c == '\r').to_string())
    }

    println!("Enter query language (if empty, the query is bidirectional):");
    let tmp_lang = read_stdin_line()?;
    cli.language = if tmp_lang == "" {
        None
    } else {
        Some(Language::from_str(&tmp_lang)?)
    };

    println!("Enter query type (\"w(ord)\" [default], \"e(xact)\", \"r(egex)\"):");
    let tmp_type = read_stdin_line()?;
    cli.query_type = if tmp_type == "" {
        QueryType::Word
    } else {
        QueryType::from_str(&tmp_type)?
    };

    println!("Enter query:");
    let query_term = read_stdin_line()?;
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

    if query_result.entries().is_empty() {
        println!("Sorry, no translations found!");
    } else {
        let query_result_grouped = query_result.into_grouped();

        let mut stdout = io::stdout();

        #[cfg(unix)] Pager::with_pager("less -r").setup();

        writeln!(&mut stdout, "{}", query_result_grouped)?;
    }

    Ok(())
}