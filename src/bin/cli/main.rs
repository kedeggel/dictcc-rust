extern crate dictcc;
extern crate structopt;
#[macro_use]
extern crate structopt_derive;

use structopt::StructOpt;
use dictcc::dict::QueryType;
use dictcc::dict::Language;
use dictcc::dict::Dict;
use dictcc::error::DictResult;
use std::iter::once;

// TODO: Caching of database parameter (App Dirs) @ Matze

// TODO: Print something if result is empty @ Matze

// TODO: Interactive mode? (-i interactive) only query

// TODO: Limit output length? less cross platform/rust pager

// TODO: Disable color flag @ Matze

#[derive(StructOpt, Debug)]
#[structopt(name = "dictcc", about = "Translator powered by the translation database of dict.cc")]
struct Cli {
    /// Path to the dict.cc database file.
    #[structopt(short = "d", long = "database")]
    database_path: String,

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

fn run_query(cli: Cli) -> DictResult<()> {
    let dict = Dict::create(&cli.database_path)?;

    let query_term = once(cli.query.as_str())
        .chain(cli.query_rest.iter().map(String::as_str))
        .collect::<Vec<_>>()
        .join(" ");

    let mut query = dict.query(&query_term);

    if let Some(language) = cli.language {
        query.source_language(&language)?;
    }

    query.set_type(cli.query_type);

    let query_result = query.execute();

    let query_result_grouped = query_result.into_grouped();

    println!("{}", query_result_grouped);

    Ok(())
}