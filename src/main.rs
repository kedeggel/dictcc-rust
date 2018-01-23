extern crate dictcc;
extern crate failure;
extern crate env_logger;
extern crate regex;

fn main() {
    env_logger::init();

    let mut args = std::env::args();
    args.next();

    let path = match args.next() {
        Some(path) => path,
        None => "database/dictcc_DE-EN.txt".to_string(),
    };

    if let Err(err) = dictcc::parse::debug::parse_test(&path) {
        println!("{}", err);
        println!("{:?}", err);
    }
}
