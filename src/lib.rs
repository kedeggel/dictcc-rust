extern crate csv;

use std::error::Error;


pub fn parse_test() -> Result<(), Box<Error>> {
    let mut rdr = csv::ReaderBuilder::new()
        .delimiter(b'\t')
        .has_headers(false)
        .quoting(false)
        .comment(Some(b'#'))
        .from_path("../database/dictcc_DE-EN.txt")?;

    let mut vec = vec![];

    for (i, record) in rdr.records().enumerate() {
        let record = match record {
            Ok(record) => record,
            Err(err) => {
                let abort = {
                    let kind: &csv::ErrorKind = err.kind();

                    match *kind {
                        csv::ErrorKind::UnequalLengths { .. } => false,
                        _ => true,
                    }
                };


                if abort {
                    return Err(Box::new(err));
                } else {
                    eprintln!("Dictionary Parsing Error: {}", err);
                    continue;
                }
            }
        };

        if i % 1000 == 0 {
            println!("{:?}", record);
        }

        vec.push(record);
    }

    eprintln!("vec.len() = {:?}", vec.len());

    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
