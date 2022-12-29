/* -*- coding: utf8 -*-
 *
 *  node.rs: Implements all structure and logic to manage Node
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

use crate::rangeset::RangeSet;
use lazy_static::lazy_static;
use regex::Regex;
use std::error::Error;
use std::fmt;
use std::process::exit; // used for testing

/// A Node is a name that may contain multiple RangeSets and
/// that defnines a machine name. For instance `node[1-14]` is
/// a valid Node defining 14 nodes name from node1 to node14.
/// Let's say that within these nodes we want to reference each
/// core (32 per cpu) of each cpu (2 per node) we could do this
/// writing `node[1-14]-cpu[1-2]-core[1-32]`.
///
/// ```rust
///
/// use nodeset::node::Node;
/// use nodeset::range::Range;
/// use nodeset::rangeset::RangeSet;
/// use std::process::exit;
/// let node = match Node::new("r1esw[2-6]") {
///     Ok(n) => n,
///     Err(e) => {
///         println!("Error: {}", e);
///         exit(1);
///     }
/// };
/// let v: Vec<_> = node.into_iter().map(|x| x).collect();
/// assert_eq!(v, ["r1esw2", "r1esw3", "r1esw4", "r1esw5", "r1esw6"]);
/// ```
///
///
/// Structure used to keep Node definition.
#[derive(Debug)] /* Auto generates Debug trait */
pub struct Node {
    name: String,
    sets: Vec<RangeSet>,
    values: Vec<(u32, usize)>,
    first: bool,
}

#[derive(Debug)]
pub enum NodeErrorType {
    Regular(ErrorKind),
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ErrorKind {
    RegexNoMatch,
}

impl ErrorKind {
    fn as_str(&self) -> &str {
        match *self {
            ErrorKind::RegexNoMatch => "no match found in string",
        }
    }
}

impl fmt::Display for NodeErrorType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            NodeErrorType::Regular(ref err) => write!(f, "Error: {:?}", err),
        }
    }
}

impl Error for NodeErrorType {
    fn description(&self) -> &str {
        match *self {
            NodeErrorType::Regular(ref err) => err.as_str(),
        }
    }
}

lazy_static! {
    static ref RE: Regex = Regex::new(r"\[([\d,\-/]+)\]|([\d]+)").unwrap();
}

impl Node {
    pub fn amount(&self) -> u32 {
        if self.sets.is_empty() {
            if self.name.is_empty() {
                0
            } else {
                1
            }
        } else {
            let mut total = 1;
            for r in self.sets.iter() {
                total *= r.amount();
            }
            total
        }
    }

    /// Captures with regex all possible (and non overlapping) rangeset in the node name
    /// for instance rack[1-8]-node[1-42] should return 1-8 and 1-42 as rangeset
    /// It will capture mixed types of rangesets ie: rack1-node[1-42]-cpu2
    fn capture_with_regex(nodename: &str) -> Result<(String, Vec<String>), NodeErrorType> {
        let mut rangesets: Vec<String> = Vec::new();
        let mut name = nodename.to_string();
        for capture in RE.captures_iter(nodename) {
            // println!("capture: {:?}", capture);
            match capture.get(1) {
                Some(text) => rangesets.push(text.as_str().to_string()),
                None => {
                    match capture.get(2) {
                        Some(text) => rangesets.push(text.as_str().to_string()),
                        None => (),
                    };
                }
            };
        }
        if !rangesets.is_empty() {
            name = RE.replace_all(nodename, "{}").to_string();
        }
        // println!("name: {}", name);

        Ok((name, rangesets))
    }

    /* Node examples: "node[1-5/2]" or "rack[1,3-5,89]" or "cpu[1-2]core[1-64]" or node01 */
    pub fn new(str: &str) -> Result<Node, NodeErrorType> {
        let (name, rangesets) = Node::capture_with_regex(str)?;
        let mut sets: Vec<RangeSet> = Vec::new();
        let mut values: Vec<(u32, usize)> = Vec::new();
        for set in rangesets {
            let rangeset = match RangeSet::new(&set) {
                Ok(r) => r,
                Err(_) => return Err(NodeErrorType::Regular(ErrorKind::RegexNoMatch)),
            };
            sets.push(rangeset);
            values.push((0, 0));
        }

        Ok(Node {
            name,
            sets,
            values,
            first: true,
        })
    }

