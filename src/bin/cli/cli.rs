use colored;
use config::Config;
use dictcc::dict::{Dict, Language, QueryType};
use error::DictCliResult;
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
    let config = Config::update_with_cli(&cli)?;

    if cli.no_color {
        colored::control::set_override(false)
    }

    let mut cloned_cli = cli.clone();
    let dict = Dict::create(config.get_database_path())?;

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