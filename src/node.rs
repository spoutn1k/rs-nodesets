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
use std::fmt::Write;
use regex::Regex;
use std::error::Error;
use std::fmt;
use std::str::FromStr;

#[cfg(test)]
use std::process::exit;

/// A Node is a name that may contain multiple RangeSets and
/// that defnines a machine name. For instance `node[1-14]` is
/// a valid Node defining 14 nodes name from node1 to node14.
/// Let's say that within these nodes we want to reference each
/// core (32 per cpu) of each cpu (2 per node) we could do this
/// writing `node[1-14]-cpu[1-2]-core[1-32]`.
///
/// ```rust
///
/// use nodeset::Node;
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
/// Note : to transform a node into a vector of Strings you may
///        prefer to use `node_to_vec_string()` function.

/*
 * Structure used to keep Node definition
 * * name is the name of the node where everything between brackets
 *        (and the brackets themselves) is replaced by '{}'.
 * * sets is a vector of rangesets: one rangeset per brackets found.
 * * values is used to compute the iterator (and get_next) method
 *          and is a tuple (index, pad) corresponding to the RangeSet
 *          at the same index in the vector
 * * first is also used to compute the iterator and is true until
 *         the first time we pass into the iterator.
 */
#[derive(Debug)]
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

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ErrorKind {
    RegexNoMatch,
    RegexErrorMatch(String),
    RangeSetCreation(String),
}

impl ErrorKind {
    fn as_str(&self) -> &str {
        match *self {
            ErrorKind::RegexNoMatch => "no match found in string",
            ErrorKind::RegexErrorMatch(_) => "matching seems wrong. Verify that ranges are correctly formatted",
            ErrorKind::RangeSetCreation(_) => "unable to create rangeset",
        }
    }
}

impl fmt::Display for NodeErrorType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            NodeErrorType::Regular(ref err) => match err {
                ErrorKind::RegexNoMatch => write!(f, "{}", err.as_str()),
                ErrorKind::RegexErrorMatch(s) => write!(f, "{} '{}'", err.as_str(), s),
                ErrorKind::RangeSetCreation(s) => write!(f, "{} '{}'", err.as_str(), s),
            },
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

/// Transforms a nodeset (String) into a vector of nodes (String)
/// by expanding the created Node structure. This method can be
/// very expensive on memory consumption depending on the number
/// of nodes once that are expanded.
/// ```rust
/// use nodeset::node_to_vec_string;
///
/// let v = node_to_vec_string("r1esw[2-6]").unwrap();
/// assert_eq!(v, ["r1esw2", "r1esw3", "r1esw4", "r1esw5", "r1esw6"]);
/// ```
pub fn node_to_vec_string(node_str: &str) -> Result<Vec<String>, Box<dyn Error>> {
    let node = match Node::new(node_str) {
        Ok(n) => n,
        Err(e) => return Err(Box::new(e)),
    };
    let v: Vec<String> = node.into_iter().collect();
    Ok(v)
}

/* This regular expression is used to capture each rangeset in a string defining a Node */
lazy_static! {
    static ref RE: Regex = Regex::new(r"\[([\d,\-/]+)\]|([\d]+)").unwrap();
}

impl Node {
    /// Counts the number of elements in Node's definition.
    pub fn len(&self) -> u32 {
        if self.sets.is_empty() {
            if self.name.is_empty() {
                0
            } else {
                1
            }
        } else {
            let mut total = 1;
            for r in self.sets.iter() {
                total *= r.len();
            }
            total
        }
    }
    /// Tells whether a Node is empty or not.
    pub fn is_empty(&self) -> bool {
        self.sets.is_empty() && self.name.is_empty()
    }

