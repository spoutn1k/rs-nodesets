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

/// A RangeSet is a vector of Range.
/// Unlike Range a RangeSet may not be ordered. Specified order is
/// kept when generating numbers with the iterator.
/// * `set` is the vector of Range. It may be empty.
/// * `curr` is used remember the current index in the vector of Ranges
///      and is used to calculate next number in RangeSet iterator's
///      implementation.
///
/// RangeSet examples:
/// * "1,3-5,89"
/// * "9-2,101,2-8/2"
///
/// Example:
/// ```rust
/// use nodeset::rangeset::RangeSet;
/// let rangeset = RangeSet::new("22-28/2,29");
/// ```
#[derive(Debug)] /* Auto generates Debug trait */
pub struct RangeSet {
    set: Vec<Range>,
    curr: usize,
}

impl RangeSet {
    /// True when we only have one member and not a set ie: node003
    pub fn is_alone(&self) -> bool {
        self.set.len() == 1 && self.set[0].start_is_end() && self.set[0].step_is_one()
    }

    pub fn reset(&mut self) {
        self.curr = 0;
        for i in 0..self.set.len() {
            self.set[i].reset()
        }
    }

    pub fn get_current(&self) -> (u32, usize) {
        let index = self.curr;
        let pad = self.set[index].get_pad();

        (self.set[index].get_current(), pad)
    }

    pub fn amount(&self) -> u32 {
        if self.set.is_empty() {
            0
        } else {
            let mut total = 0;
            for r in self.set.iter() {
                total += r.amount();
            }
            total
        }
    }

    pub fn get_next(&mut self) -> Option<(u32, usize)> {
        let index = self.curr;
        let mut pad = self.set[index].get_pad();

        let next = match self.set[index].get_next() {
            Some(number) => number, // gives next number in Range range.
            None => {
                /* This tells us that range Range is finished : need to iter over next range. */
                if index + 1 < self.set.len() {
                    /* There is another Range in the vector */
                    self.curr = index + 1;
                    pad = self.set[self.curr].get_pad();
                    match self.set[self.curr].get_next() {
                        Some(number) => number,
                        None => return None,
                    }
                } else {
                    /* There is no other Range in the vector */
                    return None;
                }
            }
        };
        Some((next, pad))
    }

    /// "[1-5/2]" or "[1,3-5,89]" or "[9-15/3,4,9-2]"
    pub fn new(strange: &str) -> Result<RangeSet, Box<dyn Error>> {
        let mut set: Vec<Range> = Vec::new();
        let rangeset: Vec<&str> = strange.split(',').collect();
        let curr = 0;

        for rs in rangeset {
            let range = match Range::new(rs) {
                Ok(r) => r,
                Err(e) => return Err(e),
            };
            set.push(range);
        }
        Ok(RangeSet {
            set,
            curr,
        })
    }

    pub fn empty() -> RangeSet {
        let set: Vec<Range> = Vec::new();
        let curr = 0;

        RangeSet {
            set,
            curr,
        }
    }
}

/// RangeSet iterator returns an already padded String as Range does.
impl Iterator for RangeSet {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        let (next_u32, pad) = match self.get_next() {
            Some(v) => v,
            None => return None,
        };

        let next = format!("{:0pad$}", next_u32);
        Some(next)
    }
}

/// Display trait for RangeSet. It will display the RangeSet in a folded way
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
