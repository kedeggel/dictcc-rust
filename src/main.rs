extern crate csv;

use std::error::Error;

fn parse_test() -> Result<(), Box<Error>> {
    let mut rdr = csv::ReaderBuilder::new()
        .delimiter(b'\t')
        .has_headers(false)
        .quoting(false)
        .comment(Some(b'#'))
        .from_path("../database/dictcc_DE-EN.txt")?;

    for record in rdr.records().skip(1000).take(100) {
        let record = record?;
        eprintln!("record = {:?}", record);
    }

    Ok(())
}

fn main() {
    parse_test().unwrap();
}
