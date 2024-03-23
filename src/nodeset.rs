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

use crate::node::{Node, NodeErrorType};
use std::error::Error;
use std::fmt;
use std::str::FromStr;

#[derive(Debug)]
pub struct NodeSet {
    set: Vec<Node>,
    current_iter_index: Option<usize>,
}

impl NodeSet {
    /// Counts the number of node in the NodeSet
    pub fn len(&self) -> usize {
        self.set.iter().map(|node| node.len() as usize).sum()
    }

    /// Tells whether a NodeSet is empty or not.
    pub fn is_empty(&self) -> bool {
        self.set.is_empty()
    }

    /// Transforms a nodeset (String) into a string by expanding the Node
    /// structures.
    pub fn expand(&self, separator: &str) -> Result<String, Box<dyn Error>> {
        #[rustfmt::skip]
        let full_output = self.set.iter()
            .map(|node| node.expand(separator))
            .collect::<Result<Vec<String>, Box<dyn Error>>>()?
            .join(separator);

        Ok(full_output)
    }

    /// Intersection of NodeSet with an other NodeSet.
    pub fn intersection(&self, other: &Self) -> Self {
        let mut set = vec![];

        for node in &self.set {
            #[rustfmt::skip]
            set.extend(
                other.set.iter()
                .filter_map(|o| o.intersection(node))
                .collect::<Vec<Node>>()
            );
        }

        Self {
            set,
            current_iter_index: None,
        }
    }

    pub fn optimize(&self) -> Self {
        let mut optimized_set: Vec<Node> = vec![];

        for node in &self.set {
            #[rustfmt::skip]
            let matches: Vec<(usize, Result<_, _>)> = optimized_set.iter()
                .enumerate()
                .map(|(idx, n)| (idx, n.union(&node)))
                .filter(|(_, res)| res.is_ok())
                .collect();

            match matches.len() {
                0 => optimized_set.push(node.clone()),
                1 => {
                    let (index, union) = matches.first().unwrap();
                    optimized_set[index.clone()] = union.as_ref().unwrap().clone();
                }
                _ => unreachable!(),
            }
        }

        Self {
            set: optimized_set,
            current_iter_index: None,
        }
    }

    pub fn new<S: AsRef<str>>(string: S) -> Result<Self, NodeErrorType> {
        // Create a copy of the original string to butcher
        let mut stencil = string.as_ref().to_string();

        // Let the nodes figure out the rangesets, then overwrite them in the copy
        let (_, rangesets) = Node::capture_with_regex(string.as_ref())?;
        for rs in rangesets {
            unsafe {
                stencil = stencil.replace(&rs, &String::from_utf8_unchecked(vec![b'_'; rs.len()]));
            }
        }

        // We can now split using the commas left in the stencil, as they are vetted and not part
        // of a rangeset definition
        let mut set = vec![];
        let mut cursor = 0;
        while cursor < stencil.len() {
            let range;

            match stencil[cursor..].find(',') {
                Some(comma_offset) => {
                    range = cursor..(cursor + comma_offset);
                    cursor += comma_offset + 1
                }
                None => {
                    range = cursor..stencil.len();
                    cursor = usize::max_value();
                }
            }

            set.push(Node::new(&string.as_ref()[range])?);
        }

        Ok(Self {
            set,
            current_iter_index: None,
        }
        .optimize())
    }
}

/// Iterator implementation for NodeSet to allow one to use `for n in node {...}` construction.
impl Iterator for NodeSet {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        let mut global = self.set.iter().flat_map(|node| node.clone().into_iter());

        match self.current_iter_index {
            None => {
                self.current_iter_index = Some(1);
                global.next()
            }
            Some(index) => {
                self.current_iter_index = Some(index + 1);
                global.skip(index).next()
            }
        }
    }
}

/// FromStr trait lets you assign from a static string.
impl FromStr for NodeSet {
    type Err = NodeErrorType;

    fn from_str(node_str: &str) -> Result<Self, Self::Err> {
        NodeSet::new(node_str)
    }
}

/// PartialEq trait for NodeSet. We compare the Nodes in the internal vector.
impl PartialEq for NodeSet {
    fn eq(&self, other: &Self) -> bool {
        if self.set.len() == other.set.len() {
            self.set.iter().zip(other.set.iter()).filter(|&(a, b)| a == b).count() == self.set.len()
        } else {
            false
        }
    }
}

/// Display trait for Node. It will display the nodes in a comma-separated list
impl fmt::Display for NodeSet {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let nodes: Vec<String> = self.set.iter().map(|node| format!("{node}")).collect();
        write!(f, "{}", nodes.join(","))
    }
}

#[test]
fn test_nodeset_creation() {
    let nodeset = NodeSet::new("node[1-10],gpu-node[1-20/2],apu-node[4]").unwrap();
    let node = Node::new("node[1-10]").unwrap();
    let gpu = Node::new("gpu-node[1-20/2]").unwrap();
    let apu = Node::new("apu-node[4]").unwrap();
    assert_eq!(
        nodeset,
        NodeSet {
            set: vec![node, gpu, apu],
            current_iter_index: None,
        }
    );
}

#[test]
fn test_nodeset_creation_optimize() {
    let nodeset = NodeSet::new("node[1-10],gpu-node[1-20/2],node[5-20]").unwrap();
    let node = Node::new("node[1-20]").unwrap();
    let gpu = Node::new("gpu-node[1-20/2]").unwrap();
    assert_eq!(
        nodeset,
        NodeSet {
            set: vec![node, gpu],
            current_iter_index: None,
        }
    );
}

#[test]
fn test_nodeset_expansion() {
    let nodeset = NodeSet::new("node[1-2],gpu-node[1-4/2],apu-node[4]").unwrap();
    assert_eq!(nodeset.expand(",").unwrap(), "node1,node2,gpu-node1,gpu-node3,apu-node4".to_string());
}

#[test]
fn test_nodeset_intersection() {
    let a = NodeSet::new("node[1-50],gpu-node[1-20/5],apu-node[1-1000]").unwrap();
    let b = NodeSet::new("node[50-100],gpu-node[1-20/10],apu-node[500]").unwrap();
    assert_eq!(a.intersection(&b).expand(",").unwrap(), "node50,gpu-node1,gpu-node11,apu-node500".to_string());
}

#[test]
fn test_nodeset_len() {
    let nodeset = NodeSet::new("node[1-2],gpu-node[1-4/2],apu-node[4]").unwrap();
    assert_eq!(nodeset.len(), 5);
}

#[test]
fn test_nodeset_iteration() {
    let nodeset = NodeSet::new("node[1-2],gpu-node[1-4/2],apu-node[4]").unwrap();
    let mut iter = nodeset.into_iter();
    assert_eq!(iter.next(), Some("node1".to_string()));
    assert_eq!(iter.next(), Some("node2".to_string()));
    assert_eq!(iter.next(), Some("gpu-node1".to_string()));
    assert_eq!(iter.next(), Some("gpu-node3".to_string()));
    assert_eq!(iter.next(), Some("apu-node4".to_string()));
    assert_eq!(iter.next(), None);
}

#[test]
fn test_nodeset_equality() {
    let a = NodeSet::new("node[1-2],gpu-node[1-4/2],apu-node[4]").unwrap();
    let b = NodeSet::new("node[1-2],gpu-node[1-4/2],apu-node[4]").unwrap();
    assert_eq!(a, b);
}