    /// Transforms a nodeset (String) into a string
    /// by expanding the created Node structure.
    pub fn expand(&self, separator: &str) -> Result<String, Box<dyn Error>> {
        // This is a way to do a clone or a copy as we can not iterate
        // over self for now.
        let node: Node = Node::new(&self.to_string())?;
        let len: usize = node.len().try_into().unwrap();
        let mut to_return = String::new();
        for (i, n) in node.enumerate() {
            if i == len - 1 {
                write!(&mut to_return, "{n}").unwrap();
            } else {
                write!(&mut to_return, "{n}{separator}").unwrap();
            }
        }
        Ok(to_return)
    }

    /// Intersection of self Node with an other Node :
    ///  `node[1,3-5,89]-cpu[2-4]` and `node[9-2,89,101,2-8/2]-cpu[1-3]`
    ///  -> `node[3-5,89]-cpu[2-3]`
    pub fn intersection(&self, other: &Self) -> Option<Node> {
        let mut ns_sets: Vec<RangeSet> = Vec::new();
        let mut values: Vec<(u32, usize)> = Vec::new();

        if self.name != other.name {
            None
        } else {
            for (i, rs_a) in self.sets.iter().enumerate() {
                let rs_b: &RangeSet = &other.sets[i];
                if let Some(inter) = rs_a.intersection(rs_b) {
                    ns_sets.push(inter);
                    values.push((0, 0));
                } else {
                    return None;
                }
            }
            Some(Node {
                name: self.name.to_string(),
                sets: ns_sets,
                values,
                first: false,
            })
        }
    }

    /* Captures with regex all possible (and non overlapping) rangeset in the node name
     * for instance rack[1-8]-node[1-42] should return 1-8 and 1-42 as rangeset
     * It will capture mixed types of rangesets ie: rack1-node[1-42]-cpu2
     */
    fn capture_with_regex(nodename: &str) -> Result<(String, Vec<String>), NodeErrorType> {
        let mut rangesets: Vec<String> = Vec::new();
        let mut name = nodename.to_string();
        for capture in RE.captures_iter(nodename) {
            //println!("capture: {capture:?}");
            match capture.get(1) {
                Some(text) => rangesets.push(text.as_str().to_string()),
                None => {
                    if let Some(text) = capture.get(2) {
                        rangesets.push(text.as_str().to_string())
                    };
                }
            };
        }
        if !rangesets.is_empty() {
            name = RE.replace_all(nodename, "{}").to_string();
        }
        // name that still contains these characters indicates that the nodename is malformed.
        if name.contains('[') || name.contains(']') || name.contains('/') {
            return Err(NodeErrorType::Regular(ErrorKind::RegexErrorMatch(name)));
        }
        //println!("name: {name}");

        Ok((name, rangesets))
    }

    /// Node examples: "node[1-5/2]" or "rack[1,3-5,89]" or "cpu[1-2]core[1-64]" or "node01"
    pub fn new(str: &str) -> Result<Node, NodeErrorType> {
        let (name, rangesets) = Node::capture_with_regex(str)?;
        let mut sets: Vec<RangeSet> = Vec::new();
        let mut values: Vec<(u32, usize)> = Vec::new();
        for set in rangesets {
            let rangeset = match RangeSet::new(&set) {
                Ok(r) => r,
                Err(_) => return Err(NodeErrorType::Regular(ErrorKind::RangeSetCreation(set))),
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
            replaced = nodestr.replacen("{}", format!("{current:0pad$}").as_str(), 1);
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

/// Iterator implementation for Node to allow one to use `for n in node {...}` construction.
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

            // Get the next one and if anxy make the node string
            // out of it and return this.
            self.get_next().map(|_| self.make_node_string())
        }
    }
}

/// FromStr trait lets you write: `let a_node: Node = "node[1-6]-socket[1-2]-core[1-64]".parse().unwrap();`
impl FromStr for Node {
    type Err = NodeErrorType;

    fn from_str(node_str: &str) -> Result<Self, Self::Err> {
        Node::new(node_str)
    }
}

/// PartialEq trait for Node to know if a Node is equal or not
/// to another Node. curr (Iterator's position) is not taken into
/// account. Nodes are equal if name is equal and all RangeSets
/// are equal in the same order (order matters).
impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        if self.name != other.name {
            return false;
        }

