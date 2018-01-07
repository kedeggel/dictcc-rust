extern crate dictcc;
extern crate failure;
extern crate env_logger;

fn main() {
    env_logger::init();

    if let Err(err) = dictcc::parse::debug::parse_test() {
        println!("{}", err);
        println!("{:?}", err);
    }

}
