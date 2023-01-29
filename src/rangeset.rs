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

use crate::range::{fold_vec_u32_in_vec_range, vec_u32_intersection, Range};
use std::error::Error;
use std::fmt;
use std::fmt::Write;
use std::str::FromStr;

#[cfg(test)]
use std::process::exit; //used for testing

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

    /// Counts the number of elements in the rangeset
    pub fn len(&self) -> u32 {
        if self.set.is_empty() {
            0
        } else {
            let mut total = 0;
            for r in self.set.iter() {
                total += r.len();
            }
            total
        }
    }

    /// Tells whether a RangeSet is empty or not.
    pub fn is_empty(&self) -> bool {
        self.set.is_empty()
    }

    /// Intersection of self RangeSet with other RangeSet :
    ///  `1,3-5,89` and `9-2,101,2-8/2`
    pub fn intersection(&self, other: &Self) -> Option<RangeSet> {
        // special cases where self or other is empty
        if self.is_empty() {
            return Some(RangeSet {
                set: other.set.clone(),
                curr: other.curr,
            });
        } else if other.is_empty() {
            return Some(RangeSet {
                set: self.set.clone(),
                curr: self.curr,
            });
        }
        // here self and other are not empty so we get at least
        // 2 vectors.

        let mut first: Vec<u32> = Vec::new();
        let mut second: Vec<u32> = Vec::new();
        let mut pad: usize = 0;

        for r in &self.set {
            pad = pad.max(r.get_pad());
            let mut v = r.generate_vec_u32();
            first.append(&mut v);
        }
        for r in &other.set {
            pad = pad.max(r.get_pad());
            let mut v = r.generate_vec_u32();
            second.append(&mut v);
        }

        if let Some(inter) = vec_u32_intersection(first, second) {
            //println!("{:?}", inter);
            let range_vec = fold_vec_u32_in_vec_range(inter, pad);
            //println!("{:?}", range_vec);
            Some(RangeSet {
                set: range_vec,
                curr: 0,
            })
        } else {
            None
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

/// FromStr trait lets you write: `let a_rangeset: RangeSet = "01-10/2,15-30/3".parse().unwrap();`
impl FromStr for RangeSet {
    type Err = Box<dyn Error>;

    fn from_str(strange: &str) -> Result<Self, Self::Err> {
        RangeSet::new(strange)
    }
}

/// PartialEq trait for RangeSet to know if a rangeSet is equal or not
/// to another rangeSet. curr (Iterator's position) is not taken into
/// account. A RangeSet is equal to another one when all ranges are
/// equal to each others in the same order (order matters).
impl PartialEq for RangeSet {
    fn eq(&self, other: &Self) -> bool {
        let mut ok: bool = true;
        if self.set.len() == other.set.len() {
            for i in 0..self.set.len() {
                ok = ok && self.set[i] == other.set[i]
            }
            ok
        } else {
            false
        }
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

/*********************************** Tests ***********************************/

#[cfg(test)] /* Helper function for testing */
fn get_rangeset_values_from_str(rangeset_str: &str) -> Vec<String> {
    let rangeset = match RangeSet::new(rangeset_str) {
        Ok(r) => r,
        Err(e) => {
            println!("Error: {}", e);
            exit(1);
        }
    };
    let mut v: Vec<String> = Vec::new();
    for r in rangeset {
        v.push(r);
    }
    v
}

#[test]
fn testing_creating_rangeset() {
    let rangeset = RangeSet::new("1-10").unwrap();
    let range = Range::new("1-10").unwrap();
    assert_eq!(
        rangeset,
        RangeSet {
            set: vec![range],
            curr: 0
        }
    );

    let rangeset = RangeSet::new("1,2,3-10").unwrap();
    let range_a = Range::new("1").unwrap();
    let range_b = Range::new("2").unwrap();
    let range_c = Range::new("3-10").unwrap();
    assert_eq!(
        rangeset,
        RangeSet {
            set: vec![range_a, range_b, range_c],
            curr: 0
        }
    );

    let rangeset = RangeSet::new("1,2,3-10").unwrap();
    let range_a = Range::new("1").unwrap();
    let range_b = Range::new("2").unwrap();
    let range_c = Range::new("3-10").unwrap();
    assert_ne!(
        rangeset,
        RangeSet {
            set: vec![range_b, range_a, range_c],
            curr: 0
        }
    );
}

#[test]
fn testing_rangeset_values() {
    let value = get_rangeset_values_from_str("1,3-5,89");
    assert_eq!(value, vec!["1", "3", "4", "5", "89"]);

    let value = get_rangeset_values_from_str("9-2,101,2-8/2");
    assert_eq!(value, vec!["9", "8", "7", "6", "5", "4", "3", "2", "101", "2", "4", "6", "8"]);

    let value = get_rangeset_values_from_str("10-01/2,32-72/4");
    assert_eq!(value, vec!["10", "08", "06", "04", "02", "32", "36", "40", "44", "48", "52", "56", "60", "64", "68", "72"]);

    let value = get_rangeset_values_from_str("01-10,7-12/2");
    assert_eq!(value, vec!["01", "02", "03", "04", "05", "06", "07", "08", "09", "10", "7", "9", "11"]);
}

#[test]
fn testing_rangeset_intersection() {
    let rs_a: RangeSet = "1,3-5,89".parse().unwrap();
    // "1", "3", "4", "5", "89"
    let rs_b: RangeSet = "9-2,101,2-8/2,89".parse().unwrap();
    // "9", "8", "7", "6", "5", "4", "3", "2", "101", "2", "4", "6", "8", "89"

    let inter = rs_a.intersection(&rs_b);
    let range_a = Range::new("3-5").unwrap();
    let range_b = Range::new("89").unwrap();
    println!("{:?}", inter);
    assert_eq!(
        inter,
        Some(RangeSet {
            set: vec![range_a, range_b],
            curr: 0
        })
    );

    let rs_a: RangeSet = "10-01/2,32-72/4".parse().unwrap();
    // "10", "08", "06", "04", "02", "32", "36", "40", "44", "48", "52", "56", "60", "64", "68", "72"
    let rs_b: RangeSet = "01-10,7-12/2,50-60/2".parse().unwrap();
    // "01", "02", "03", "04", "05", "06", "07", "08", "09", "10", "7", "9", "11"

    let inter = rs_a.intersection(&rs_b);
    let range_a = Range::new("02-10/2").unwrap();
    let range_b = Range::new("52-60/4").unwrap();
    println!("{:?}", inter);
    assert_eq!(
        inter,
        Some(RangeSet {
            set: vec![range_a, range_b],
            curr: 0
        })
    );
}
