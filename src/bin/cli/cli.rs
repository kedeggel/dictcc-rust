extern crate colored;

use dictcc::Language;
use error::DictCliResult;
#[cfg(unix)]
use pager::Pager;
use simplelog::{self, LevelFilter, TermLogger};
use std::default::Default;
use std::io;
use std::io::prelude::*;
use std::path::PathBuf;
use std::str::FromStr;
use persistence::db::DBAction;
use dictcc::sqlite::SqliteController;
use persistence::db::sqlite_db_path;
use error::DictCliError;


#[derive(StructOpt, Debug, Clone)]
#[structopt(name = "dictcc", about = "Offline Translator powered by the database of dict.cc")]
pub struct Cli {
    /// Activates the interactive mode.
    #[structopt(short = "i", long = "interactive")]
    pub interactive_mode: bool,

    /// Disable colored output.
    #[structopt(short = "c", long = "no-color")]
    pub no_color: bool,

    /// Do not use the configuration file.
    #[structopt(long = "no-config")]
    pub no_config: bool,

    /// Do not use a pager to buffer long output.
    #[structopt(long = "no-pager")]
    pub no_pager: bool,

    /// Verbose mode (-v, -vv, -vvv, etc.)
    #[structopt(short = "v", long = "verbose", parse(from_occurrences))]
    pub verbose: u8,

    /// In which language the query is written. If not specified, the query is bidirectional.
    #[structopt(short = "l", long = "language")]
    pub language: Option<Language>,

    /// The query to be translated.
    #[structopt(
    required_unless = "interactive_mode",
    required_unless = "manage",
    )]
    pub query: Option<String>,

    // DB Actions

    /// List all dictionaries in the database.
    #[structopt(long = "list", group = "manage")]
    pub list: bool,

    /// Add a new dictionary to the database.
    /// Expects a path to a dict.cc database file.
    #[structopt(long = "add", group = "manage", parse(from_os_str))]
    pub add: Option<PathBuf>,

    /// Delete a dictionary from the database.
    /// Expects the identifier of the dictionary.
    #[structopt(long = "delete", group = "manage")]
    pub delete: Option<String>,
}


pub fn run_cli(cli: Cli) -> DictCliResult<()> {
    init_log(&cli)?;

    debug!("cli = {:?}", cli);

    if cli.no_color {
        colored::control::set_override(false)
    }

    if let Some(management) = Into::<Option<DBAction>>::into(&cli) {
        management.execute()?;
        return Ok(());
    }

    let mut controller = SqliteController::new(sqlite_db_path()?)?;

    if cli.query.is_some() {
        run_query(&cli, &mut controller)?;
    }
    if cli.interactive_mode {
        let mut cli = cli;

        loop {
            if !update_cli_interactive(&mut cli)? {
                break;
            }
            run_query(&cli, &mut controller)?;
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

    println!("Enter query:");
    let query_term = read_stdin_line()?;
    if query_term == "" {
        return Ok(false);
    }
    cli.query = Some(query_term);
    Ok(true)
}


fn run_query(cli: &Cli, controller: &mut SqliteController) -> DictCliResult<()> {
    let dicts = controller.list_dicts()?;

    // TODO: only print biggest result set
    // TODO: allow narrowing the query to specific dicts

    if dicts.len() == 0 {
        return Err(DictCliError::NoDictionary)
    }

    for metadata in dicts {
        println!("Dictionary: {}", metadata.languages);

        let mut dict = controller.get_dict(&metadata.dict_id)?;

        let mut query = dict.query(cli.query.as_ref().unwrap());

        if let Some(ref language) = cli.language {
            query.source_language(language)?;
        }

        let query_result = query.execute()?;

        if query_result.entries().is_empty() {
            println!("Sorry, no translations found!");
        } else {
            let query_result_grouped = query_result.into_grouped();

            let mut stdout = io::stdout();

            if !(cli.interactive_mode || cli.no_pager) {
                #[cfg(unix)] Pager::with_pager("less -r").setup();
            }

            writeln!(&mut stdout, "{}", query_result_grouped)?;
        }
    }



    Ok(())
}
