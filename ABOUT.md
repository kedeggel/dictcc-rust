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

Also, our `nom` parser is not perfect, it has some problems with nested brackets and we learned, that `nom` is quite complex and probably not the right tool for the job. `nom` seems to be mainly focused on binary file decoding and rust macros v1.1 are not easy to debug. In the future, we would like to tackle the same problem with [Pest](https://github.com/pest-parser/pest), which looks pretty promising, as it is a parser generator and not a parser combinator library, and seems to be well maintained. But there are quite a few other contenders in the rust parsing space, namely: [rust-peg](https://github.com/kevinmehall/rust-peg), [chomp](https://github.com/m4rw3r/chomp) and [pom](https://github.com/J-F-Liu/pom).

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
*TODO* @Mathias (Documentation, Example use case)

## CLI
*TODO* @Mathias (features, crates used)

## Next steps
*TODO*   

### Data Structures/Database
*TODO* @Mathias (crates/DBs evaluated)


## Problems we met
- Creating more than a million dictionary entries takes time, plenty of time, so we had to be thrifty with cloning, copying, (etc.) around.
- When using extern crates, remember that they are also just written by human beings, so they are not perfect. When we were looking for a pager for better display of our output, we discovered that [this pager](https://crates.io/crates/pager) (the only existing rust pager) doesn't work on Windows (what was/is our main developing systems). Therefore we had to use conditional compilation to tell the compiler only to use this dependency when we work on Linux.
