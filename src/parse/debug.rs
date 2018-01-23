extern crate csv;
extern crate failure;

use error::DictResult;
use dict::{Dict, QueryDirection};

pub fn parse_test(path: &str) -> DictResult<()> {
    let dict = Dict::create(path)?;
    let mut dq = dict.query();

    loop {
        println!("Direction (left, right or both):");
        let mut direction = String::new();
        ::std::io::stdin().read_line(&mut direction).unwrap();
        match direction.trim_right_matches('\n') {
            "right" => { dq.set_query_direction(QueryDirection::ToRight); }
            "left" => { dq.set_query_direction(QueryDirection::ToLeft); }
            "both" => { dq.set_query_direction(QueryDirection::Bidirectional); }
            _ => {}
        }


        println!("Searchtype (word, exact or regex):");
        let mut qtype = String::new();
        ::std::io::stdin().read_line(&mut qtype).unwrap();
        match qtype.trim_right_matches('\n') {
            "word" => { dq.word(); }
            "exact" => { dq.exact(); }
            "regex" => { dq.regex(); }
            _ => {}
        }

        println!("Search:");
        let mut query = String::new();
        ::std::io::stdin().read_line(&mut query).unwrap();
        query = query.trim_right_matches("\n").to_string();
        if query == "" {
            break
        }

        eprintln!("query = {:?}", query);
        let dqr = dq.query(&query);
        for (i, res) in dqr.get_results().iter().enumerate() {
            println!("Result {}: {}", i + 1, res);
            println!("Result {} (verbose): {}", i + 1, res.to_long_string());
        }

        println!("{}", dqr.into_grouped());
    }
    Ok(())
}
