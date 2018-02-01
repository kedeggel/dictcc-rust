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

## API
*TODO*

## Next steps
*TODO*   
CLI
