extern crate dictcc;
extern crate failure;

fn main() {
    if let Err(err) = dictcc::parse::parse_test() {
        println!("{}", err);
        println!("{:?}", err);
    }
}
