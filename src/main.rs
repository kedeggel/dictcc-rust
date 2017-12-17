extern crate dictcc;
extern crate failure;

fn main() {
    if let Err(err) = dictcc::parse::debug::parse_test() {
        println!("{}", err);
        println!("{:?}", err);
    }

}
