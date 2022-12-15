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

#[derive(Debug)] /* Auto generates Debug trait */
pub struct Node {
    name: String,
    set: RangeSet,
}

impl Node {
    /* Node examples: "node[1-5/2]" or "rack[1,3-5,89]" or "cpu[1-64/2]" */
    pub fn new(str: &str) -> Result<Node, Box<dyn Error>> {
        let mut set = RangeSet::empty();
        let splitted: Vec<&str> = str.split_terminator(|c| c == '[' || c == ']').collect();
        let mut i = 1;
        let mut name: String = format!("");

        for range in splitted {
            if i % 2 == 0 {
                set = RangeSet::new(range)?;
            } else {
                name = format!("{}", range);
            }
            i = i + 1 ;
        }
        Ok(Node { name, set })
    }
}
