# Parsenip

A HTML parser (and formatter) written in 🦀 Rust 🦀.

## Usage

use in the command line `cargo r < input.html` (currently only accepts 'minified' html)

## Features

-   parse html input into a tree format
-   lex html input (token representation)
-   extract attributes and innerhtml

## Goals

-   blazing fast 🚀🔥🔥🔥

## TODO

-   better docs
-   flesh out the cli
    -   use clap 👏
    -   support multiple operations (such as formatting)
-   accept non-minified html
-   report informative errors, e.g. missing `Close('div')` after token 1
-   optimistic parsing: try to return what is parseable, possibly as multiple disjointed trees
-   serialise into HTML
-   tree operations:
    -   search
    -   manipulate
    -   gaslight
    -   girlboss
