extern crate nom;

use nom::IResult;
use std::str::from_utf8;
use std::fmt::Debug;

named!(round_br<&str,&str>, delimited!(tag_s!("("), is_not_s!(")"), tag_s!(")")));
named!(square_br<&str,&str>, delimited!(tag_s!("["), is_not_s!("]"), tag_s!("]")));
named!(curly_br<&str,&str>, delimited!(tag_s!("{"), is_not_s!("}"), tag_s!("}")));

fn debug() {
    let round_res = round_br("(foo)");
    let square_res = square_br("[foo]");
    let curly_res = curly_br("{foo}");

    eprintln!("round_res = {:?}", round_res);
    eprintln!("square_res = {:?}", square_res);
    eprintln!("curly_res = {:?}", curly_res);
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        super::debug();
    }
}
