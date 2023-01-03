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
use std::str::FromStr;

#[cfg(test)]
use std::process::exit; //used for testing

/// A range is composed of ordered numbers (at least one)
/// A range may be incremental or decremental. Padding is
/// taken into account with the number of zeros found at
/// the beginning of the first number.
///
/// Range examples:
/// * 10-49
/// * 1-25/2
/// * 101
/// * 097-103
/// * 30-0/4
///
/// Example:
/// ```rust
/// use nodeset::range::Range;
/// let range = Range::new("01-15/3");
/// ```

/*
 *  Structure description that may help developers:
 * * `start` is the start of the range
 * * `end` is the end of the range
 * * `step` is the step to apply between each increment of this Range
 * * `pad` is a padding to be applied when displaying a Range. It will
 *         be calculated only if start number begins with one or more 0.
 *         is equal to 0 if no padding has to be applied.
 * * `curr` is used to remember the current value when calculating next
 *          number in Range iterator's implementation.
 */
#[derive(Debug)] /* Auto generates Debug trait */
pub struct Range {
    start: u32,
    end: u32,
    step: u32,
    pad: usize,
    curr: u32,
}

/// "Guess" the padding that is requested by counting the number
/// of characters of the initial string and comparing it with
/// the one generated by getting a new  string from that number.
pub fn guess_padding(value: &str) -> Result<usize, Box<dyn Error>> {
    let len1 = value.len();
    let number: u32 = value.parse()?;
    let len2 = number.to_string().len();

    match len1.cmp(&len2) {
        Ordering::Greater => Ok(len1),
        _ => Ok(0),
    }
}

impl Range {
    /// True when start range is the same as end ie: this range
    /// has only one number.
    pub fn start_is_end(&self) -> bool {
        self.start == self.end
    }

    /// True if the Range is counting one by one. We won't
    /// use /1 to display the Range as this is the "normal"
    /// case ie we write 1-12 instead of 1-12/1
    pub fn step_is_one(&self) -> bool {
        self.step == 1
    }

    /// Resets the Range to its initial value.
    pub fn reset(&mut self) {
        self.curr = self.start;
    }

    /// Returns the padding that applies to the Range.
    pub fn get_pad(&self) -> usize {
        self.pad
    }

    /// counts the number of values in the Range
    pub fn amount(&self) -> u32 {
        match self.start.cmp(&self.end) {
            Ordering::Greater => 1 + ((self.start - self.end) / self.step),
            Ordering::Less => 1 + ((self.end - self.start) / self.step),
            Ordering::Equal => 1,
        }
    }

    /// This function is for internal use of the library.
    /// it returns `curr` field of the Range structure that
    /// is used for the Iterator.
    pub fn get_current(&self) -> u32 {
        self.curr
    }

    /// Returns the next value as an `Option<u32>`.
    /// It returns None when there is no next value to
    /// get. Note that Range implements Iterator trait
    /// that you may use in normal cases.
    pub fn get_next(&mut self) -> Option<u32> {
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
        Some(curr)
    }

    /// Creates a new Range with an &str like "1-5/2" or "1" or "9-15"
    /// it may even be in reverse mode such as "15-9". Padding is
    /// guessed in either mode.
    pub fn new(strange: &str) -> Result<Range, Box<dyn Error>> {
        /* Try to figure out if we have a base/step formatted range */
        let (base, step) = match strange.split_once('/') {
            Some((base, step)) => (base, step.parse()?),
            None => (strange, 1),
        };

        /* Base is formatted like start-end or with only one number */
        let (start_str, end_str) = match base.split_once('-') {
            Some((start, end)) => (start, end),
            None => (base, base),
        };

        /* Determining if we need padding, if start begins with zeros    */
        /* for example 001 needs padding where as 189 doesn't            */
        /* Padding is also guessed in reverse mode: 100-080 will produce */
        /* 100 099 098...                                                */
        let start = start_str.parse()?;
        let end = end_str.parse()?;
        let pad: usize;

        if start <= end {
            pad = guess_padding(start_str)?;
        } else {
            pad = guess_padding(end_str)?;
        }

        let curr = start;

        Ok(Range {
            start,
            end,
            step,
            pad,
            curr,
        })
    }
}

/// Range iterator returns an already padded String.
impl Iterator for Range {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        let curr = match self.get_next() {
            Some(value) => value,
            None => return None,
        };
        let pad = self.pad;
        return Some(format!("{:0pad$}", curr));
    }
}

/// FromStr trait lets you write: `let a_range: Range = "01-10/2".parse().unwrap();`
impl FromStr for Range {
    type Err = Box<dyn Error>;

    fn from_str(strange: &str) -> Result<Self, Self::Err> {
        Range::new(strange)
    }
}

/// Display trait for Range. It will display the range in a folded way: 01-18/3.
impl fmt::Display for Range {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let pad = self.pad;

        let start_end_str: String = if self.start != self.end {
            format!("{:0pad$}-{:0pad$}", self.start, self.end)
        } else {
            format!("{:0pad$}", self.start)
        };

        let to_display: String = if self.step != 1 {
            format!("{}/{}", start_end_str, self.step)
        } else {
            start_end_str
        };

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

/*********************************** Tests ***********************************/

#[cfg(test)] /* Helper function for testing */
fn get_range_values_from_str(range_str: &str) -> Vec<String> {
    let range = match Range::new(range_str) {
        Ok(r) => r,
        Err(e) => {
            println!("Error: {}", e);
            exit(1);
        }
    };
    let mut v: Vec<String> = Vec::new();
    for r in range {
        v.push(r);
    }
    v
}

#[test]
fn testing_creating_range() {
    let range = Range::new("1-10").unwrap();
    assert_eq!(
        range,
        Range {
            start: 1,
            end: 10,
            step: 1,
            pad: 0,
            curr: 0
        }
    );

    let range = Range::new("10-1").unwrap();
    assert_eq!(
        range,
        Range {
            start: 10,
            end: 1,
            step: 1,
            pad: 0,
            curr: 0
        }
    );

    let range = Range::new("1-10/2").unwrap();
    assert_eq!(
        range,
        Range {
            start: 1,
            end: 10,
            step: 2,
            pad: 0,
            curr: 0
        }
    );

    let range = Range::new("10-1/3").unwrap();
    assert_eq!(
        range,
        Range {
            start: 10,
            end: 1,
            step: 3,
            pad: 0,
            curr: 0
        }
    );
}

#[test]
fn testing_range_values() {
    let value = get_range_values_from_str("1-14/4");
    assert_eq!(value, vec!["1", "5", "9", "13"]);

    let value = get_range_values_from_str("38-42");
    assert_eq!(value, vec!["38", "39", "40", "41", "42"]);

    let value = get_range_values_from_str("1");
    assert_eq!(value, vec!["1"]);

    let value = get_range_values_from_str("097-103");
    assert_eq!(value, vec!["097", "098", "099", "100", "101", "102", "103"]);

    let value = get_range_values_from_str("42-38");
    assert_eq!(value, vec!["42", "41", "40", "39", "38"]);
}
