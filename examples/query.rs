extern crate dictcc;

use dictcc::Dict;

fn main() {
    let dict = Dict::create("test/database/test_database.txt").unwrap();

    let foo = dict.query("foo").exact().execute().unwrap().into_grouped();

    println!("Foo:\n{}", foo);

    let all = dict.query("*").regex().execute().unwrap().into_grouped();

    println!("All:\n{}", all);
}