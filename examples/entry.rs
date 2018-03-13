extern crate dictcc;

use dictcc::VecDict;

fn main() {
    let dict = VecDict::create("test/database/test_database.txt").unwrap();

    let query_result = dict.query("Wort").execute().unwrap();

    for entry in query_result.entries() {
        println!("Plain word: {}", entry.left_word.plain_word());
        println!("The word with optional parts: {}", entry.left_word.word_with_optional_parts());
        println!("Acronyms: {:?}", entry.left_word.acronyms());
        println!("Comments: {:?}", entry.left_word.comments());
        println!("Gender Tags: {:?}", entry.left_word.genders());
    }

    // Pretty table printing
    println!("{}", query_result.into_grouped());
}