# *nodeset*'s readme

## Description

Will be a library and a program to manage nodesets.
As of now one can do:

```rust
    use nodeset::node::Node;

    let node = match Node::new("r[1-10/2,15]esw[2-8]") {
        Ok(n) => n,
        Err(e) => {
            println!("Error: {}", e);
            exit(1);
        }
    };
    println!("Node string display : {}", node_str);
    println!("Node normal display : {}", node);
    println!("Node debug display  : {:?}", node);

    // use of the iterator
    for n in node {
        print!("{} ", n);
    }
    println!();
```

## Installation

To install *nodeset* you need first to install
[Rust](https://www.rust-lang.org/tools/install).

### From crates.io

Use cargo to directly install *nodeset* for your system:
`cargo install nodeset` (depending on your system this
may take some time - on mine it takes nearly 5 minutes).

### From source

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

