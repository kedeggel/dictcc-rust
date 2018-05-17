extern crate app_dirs;
extern crate colored;
extern crate dictcc;
extern crate dunce;
#[macro_use]
extern crate failure;
#[macro_use]
extern crate log;
#[cfg(unix)]
extern crate pager;
extern crate simplelog;
extern crate structopt;
#[macro_use]
extern crate structopt_derive;

use cli::{run_cli, Cli};
use std::io::ErrorKind;
use structopt::StructOpt;
use error::DictCliError;
use std::time::Duration;

mod error;
mod cli;
mod persistence;

fn main() {
    let cli: Cli = Cli::from_args();

    if let Err(err) = run_cli(cli) {
        let err = match err {
            DictCliError::Io(err) => {
                match err.kind() {
                    ErrorKind::BrokenPipe => {
                        // Pager exited prematurely, this is an expected error
                        ::std::process::exit(0);
                    }
                    _ => {
                        // Other IO Error
                        DictCliError::Io(err)
                    }
                }
            }
            err => err,
        };

        error!("{}", err);
        debug!("{:?}", err);
        std::process::exit(1);
    }
}
