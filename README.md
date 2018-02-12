# dictcc-rust
[![Crates.io](https://img.shields.io/crates/v/dictcc.svg)](https://crates.io/crates/dictcc)
[![dictcc](https://docs.rs/dictcc/badge.svg)](https://docs.rs/dictcc)
[![Build Status](https://travis-ci.org/kedeggel/dictcc-rust.svg?branch=master)](https://travis-ci.org/kedeggel/dictcc-rust)
[![Build status](https://ci.appveyor.com/api/projects/status/hdtge4kfoj961ur7/branch/master?svg=true)](https://ci.appveyor.com/project/kedeggel/dictcc-rust/branch/master)

Rust API for use of dict.cc translation data

## Example

```
extern crate dictcc;

use dictcc::Dict;

fn main() {
    let dict = Dict::create("test/database/test_database.txt").unwrap();

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
```