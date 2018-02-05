extern crate dictcc;
extern crate structopt;
#[macro_use]
extern crate structopt_derive;

use structopt::StructOpt;
use dictcc::dict::QueryType;

#[derive(StructOpt, Debug)]
#[structopt(name = "dictcc", about = "Translator powered by the translation database of dict.cc")]
struct Cli {
    /// Path to the dict.cc database file.
    #[structopt(short = "d", long = "database")]
    database_path: String,

    /// In which language the query is written. If not specified, the query is bidirectional.
    #[structopt(short = "l", long = "language")]
    language: Option<String>,

    #[structopt(short = "t", long = "type")]
    query_type: QueryType,

    /// First query term.
    query: String,

    /// Rest of the query.
    query_rest: Vec<String>,
}

fn main() {
    let opt = Cli::from_args();
    println!("{:?}", opt);
}