#![feature(test)]

extern crate test;
extern crate dictcc;

#[cfg(test)]
mod tests {
    use super::*;
    use test::Bencher;

    #[bench]
    fn bench_parse(b: &mut Bencher) {
        b.iter(|| dictcc::parse_test());
    }
}