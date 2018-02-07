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
extern crate simplelog;

use cli::{Cli, run_cli};
use structopt::StructOpt;

mod error;
mod cli;
mod config;

// TODO: Limit output length? less cross platform/rust pager

fn main() {
    let cli: Cli = Cli::from_args();

    println!("{:?}", cli);

    if let Err(err) = run_cli(cli) {
        eprintln!("{}", err);
        std::process::exit(1);
    }
}
