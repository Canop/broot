
Broot uses [syntect](https://crates.io/crates/syntect) for syntax highlighting in text files.

This (excellent) library needs Sublime Text syntax definitions for all languages (when a language definition isn't found, Broot displays the text monochrome).

Syntect doesn't come with an extensive set of definitions.

The [bat](https://github.com/sharkdp/bat) project maintains with care an important list of such definitions (most as submodules, some with patches).

It's the best public list I found, so I've included the resulting syntax set here as `syntaxes.bin`.

You may replace this file with your own, building it with Syntect's [`syntect::dumps::dump_to_file`](https://docs.rs/syntect/4.6.0/syntect/dumps/fn.dump_to_file.html) function.
