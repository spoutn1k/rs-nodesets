/* -*- coding: utf8 -*-
 *
 *  rangeset.rs: Implements all structure and logic to manage RangeSet
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

use crate::range::Range;
use std::error::Error;
use std::fmt;
use std::fmt::Write;

/// A RangeSet is a vector of Range. Unlike Range a RangeSet
/// may not be ordered. Specified order is kept when generating numbers
/// with the iterator.
/// set is the vector of Range. It may be empty.
/// curr is used remember the current index in the vector of Ranges
///      and is used to calculate next number in RangeSet iterator's
///      implementation.
///
/// RangeSet examples:
/// * 1,3-5,89
/// * 9-2,101,2-8/2
///
#[derive(Debug)] /* Auto generates Debug trait */
pub struct RangeSet {
    set: Vec<Range>,
    curr: usize,
}

impl RangeSet {
    /// "[1-5/2]" or "[1,3-5,89]" or "[9-15/3]"
    pub fn new(strange: &str) -> Result<RangeSet, Box<dyn Error>> {
        let mut set: Vec<Range> = Vec::new();
        let rangeset: Vec<&str> = strange.split(",").collect();
        let curr = 0;

        for rs in rangeset {
            let range = match Range::new(rs) {
                Ok(r) => r,
                Err(e) => return Err(e),
            };
            set.push(range);
        }
        Ok(RangeSet { set, curr })
    }
}

impl Iterator for RangeSet {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        let index = self.curr;

        let next: Option<Self::Item> = match self.set[index].next() {
            Some(number) => Some(number), // gives next number in Range range.
            None => {
                /* This tells us that range Range is finished : need to iter over next range. */
                if index + 1 < self.set.len() {
                    /* There is another Range in the vector */
                    self.curr = index + 1;
                    self.set[self.curr].next()
                } else {
                    /* There is no other Range in the vector */
                    None
                }
            }
        };

        return next;
    }
}

/// Display trait for RangeSet. It will display the range in a folded way
impl fmt::Display for RangeSet {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut to_display = String::new();
        let len = self.set.len();

        for (i, range) in self.set.iter().enumerate() {
            if i == len - 1 {
                write!(&mut to_display, "{}", range).unwrap();
            } else {
                write!(&mut to_display, "{},", range).unwrap();
            }
        }

        write!(f, "{}", to_display)
    }
}

