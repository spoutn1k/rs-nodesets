/* -*- coding: utf8 -*-
 *
 *  range.rs: Implements all logic and structures to manage Range
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

use std::cmp::Ordering;
use std::error::Error;
use std::fmt;

/// A range is composed of ordered numbers (at least one)
/// A range may be incremental or decremental
/// start is the start of the range
/// end is the end of the range
/// step is the step to apply between each increment of this Range
/// pad is a padding to be applied when displaying a Range. It will
///     be calculated only if start number begins with one or more 0.
///     is equal to 0 if no padding has to be applied.
/// curr is used to remember the current value when calculating next
///      number in Range iterator's implementation.
///
/// Range examples:
/// * 10-49
/// * 1-25/2
/// * 78-89
/// * 101
/// * 097-103
/// * 30-0/4
#[derive(Debug)] /* Auto generates Debug trait */
pub struct Range {
    start: u32,
    end: u32,
    step: u32,
    pad: usize,
    curr: u32,
}

impl Range {
    pub fn start_is_end(&self) -> bool {
        self.start == self.end
    }

    pub fn step_is_one(&self) -> bool {
        self.step == 1
    }

    /// "1-5/2" or "1" or "9-15"
    pub fn new(strange: &str) -> Result<Range, Box<dyn Error>> {
        /* Try to figure out if we have a base/step formatted range */
        let (base, step) = match strange.split_once('/') {
            Some((base, step)) => (base, step.parse()?),
            None => (strange, 1),
        };

        /* Base is formatted like start-end or with only one number */
        let (start, end) = match base.split_once('-') {
            Some((start, end)) => (start, end.parse()?),
            None => (base, base.parse()?),
        };

        /* Determining if we need padding, if start begins with zeros */
        /* for example 001 needs padding where as 189 doesn't         */
        let len1 = start.len();
        let number: u32 = start.parse()?;
        let len2 = number.to_string().len();

        let pad: usize = match len1.cmp(&len2) {
            Ordering::Greater => len1,
            _ => 0,
        };

        let start = start.parse()?;
        let curr = start;

        Ok(Range { start, end, step, pad, curr })
    }
}

/// Range iterator returns an already padded String.
impl Iterator for Range {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        let curr = self.curr;

        if self.start > self.end {
            /* going backward here */
            if curr < self.end {
                return None;
            } else {
                self.curr = curr - self.step;
            }
        } else {
            /* going forward here */
            if curr > self.end {
                return None;
            } else {
                self.curr = curr + self.step;
            }
        }
        let pad = self.pad;
        return Some(format!("{:0pad$}", curr));
    }
}

/// Display trait for Range. It will display the range in a folded way
impl fmt::Display for Range {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let pad = self.pad;

        let start_end_str: String = if self.start != self.end { format!("{:0pad$}-{:0pad$}", self.start, self.end) } else { format!("{:0pad$}", self.start) };

        let to_display: String = if self.step != 1 { format!("{}/{}", start_end_str, self.step) } else { start_end_str };

        write!(f, "{}", to_display)
    }
}

/// PartialEq trait for Range to know if a range is equal or not
/// to another range.
/// padding is not taken into account ie 1-100/2 equals 001-100/2
/// curr is not taken into account the range is the same anywhere
/// the iterator may be located
impl PartialEq for Range {
    fn eq(&self, other: &Self) -> bool {
        self.start == other.start && self.end == other.end && self.step == other.step
    }
}

/*** Tests ***/

#[test]
fn testing_creating_range() {
    let range = Range::new("1-10").unwrap();
    assert_eq!(range, Range { start: 1, end: 10, step: 1, pad: 0, curr: 0 });

    let range = Range::new("10-1").unwrap();
    assert_eq!(range, Range { start: 10, end: 1, step: 1, pad: 0, curr: 0 });

    let range = Range::new("1-10/2").unwrap();
    assert_eq!(range, Range { start: 1, end: 10, step: 2, pad: 0, curr: 0 });

    let range = Range::new("10-1/3").unwrap();
    assert_eq!(range, Range { start: 10, end: 1, step: 3, pad: 0, curr: 0 });
}
