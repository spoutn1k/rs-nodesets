/* -*- coding: utf8 -*-
 *
 *  nodeset.rs: a binary to do some basic tests while developing
 *
 *  (C) Copyright 2022 - 2023 Olivier Delhomme
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

use clap::{Args, Parser, Subcommand};
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
use nodeset::NodeSet;
use std::error::Error;
use std::process::exit;

// This structure holds arguments provided to the program from the command line.
#[derive(Parser, Debug)]
/// This program manages nodeset(s) and is heavily inspired by clustershell's nodeset command
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Arguments {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Count(Count),
    Expand(Expand),
    Fold(Fold),
}

/// counts the number of nodes in nodeset(s).
#[derive(Args, Debug)]
struct Count {
    /// sums all nodes of every given nodesets as it was one nodeset
    #[arg(short, long)]
    total: bool,
    nodesets: Vec<String>,
}

/// expands nodeset(s) to separate nodes, as is.
#[derive(Args, Debug)]
struct Expand {
    /// character to use to separate nodes
    #[arg(short, long)]
    #[arg(default_value_t = ' ')]
    separator: char,

    nodesets: Vec<String>,
}

/// Folds nodeset(s) into a synthetic notation
#[derive(Args, Debug)]
struct Fold {
    nodesets: Vec<String>,
}

fn count(count: &Count) {
    let mut total = 0;
    for node_str in &count.nodesets {
        let node = match NodeSet::new(node_str) {
            Ok(n) => n,
            Err(e) => {
                println!("Error: {e}");
                exit(1);
            }
        };
        if count.total {
            total += node.len();
        } else {
            println!("{}", node.len());
        }
    }
    if count.total {
        println!("{total}");
    }
}

fn expand(expand: &Expand) -> Result<(), Box<dyn Error>> {
    let separator = &expand.separator;

    for node_str in &expand.nodesets {
        let node = match NodeSet::new(node_str) {
            Ok(n) => n,
            Err(e) => return Err(Box::new(e)),
        };
        match node.expand(format!("{separator}").as_str()) {
            Ok(s) => println!("{s}"),
            Err(e) => println!("Error while expanding nodeset {node}: {e}"),
        };
    }
    Ok(())
}

fn fold(fold: &Fold) {
    for node_str in &fold.nodesets {
        let node = match NodeSet::new(node_str) {
            Ok(n) => n,
            Err(e) => {
                println!("Error: {e}");
                exit(1);
            }
        };
        println!("{node}");
        println!("{node:?}");
    }
}

fn main() {
    let args = Arguments::parse();

    match &args.command {
        Commands::Count(c) => {
            count(c);
        }
        Commands::Expand(e) => {
            if let Err(e) = expand(e) {
                println!("Error: {e}");
                exit(1);
            }
        }
        Commands::Fold(f) => {
            fold(f);
        }
    };
}
