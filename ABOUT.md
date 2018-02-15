# Development Documentation
## Who Are We?
Mathias Lengler (Github: [MathiasLengler](https://github.com/MathiasLengler)) and Kevin Deggelmann ([kedeggel](https://github.com/kedeggel))  
Both of us study applied computer science at the University of Applied Sciences (HTWG) in Konstanz, Germany.

## Idea
For the lecture course "Programmieren in Rust" (programming Rust) the exercise was to develop an own crate, that should be publish on [crates.io](https://crates.io/).  
After careful consideration we came upon the idea to create an library for translating from one language into the other.

First we thought about using existing online dictionaries and pass the translation request to one of these websites.  
But we faced one big problem: Licenses  
We made the acquaintance of them: None of these dictionaries allows to use them per sending HTTP requests for free. Either the company wants us to pay money to use their services (e.g. Google Translation API) or they need the advertising revenue (dict.cc) that would be omitted if the user didn't visit their website. The first problem could be avoided with using a free trial account, *BUT* we thought about the main advantage of Rust: it's open-source and can be used by everyone for free, so we decided that offering a crate for whose use must be paid is not the right way.  
Then we dug deeper into dict.cc's [Terms of Use](https://www1.dict.cc/translation_file_request.php?l=e) whose content summarized is that it's allowed to publish programs using their vocabulary databases, only the data must not be published with them but must be downloaded by every user to make sure they read these terms, too. This means that we write our crate, publish it, and the user only needs to download the data and bind the path to the program.

## Working method
So, we found our base for the following work. We opened an issues where we collected our ideas for the use cases, the API and how the data should be structured. We distributed tasks and used different feature branches to make look our two-man project more professional.

## Parsing the database

To illustrate the parsing of a line in the database, we like to show the process with an example:

```
(optionales) Wort &amp; {f} [Kommentar] <Akronym, anderes Akronym>	(optional) word &amp; {f} [comment] <acronym, other acronym>	verb past-p
```

First we use the crate `csv` to configure a reader with the correct CSV-parameters (comment character, delimiter, ...) and read the database one row at a time. Each row gets deserialized directly into a struct. This mapping is powered by `serde`. The example line results in a this struct:

```
RawDictEntry (
    left_word: "(optionales) Wort &amp; {f} [Kommentar] <Akronym, anderes Akronym>",
    right_word: "(optional) word &amp; {f} [comment] <acronym, other acronym>",
    word_classes: "verb past-p"
)
```

Then we use the crate `htmlescape` to decode [HTML character entity references](https://en.wikipedia.org/wiki/List_of_XML_and_HTML_character_entity_references). These are used inconsistently in the database because there seems to be little validation on the side of dict.cc and all entries are entered manually by humans. `htmlescape` fails to parse many entries. If a entry cannot be decoded, we use the raw entry instead.

```
HtmlDecodedDictEntry {
    left_word: "(optionales) Wort & {f} [Kommentar] <Akronym, anderes Akronym>",
    right_word: "(optional) word & {f} [comment] <acronym, other acronym>",
    word_classes: "verb past-p"
}
```

Now we parse the bracket syntax of dict.cc, see [Guidelines 5. Brackets](https://contribute.dict.cc/guidelines/). This is achieved using a `nom` parser and a abstract syntax tree of `WordNode(s)`

```
WordNodesDictEntry {
    left_word_nodes: WordNodes {
        nodes: [
            Round(
                "optionales"
            ),
            Word(
                "Wort"
            ),
            Word(
                "&"
            ),
            Curly(
                "f"
            ),
            Square(
                "Kommentar"
            ),
            Angle(
                [
                    "Akronym",
                    "anderes Akronym"
                ]
            )
        ]
    },
    right_word_nodes: WordNodes {
        nodes: [
            Round(
                "optional"
            ),
            Word(
                "word"
            ),
            Word(
                "&"
            ),
            Curly(
                "f"
            ),
            Square(
                "comment"
            ),
            Angle(
                [
                    "acronym",
                    "other acronym"
                ]
            )
        ]
    },
    word_classes: "verb past-p"
}
```

This rich representation of an entry allows use to do multiple useful things.

- limit the query to only find the parts of an entry, which should be searchable
- provide a pretty colored output of a entry (see CLI)
- sort the entries correctly 

But also in this parsing step we need to be conservative with the data set. Quite a few entries contain uneven brackets or other malformed syntax. This is also the result of many years of manual edits. As in the HTML-Encoding, we use a fallback if our parser is not successful. This treats to whole entry as a `Word`, so can still be found and shown, but we loose the functionality listed above. In (semi)-natural language processing there is always some kind of compromise. 

Also, our `nom` parser is not perfect, it has some problems with nested brackets and we learned, that `nom` is quite complex and probably not the right tool for the job. `nom` seems to be mainly focused on binary file decoding and Rust macros v1.1 are not easy to debug. In the future, we would like to tackle the same problem with [Pest](https://github.com/pest-parser/pest), which looks pretty promising, as it is a parser generator and not a parser combinator library, and seems to be well maintained. But there are quite a few other contenders in the Rust parsing space, namely: [Rust-peg](https://github.com/kevinmehall/Rust-peg), [chomp](https://github.com/m4rw3r/chomp) and [pom](https://github.com/J-F-Liu/pom).

In the final step, we convert our `WordNodesDictEntry` in a `DictEntry`, which is the central data structure for further handling of an entry. Here we parse the word classes into an vector of enums. We extract a `indexed_word` from the `WordNodes` and cache it in the data structure. This is used for searching and sorting of entries. We also cache the word count, as this is used for grouping of entries in the CLI.

```
DictEntry {
    left_word: DictWord {
        indexed_word: "optionales wort &",
        word_nodes: WordNodes {
        	...
        },
        word_count: 3
    },
    right_word: DictWord {
        indexed_word: "optional word &",
        word_nodes: WordNodes {
            ...
        },
        word_count: 3
    },
    word_classes: [
        Verb,
        Past
    ]
}
```

## API
For an overview of our library API we recommend a look at our Rust documentation.  We used the crate level attribute `#![warn(missing_docs)]` and documented all public API endpoints.

For an simple usage example, we provided some examples in the [examples directory](./examples). These are executable via the cargo flag `--example`.

## CLI

We also implemented a easy to use CLI. For handling of arguments we used the excellent crate `structopt` which in turn uses the feature rich `clap`. This way, we get most of the input validation for free and a nice help message:

```
dictcc 0.0.0
Mathias Lengler, Kevin Deggelmann
Offline Translator powered by the database of dict.cc

USAGE:
    dictcc.exe [FLAGS] [OPTIONS] <query>

FLAGS:
    -h, --help           Prints help information
    -i, --interactive    Activates the interactive mode.
    -c, --no-color       Disable colored output.
    -V, --version        Prints version information
    -v, --verbose        Verbose mode (-v, -vv, -vvv, etc.)

OPTIONS:
    -d, --database <database_path>    Path to the dict.cc database file. If not specified, the last used path is used
                                      instead. If there never was a path specified, an error is shown.
    -l, --language <language>         In which language the query is written. If not specified, the query is
                                      bidirectional.
    -t, --type <query_type>           "w" | "word" - Matches on a word in an entry. "e" | "exact" - Must match the
                                      complete entry. "r" | "regex" - Matches using the regex provided by the user.
                                      [default: Word]

ARGS:
    <query>    The query to be translated.
```

To get a better overview of the often huge result sets, we implemented a grouping of results, which was inspired by the result page of dict.cc. The formatting and coloring of the output was achieved using the creates `prettytable-rs` and `colored`. An example based on the test database:

```
 Verbs
 --------------------
  Verb | verb | Verb

 Nouns
 --------------------------
  DE         | EN   | Noun
  foo        | foo  | Noun
  Substantiv | noun | Noun

 Others
 -------------------
  a | c | Adjective
  B | B | Adjective
  c | a | Adjective

 2 Words: Verbs
 ----------------------------
  foo Verb | foo verb | Verb
```

We implemented the CLI solely on the public API of the library, which was a good test for its usability. In this way, we recognized a few shortcomings and fixed them.

Early on it became apparent, that the database path parameter was quite unwieldy for the use case we had in mind. The user had to supply it each time, just to look up a quick translation. For this reason, we implemented caching of this parameter.

This was a unexpectedly complex task, as we had to figure out, how to create, write, read and update a configuration file and know where to put it. For the location of the file, we used the create [app_dirs](https://docs.rs/app_dirs/1.1.1/app_dirs/), which handles the location of configuration folders correctly across different platforms. To read and write the file, we used the crate `toml` which is also powered by `serde`. This allows us to easily add other configuration/caching parameters in the future.

## Next steps

We developed this crate with a focus on functionality first. Even though Rust is a very fast language, one can still write slow software in it. 

This is currently the main shortcoming of this crate: it takes too long to read in the database (4-8s) and all entries are kept in memory (~600 MB RAM usage).

For an fast and little command line translator, this is not optimal.

Early on, we thought about multiple ways to solve this problem, but mainly through time constraints and other features, we were not able to thoroughly explore and implement them yet.

### Data Structures/Database

Our current data structure is simply a vector of entries in memory. This has the drawbacks listed above. The query is implemented as a linear search through the vector and quite fast for our use case.

There are multiple problems to solve:

- fast startup
- don't keep the dictionary in memory
- allow for fast searching of an entry
- respect the rules for searching (for example not in comments)

There are quite a few different ways in which a dictionary can be implemented more efficiently then our naive approach:

First there are more compact in memory representations:

- [Tries](https://en.wikipedia.org/wiki/Trie) / [Radix tree](https://en.wikipedia.org/wiki/Radix_tree) Rust implementations are available [Rust_radix_trie](https://github.com/michaelsproul/Rust_radix_trie)
- [fst (finite state transducers)](https://github.com/BurntSushi/fst). A Rust library, which looks promising. The author has written quite comprehensive [blog posts](https://blog.burntsushi.net/transducers/) about it.

There are a few concerns with these: Can the different query types be implemented on top of these data structures? Also, the reduction of RAM usage is dependent on the compressibility of the dict.cc database. Finally, it is unclear, how fast these structures can be initialized or how they can be cached to the file system. This is an important aspect for the CLI, which should start up quickly.

One could think of a hybrid approach: map from a data structure mentioned above to an index in the CSV file and parse the the entries on demand. This sounds promising, but could be quite complex and fragile.

Lastly, there a approaches based on the file system:

- Write some parsed Rust data structure to disk. This could help with the initial load time, as it is ca. 2/3 dominated by parsing and string conversions (measured with Intel VTune). But the overhead of reading and converting from disk is unknown. This also does not addresses the RAM usage.
- Use a database. This is probably the right option (Don't reinvent the wheel). But there are also multiple options and strategies to choose from:
	- SQLite with [FTS5 extension](https://sqlite.org/fts5.html). Bindings could be created with [diesel](https://github.com/diesel-rs/diesel), but support for FTS5 is unclear. If diesel does not support this use case, there are [other bindings](https://github.com/jgallagher/rusqlite) available.
	- A NoSQL DB. There are many different DBs out there, but not many are embedded, which is a requirement for the CLI. Availability of Rust bindings is also unclear.
	- [tantivy](https://github.com/tantivy-search/tantivy), a "full-text search engine". Written in Rust, so bindings are no problem. Seems to have quite a good feature set and documentation, but is relatively young and still in development. Also seems to focus more on full text search, which is not exactly our use case. Nevertheless the most promising option yet.


## Problems we met
- Creating more than a million dictionary entries takes time, plenty of time, so we had to be thrifty with cloning, copying, (etc.) around.
- When using extern crates, remember that they are also just written by human beings, so they are not perfect. When we were looking for a pager for better display of our output, we discovered that [this pager](https://crates.io/crates/pager) (the only existing Rust pager) doesn't work on Windows (what was/is our main developing systems). Therefore we had to use conditional compilation to tell the compiler only to use this dependency when we work on Linux.
