extern crate csv;
#[macro_use] extern crate failure;

pub mod parse;
pub mod dict;
mod error;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