        let mut ok: bool = true;
        if self.sets.len() == other.sets.len() {
            for i in 0..self.sets.len() {
                ok = ok && self.sets[i] == other.sets[i]
            }
            ok
        } else {
            false
        }
    }
}

/// Display trait for Node. It will display the node in a folded way (node[1-9/2,98])
impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut nodestr: &str = self.name.as_str();
        let mut replaced;
        for set in &self.sets {
            if set.is_alone() {
                replaced = nodestr.replacen("{}", format!("{set}").as_str(), 1)
            } else {
                replaced = nodestr.replacen("{}", format!("[{set}]").as_str(), 1)
            };
            nodestr = replaced.as_str();
        }
        write!(f, "{nodestr}")
    }
}

/*********************************** Tests ***********************************/

#[cfg(test)] /* Helper function for testing */
fn get_node_values_from_str(node_str: &str) -> Vec<String> {
    let node = match Node::new(node_str) {
        Ok(r) => r,
        Err(e) => {
            println!("Error: {e}");
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
fn testing_creating_node() {
    let node: Node = "node[1-10]".parse().unwrap();
    let rangeset = RangeSet::new("1-10").unwrap();
    assert_eq!(
        node,
        Node {
            name: "node{}".to_string(),
            sets: vec![rangeset],
            values: vec![(0, 0)],
            first: false
        }
    );

    let node: Node = "node[1-10]-cpu[1,2]-core[1-32,34-64]".parse().unwrap();
    let rangeset_a = RangeSet::new("1-10").unwrap();
    let rangeset_b = RangeSet::new("1,2").unwrap();
    let rangeset_c = RangeSet::new("1-32,34-64").unwrap();
    assert_eq!(
        node,
        Node {
            name: "node{}-cpu{}-core{}".to_string(),
            sets: vec![rangeset_a, rangeset_b, rangeset_c],
            values: vec![(0, 0), (0, 0), (0, 0)],
            first: false
        }
    );
    let node: Node = "node[1-10]-cpu[1-2]-core[1-32,34-64]".parse().unwrap();
    let rangeset_a = RangeSet::new("1-10").unwrap();
    let rangeset_b = RangeSet::new("1-2").unwrap();
    let rangeset_c = RangeSet::new("1-32,34-64").unwrap();
    assert_ne!(
        node,
        Node {
            name: "node{}-cpu{}-core{}".to_string(),
            sets: vec![rangeset_c, rangeset_b, rangeset_a],
            values: vec![(0, 0), (0, 0), (0, 0)],
            first: false
        }
    );
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

#[test]
fn testing_node_intersection() {
    let ns_a: Node = "node[1,3-5,89]-cpu[2-4,85-90]".parse().unwrap();
    let ns_b: Node = "node[9-2,89,101,2-8/2]-cpu[1-3,86-92/2]".parse().unwrap();
    // -> node[3-5,89]-cpu[2-3,86-90/2]

    let inter = ns_a.intersection(&ns_b);
    let rs_a = RangeSet::new("3-5,89").unwrap();
    let rs_b = RangeSet::new("2-3,86-90/2").unwrap();
    println!("{inter:?}");
    assert_eq!(
        inter,
        Some(Node {
            name: "node{}-cpu{}".to_string(),
            sets: vec![rs_a, rs_b],
            values: vec![(0, 0), (0, 0)],
            first: false
        })
    );

    let ns_a: Node = "node[1,3-5,89]-cpu[2-4]".parse().unwrap();
    let ns_b: Node = "node[9-2,89,101,2-8/2]-cpu[16-32]".parse().unwrap();
    // -> no intersection !

    let inter = ns_a.intersection(&ns_b);
    println!("{inter:?}");
    assert_eq!(inter, None);
}
