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
use std::error::Error;
use std::fmt;
use regex::Regex;
use lazy_static::lazy_static;

#[derive(Debug)] /* Auto generates Debug trait */
pub struct Node {
    name: String,
    set: RangeSet,
    suffix: String,
}

#[derive(Debug)]
pub enum NodeErrorType {
    Regular(ErrorKind)
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ErrorKind {
        RegexNoMatch,
        RegexNotTwoMatch,
}


impl ErrorKind {
    fn as_str(&self) -> &str {
        match *self {
            ErrorKind::RegexNoMatch => "no match found in string",
            ErrorKind::RegexNotTwoMatch => "did not match exactly two groups"
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


impl Node {

    fn capture_with_regex(nodename: &str) -> Result<(String, String, String), NodeErrorType> {
        lazy_static! {
            static ref RE: Regex = Regex::new(r"^([[[:alpha:]]_\-]+)\[([\d,\-/]+)\]([\.\-\w]*)").unwrap();
        }
        let (name, rangeset, suffix): (String, String, String) = match RE.captures(nodename) {
            Some(value) => {
                if value.len() >= 3 {
                    if value.len() == 4 {
                        (value[1].to_string(), value[2].to_string(), value[3].to_string())
                    } else {
                        (value[1].to_string(), value[2].to_string(), "".to_string())
                    }
                } else {
                    return Err(NodeErrorType::Regular(ErrorKind::RegexNotTwoMatch));
                }
            },
            None => {
                    /* we did not match node[1-8/2] structure trying to match node01 structure type */
                    lazy_static! {
                        static ref RE: Regex = Regex::new(r"^([[[:alpha:]]_\-]+)([\d]+)").unwrap();
                    }
                    let (name, rangeset, suffix): (String, String, String) = match RE.captures(nodename) {
                        Some(value) => {
                            if value.len() == 3 {
                                if value.len() == 4 {
                                    (value[1].to_string(), value[2].to_string(), value[3].to_string())
                                } else {
                                    (value[1].to_string(), value[2].to_string(), "".to_string())
                                }
                            } else {
                                return Err(NodeErrorType::Regular(ErrorKind::RegexNotTwoMatch));
                            }
                        },
                        None => return Err(NodeErrorType::Regular(ErrorKind::RegexNoMatch)),
                    };
                    (name, rangeset, suffix)
                },
        };

        Ok((name, rangeset, suffix))
    }

    /* Node examples: "node[1-5/2]" or "rack[1,3-5,89]" or "cpu[1-64/2]" or node01 */
    pub fn new(str: &str) -> Result<Node, NodeErrorType> {
        let (name, set, suffix) = Node::capture_with_regex(str)?;
        let rangeset = match RangeSet::new(&set) {
            Ok(r) => r,
            Err(_) => return Err(NodeErrorType::Regular(ErrorKind::RegexNoMatch)),
        };

        Ok(Node { name, set: rangeset, suffix })
    }
}


/// Range iterator returns an already padded String.
impl Iterator for Node {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {

      let next = match self.set.next() {
          Some(v) => v,
          None => return None,
      };
      let nodestr = format!("{}{}{}", self.name, next, self.suffix);
      Some(nodestr)
    }
}

/// Display trait for Node. It will display the node in a folded way
impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.set.is_alone() {
            write!(f, "{}{}{}", self.name, self.set, self.suffix)
        } else {
            write!(f, "{}[{}]{}", self.name, self.set, self.suffix)
        }
    }
}


