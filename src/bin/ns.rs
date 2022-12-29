/* -*- coding: utf8 -*-
 *
 *  nodeset.rs: a binary to do some basic tests while developing
 *
 *  (C) Copyright 2022 Olivier Delhomme
 *  e-mail : olivier.delhomme@free.fr
 *
 *  This program is free software; you can redistribute it and/or modify
 *  it under the terms of the GNU General Public License as published by
 *  the Free Software Foundation; either version 3, or (at your option)
 *  any later version.
 *
 *  This program is distributed in the hope that it will be useful,
 *  but WITHOUT ANY WARRANTY; without even the implied warranty of
 *  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 *  GNU General Public License for more details.
 *
 *  You should have received a copy of the GNU General Public License
 *  along with this program; if not, write to the Free Software Foundation,
 *  Inc., 59 Temple Place - Suite 330, Boston, MA 02111-1307, USA.
 */

use clap::Parser;
/// rack[10-49]node[1-25/2,78-89,101,1001].panel[0-30/4]
/// Between ',' a Range :
/// * 10-49
/// * 1-25/2,
/// * 78-89,
/// * 101,
/// * 1001
/// * 0-30/4
/// Between '[]' a Set
/// A global name 'rack{}node{}.panel{}' and a vector of sets.
use nodeset::node::Node;
use std::process::exit;

// This structure holds arguments provided to the program from the command line.
#[derive(Parser, Debug)]
/// This program manages nodeset(s) and is heavily inspired by clustershell's nodeset command
#[command()]
struct Args {
    /// counts the number of nodes in nodeset(s).
    #[arg(short, long)]
    count: bool,
    /// expands nodeset(s) to separate nodes, as is.
    #[arg(short, long)]
    expand: bool,
    nodesets: Vec<String>,
}

fn count(args: &Args) {
    for node_str in &args.nodesets {
        let node = match Node::new(&node_str) {
            Ok(n) => n,
            Err(e) => {
                println!("Error: {}", e);
                exit(1);
            }
        };
        println!("{}", node.amount());
    }
}

fn expand(args: &Args) {
    let separator = ' '; // default separator
    for node_str in &args.nodesets {
        let node = match Node::new(&node_str) {
            Ok(n) => n,
            Err(e) => {
                println!("Error: {}", e);
                exit(1);
            }
        };
        for n in node {
            print!("{}{}", n, separator);
        }
    }
}

fn fold(args: &Args) {
    for node_str in &args.nodesets {
        let node = match Node::new(&node_str) {
            Ok(n) => n,
            Err(e) => {
                println!("Error: {}", e);
                exit(1);
            }
        };
        println!("{}", node);
    }
}

fn main() {
    let args = Args::parse();

    if args.count {
        count(&args);
    } else if args.expand {
        expand(&args);
    } else {
        fold(&args);
    }
}
