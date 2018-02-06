extern crate csv;
extern crate failure;
extern crate regex;

use error::DictResult;
use dict::{Dict, QueryDirection};

pub fn parse_test(path: &str) -> DictResult<()> {
    let dict = Dict::create(path)?;

    println!("Left Language: {}\tRight Language: {}",
             dict.get_left_language(), dict.get_right_language());
    loop {
        let mut dq = dict.query("");

        println!("Direction (left, right or both):");
        let mut direction = String::new();
        ::std::io::stdin().read_line(&mut direction).unwrap();
        match direction.trim_right_matches(|c| c == '\n' || c == '\r') {
            "right" => { dq.set_direction(QueryDirection::ToRight); }
            "left" => { dq.set_direction(QueryDirection::ToLeft); }
            "both" => { dq.set_direction(QueryDirection::Bidirectional); }
            _ => {}
        }


        println!("Searchtype (word, exact or regex):");
        let mut qtype = String::new();
        ::std::io::stdin().read_line(&mut qtype).unwrap();
        match qtype.trim_right_matches(|c| c == '\n' || c == '\r') {
            "word" => { dq.word(); }
            "exact" => { dq.exact(); }
            "regex" => { dq.regex(); }
            _ => {}
        }

        println!("Search:");
        let mut query = String::new();
        ::std::io::stdin().read_line(&mut query).unwrap();
        query = query.trim_right_matches(|c| c == '\n' || c == '\r').to_string();
        if query == "" {
            break
        }

        eprintln!("query = {:?}", query);
        let dq = dq.set_term(&query);
        let dqr = dq.execute();

        println!("{}", dqr.into_grouped());
    }
    Ok(())
}
