extern crate csv;
extern crate failure;
extern crate regex;

use error::DictResult;
use dict::{Dict, QueryDirection};
use regex::Regex;

pub fn parse_test(path: &str) -> DictResult<()> {

    // Header
    {
        let mut with_header = csv::ReaderBuilder::new().from_path(path)?;
        let header = with_header.headers().unwrap();
        let re = Regex::new("[A-Z]{2}-[A-Z]{2}").unwrap();
        let mat = re.find(header.get(0).unwrap()).unwrap();
        println!("HEADER: {:?}", header);
        println!("matches: {:?}", mat.as_str());

    }
    // --- Header


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
