# Installation

To install *nodeset* you need first to install
[Rust](https://www.rust-lang.org/tools/install).

## From crates.io

Use cargo to directly install *nodeset* for your system:
`cargo install nodeset` (depending on your system this
may take some time - on mine it takes nearly 5 minutes).

## From source

Use git to clone the repository and build the release
binary of *nodeset*:

```bash
git clone https://gitlab.com/delhomme/nodeset.git
cd nodeset
cargo build --release
```

This will compile all needed dependencies and at last will
produce a binary program in `target/release/` named `ns`.
For now it serves as a testing program for the library
implementation and does nothing useful.

## Usage


## Related links