    fn make_node_string(&self) -> String {
        let mut nodestr: &str = self.name.as_str();
        let mut replaced;

        for i in 0..self.sets.len() {
            let (current, pad) = self.values[i];
            replaced = nodestr.replacen("{}", format!("{:0pad$}", current).as_str(), 1);
            nodestr = replaced.as_str();
        }

        nodestr.to_string()
    }

    fn get_next(&mut self) -> Option<(u32, usize)> {
        for i in (0..self.sets.len()).rev() {
            //println!("{}: {:?}", i, self.sets[i]);
            match self.sets[i].get_next() {
                Some(v) => {
                    //println!("{}: {:?} - {}", i, self.sets[i], v);
                    self.values[i] = v;
                    return Some(v);
                }
                None => {
                    self.sets[i].reset();
                    self.values[i] = self.sets[i].get_current();
                    self.sets[i].get_next();
                }
            };
        }
        None
    }
}

/// Range and Rangeset iterator returns an already padded String
/// but get_next() method doesn't.
impl Iterator for Node {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        if self.sets.is_empty() {
            if self.first {
                self.first = false;
                Some(self.name.to_string())
            } else {
                None
            }
        } else {
            if self.first {
                self.first = false;
                for i in 0..self.sets.len() {
                    self.values[i] = match self.sets[i].get_next() {
                        Some(v) => v,
                        None => self.sets[i].get_current(),
                    };
                }
                return Some(self.make_node_string());
            }

            match self.get_next() {
                Some(_) => Some(self.make_node_string()),
                None => None,
            }
        }
    }
}

/// Display trait for Node. It will display the node in a folded way
impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut nodestr: &str = self.name.as_str();
        let mut replaced;
        for set in &self.sets {
            if set.is_alone() {
                replaced = nodestr.replacen("{}", format!("{}", set).as_str(), 1)
            } else {
                replaced = nodestr.replacen("{}", format!("[{}]", set).as_str(), 1)
            };
            nodestr = replaced.as_str();
        }
        write!(f, "{}", nodestr)
    }
}

#[cfg(test)] /* Helper function for testing */
fn get_node_values_from_str(node_str: &str) -> Vec<String> {
    let node = match Node::new(node_str) {
        Ok(r) => r,
        Err(e) => {
            println!("Error: {}", e);
            exit(1);
        }
    };
    let mut v: Vec<String> = Vec::new();
    for n in node {
        v.push(n);
    }
    v
}

#[test]
fn testing_nodes_values() {
    let value = get_node_values_from_str("r[1-6]esw[1-3]");
    assert_eq!(
        value,
        vec!["r1esw1", "r1esw2", "r1esw3", "r2esw1", "r2esw2", "r2esw3", "r3esw1", "r3esw2", "r3esw3", "r4esw1", "r4esw2", "r4esw3", "r5esw1", "r5esw2", "r5esw3", "r6esw1", "r6esw2", "r6esw3"]
    );

    let value = get_node_values_from_str("node[01-10,7-12/2]");
    assert_eq!(value, vec!["node01", "node02", "node03", "node04", "node05", "node06", "node07", "node08", "node09", "node10", "node7", "node9", "node11"]);

    let value = get_node_values_from_str("node001");
    assert_eq!(value, vec!["node001"]);

    let value = get_node_values_from_str("node[1]");
    assert_eq!(value, vec!["node1"]);

    let value = get_node_values_from_str("r1esw[2-6]");
    assert_eq!(value, vec!["r1esw2", "r1esw3", "r1esw4", "r1esw5", "r1esw6"]);

    let value = get_node_values_from_str("toto");
    assert_eq!(value, vec!["toto"]);

    let value = get_node_values_from_str("r[1-7/2,15]esw[2-4]");
    assert_eq!(value, vec!["r1esw2", "r1esw3", "r1esw4", "r3esw2", "r3esw3", "r3esw4", "r5esw2", "r5esw3", "r5esw4", "r7esw2", "r7esw3", "r7esw4", "r15esw2", "r15esw3", "r15esw4"]);

    let value = get_node_values_from_str("rack1-node[1-3]-cpu2");
    assert_eq!(value, vec!["rack1-node1-cpu2", "rack1-node2-cpu2", "rack1-node3-cpu2"]);

    let value = get_node_values_from_str("rack[1-2]-node[1-2]-cpu[1-2]");
    assert_eq!(value, vec!["rack1-node1-cpu1", "rack1-node1-cpu2", "rack1-node2-cpu1", "rack1-node2-cpu2", "rack2-node1-cpu1", "rack2-node1-cpu2", "rack2-node2-cpu1", "rack2-node2-cpu2"]);
}
