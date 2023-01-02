# *nodeset*'s readme

## Description

nodeset is a library and a program (`ns`) to manage nodesets written in Rust.
As of now one can iterate over nodes, count (`amount()` method) them, display
in a folded way :

```rust
    use nodeset::node::Node;
    use std::process::exit;

    let node = match Node::new("r[1-10/2,15]esw[2-8]") {
        Ok(n) => n,
        Err(e) => {
            println!("Error: {}", e);
            exit(1);
        }
    };
    println!("Node string display : {}", "r[1-10/2,15]esw[2-8]");
    println!("Node normal display : {}", node);
    println!("Node debug display  : {:?}", node);
    println!("Node count          : {}", node.amount());

    // use of the iterator
    for n in node {
        print!("{} ", n);
    }
```

It will print:
```terminal
Node string display : r[1-10/2,15]esw[2-8]
Node normal display : r[1-10/2,15]esw[2-8]
Node debug display  : Node { name: "r{}esw{}", sets: [RangeSet { set: [Range { start: 1, end: 10, step: 2, pad: 0, curr: 1 }, Range { start: 15, end: 15, step: 1, pad: 0, curr: 15 }], curr: 0 }, RangeSet { set: [Range { start: 2, end: 8, step: 1, pad: 0, curr: 2 }], curr: 0 }], values: [(0, 0), (0, 0)], first: true }
Node count          : 64
r1esw2 r1esw3 r1esw4 r1esw5 r1esw6 r1esw7 r1esw8 r3esw2 r3esw3 r3esw4 r3esw5 r3esw6 r3esw7 r3esw8 r5esw2 r5esw3 r5esw4 r5esw5 r5esw6 r5esw7 r5esw8 r7esw2 r7esw3 r7esw4 r7esw5 r7esw6 r7esw7 r7esw8 r9esw2 r9esw3 r9esw4 r9esw5 r9esw6 r9esw7 r9esw8 r15esw2 r15esw3 r15esw4 r15esw5 r15esw6 r15esw7 r15esw8
```

## ns

`ns` is a small command line tool to manage nodeset(s). Use `-h` or `--help` to get commands that are usable.


See [INSTALL.md](https://gitlab.com/delhomme/nodeset/-/blob/master/INSTALL.md) file in the root path of the code for instruction on how to install this crate.
