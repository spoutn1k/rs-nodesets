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
