/* -*- coding: utf8 -*-
 *
 *  range.rs: Implements all logic and structures to manage Range
 *
 *  (C) Copyright 2022 - 2023 Olivier Delhomme
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
/// use nodeset::Range;
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
#[derive(Debug, Clone)] /* Auto generates Debug and Clone traits */
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

fn range_step_detection(vector: Vec<u32>) -> u32 {
    let step: u32;

    if vector.len() > 1 {
        if vector[0] < vector[1] {
            step = vector[1] - vector[0];
        } else {
            step = vector[0] - vector[1];
        }
    } else {
        step = 1;
    }
    step
}

/// returns the intersection of two u32 vectors or None
pub fn vec_u32_intersection(first: Vec<u32>, second: Vec<u32>) -> Option<Vec<u32>> {
    let mut inter: Vec<u32> = Vec::new();
    let mut first: Vec<u32> = first;
    let mut second: Vec<u32> = second;

    first.sort_unstable();
    second.sort_unstable();

    //println!("first: {:?}", first);
    //println!("second: {:?}", second);

    let mut i1 = 0;
    let mut i2 = 0;
    while i1 < first.len() && i2 < second.len() {
        //println!("i1:{} i2: {}", i1, i2);
        match first[i1].cmp(&second[i2]) {
            Ordering::Equal => {
                inter.push(first[i1]);
                i1 += 1;
                i2 += 1;
            }
            Ordering::Greater => i2 += 1,
            Ordering::Less => i1 += 1,
        };
    }

    //println!("inter: {:?}", inter);
    if !inter.is_empty() {
        Some(inter)
    } else {
        None
    }
}

// This function needs a non empty sorted Vector of u32.
// It does fold every numbers in the vector into Ranges
// that are put in a vector. This vector contains at
// least one Range.
// pad will be used for all Range in the new Vector
pub fn fold_vec_u32_in_vec_range(v: Vec<u32>, pad: usize) -> Vec<Range> {
    let mut index = 0;
    let mut res: Vec<Range> = Vec::new();

    if v.len() == 1 {
        // only one value in the vector leads to only one Range with
        // start, end and curr at the same value and step to 1 (by convention)
        let range = Range::new_from_values(v[0], v[0], 1, pad, v[0]);
        res.push(range);
        res
    } else {
        // we know that we have at least two values
        // so index + 1 exists
        let mut step = v[index + 1] - v[index];
        let mut diff;
        let mut start = v[index];
        while index + 1 < v.len() {
            // println!("{index}");
            // If we have a third value ahead then begin the loop
            // until the end or until the difference between two
            // values changed
            while index + 2 < v.len() {
                diff = v[index + 2] - v[index + 1];
                if step != diff {
                    // When the difference between the next two values is
                    // not the same as the previous one, the range stops
                    // here (and pushed in the result vector) and a new
                    // one is started.
                    let end = v[index + 1];
                    let range = Range::new_from_values(start, end, step, pad, start);
                    res.push(range);
                    start = v[index + 2];
                    if index + 3 < v.len() {
                        step = v[index + 3] - v[index + 2];
                    } else {
                        step = 1;
                    }
                    break;
                } else {
                    index += 1;
                }
            }
            index += 1;
        }

        let end = v[index];
        let range = Range::new_from_values(start, end, step, pad, start);
        res.push(range);
        res
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
    pub fn len(&self) -> u32 {
        match self.start.cmp(&self.end) {
            Ordering::Greater => 1 + ((self.start - self.end) / self.step),
            Ordering::Less => 1 + ((self.end - self.start) / self.step),
            Ordering::Equal => 1,
        }
    }

    /// An existing range can not be empty -> this function
    /// always returns false
    pub fn is_empty(&self) -> bool {
        false
    }

    /// This function is for internal use of the library.
    /// it returns `curr` field of the Range structure that
    /// is used for the Iterator.
    pub fn get_current(&self) -> u32 {
        self.curr
    }

    /// tells whether the Range is in reverse order
    /// or not
    pub fn is_reverse_order(&self) -> bool {
        self.start > self.end
    }

    pub fn new_range_reversed(&self) -> Range {
        Range {
            start: self.end,
            end: self.start,
            step: self.step,
            pad: self.pad,
            curr: self.curr,
        }
    }

    /// Expands a Range into a vector of u32.
    /// Order is taken into account.
    pub fn generate_vec_u32(&self) -> Vec<u32> {
        let mut vector: Vec<u32> = Vec::new();
        let mut index: u32;

        if self.is_reverse_order() {
            index = self.start;
            while index >= self.end {
                vector.push(index);
                index -= self.step;
            }
        } else {
            index = self.start;
            while index <= self.end {
                vector.push(index);
                index += self.step;
            }
        }

        vector
    }

    /// Returns a new Range that is the union with the other one
    /// Order (reverse or not) is not kept in the new Range
    /// and is always forward
    pub fn union(&self, other: &Self) -> Vec<Range> {
        let mut first: Vec<u32> = self.generate_vec_u32();
        let mut second: Vec<u32> = other.generate_vec_u32();

        let pad = self.pad.max(other.pad);
        first.append(&mut second);
        first.sort_unstable();
        first.dedup();
        fold_vec_u32_in_vec_range(first, pad)
    }

    /// Returns a new Range that is the intersection or None.
    /// Order (reverse or not) is not kept in the new Range
    /// and is always forward
    /// Step detection is always possible because we are in
    /// an intersection of two ranges with stable step propriety
    pub fn intersection(&self, other: &Self) -> Option<Range> {
        let mut first: Vec<u32> = self.generate_vec_u32();
        let mut second: Vec<u32> = other.generate_vec_u32();

        first.sort_unstable();
        second.sort_unstable();

        match vec_u32_intersection(first, second) {
            Some(inter) => {
                let start = inter[0];
                let last = inter.len() - 1;
                let end = inter[last];
                let pad = self.pad.max(other.pad);
                let step = range_step_detection(inter);

                Some(Range {
                    start,
                    end,
                    pad,
                    curr: start,
                    step,
                })
            }
            None => None,
        }
    }

    /// Returns the next value as an `Option<u32>`.
    /// It returns None when there is no next value to
    /// get. Note that Range implements Iterator trait
    /// that you may use in normal cases.
    pub fn get_next(&mut self) -> Option<u32> {
        let curr = self.curr;

        if self.is_reverse_order() {
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

    /// Creates a new Range directly from the values
    /// that defines it: `start-end/step`
    /// pad is the minimal number of number needed: `2` with `Pad = 3` is `002`
    pub fn new_from_values(start: u32, end: u32, step: u32, pad: usize, curr: u32) -> Range {
        Range {
            start,
            end,
            step,
            pad,
            curr,
        }
    }

    /// Creates a new Range with an &str like `1-5/2` or `1` or `9-15`
    /// it may even be in reverse mode such as `15-9`. Padding is
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

        let pad: usize = if start <= end {
            guess_padding(start_str)?
        } else {
            guess_padding(end_str)?
        };

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
        Some(format!("{curr:0pad$}"))
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

        write!(f, "{to_display}")
    }
}

/// PartialEq trait for Range to know if a range is equal or not
/// to another range.
/// padding is not taken into account ie `1-100/2` equals `001-100/2`
/// curr is not taken into account the range is the same anywhere
/// the iterator may be located
impl PartialEq for Range {
    fn eq(&self, other: &Self) -> bool {
        self.start == other.start && self.end == other.end && self.step == other.step
        // && self.pad == other.pad
    }
}

/*********************************** Tests ***********************************/

#[cfg(test)] /* Helper function for testing */
fn get_range_values_from_str(range_str: &str) -> Vec<String> {
    let range = match Range::new(range_str) {
        Ok(r) => r,
        Err(e) => {
            println!("Error: {e}");
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

#[test]
fn testing_range_intersection() {
    let range_a: Range = "1-14/4".parse().unwrap();
    // 1 5 9 13
    let range_b: Range = "3-20/2".parse().unwrap();
    // 3 5 7 9 11 13 15 17 19
    let inter = range_a.intersection(&range_b);
    // 5 9 13
    assert_eq!(
        inter,
        Some(Range {
            start: 5,
            end: 13,
            step: 4,
            pad: 0,
            curr: 5
        })
    );

    let range_a: Range = "38-44".parse().unwrap();
    // 38 39 40 41 42 43 44
    let range_b: Range = "40-36".parse().unwrap();
    // 40 30 38 37 36
    let inter = range_a.intersection(&range_b);
    // 38 39 40
    assert_eq!(
        inter,
        Some(Range {
            start: 38,
            end: 40,
            step: 1,
            pad: 0,
            curr: 38
        })
    );

    let range_a: Range = "1-20/2".parse().unwrap();
    // 1 3 5 7 ...
    let range_b: Range = "2-20/2".parse().unwrap();
    // 2 4 6 8 9 ...
    let inter = range_a.intersection(&range_b);
    assert_eq!(inter, None);

    let range_a: Range = "2-20/2".parse().unwrap();
    // 2 4 6 ... 16 18 20
    let range_b: Range = "20-40/2".parse().unwrap();
    // 40 38 36 ... 24 22 20
    let inter = range_a.intersection(&range_b);
    // 20
    assert_eq!(
        inter,
        Some(Range {
            start: 20,
            end: 20,
            step: 1,
            pad: 0,
            curr: 20
        })
    );

    let range_a: Range = "02-40/2".parse().unwrap();
    // 02 04 06 08 ... 36 38 40
    let range_b: Range = "60-20/3".parse().unwrap();
    // 60 57 54 51 ... 27 24 21
    let inter = range_a.intersection(&range_b);
    // 24 30 36
    assert_eq!(
        inter,
        Some(Range {
            start: 24,
            end: 36,
            step: 6,
            pad: 2,
            curr: 20
        })
    );
}

#[test]
fn testing_range_union() {
    let range_a: Range = "1-14/4".parse().unwrap();
    // 1 5 9 13
    let range_b: Range = "3-20/2".parse().unwrap();
    // 3 5 7 9 11 13 15 17 19
    let inter = range_a.union(&range_b);
    // 1 3 5 9 11 13 15 17 19 -> 1-19/2
    assert_eq!(
        inter,
        vec![Range {
            start: 1,
            end: 19,
            step: 2,
            pad: 0,
            curr: 1
        },]
    );

    let range_a: Range = "38-44".parse().unwrap();
    // 38 39 40 41 42 43 44
    let range_b: Range = "50-56".parse().unwrap();
    // 40 30 38 37 36
    let inter = range_a.union(&range_b);
    //
    assert_eq!(
        inter,
        vec![
            Range {
                start: 38,
                end: 44,
                step: 1,
                pad: 0,
                curr: 38
            },
            Range {
                start: 50,
                end: 56,
                step: 1,
                pad: 0,
                curr: 50
            },
        ]
    );

    let range_a: Range = "1-20/2".parse().unwrap();
    // 1 3 5 7 ...
    let range_b: Range = "2-20/2".parse().unwrap();
    // 2 4 6 8 9 ...
    let inter = range_a.union(&range_b);
    assert_eq!(
        inter,
        vec![Range {
            start: 1,
            end: 20,
            step: 1,
            pad: 0,
            curr: 1
        },]
    );

    let range_a: Range = "2-20/2".parse().unwrap();
    // 2 4 6 ... 16 18 20
    let range_b: Range = "20-40/2".parse().unwrap();
    // 40 38 36 ... 24 22 20
    let inter = range_a.union(&range_b);
    // 20
    assert_eq!(
        inter,
        vec![Range {
            start: 2,
            end: 40,
            step: 2,
            pad: 0,
            curr: 1
        },]
    );

    let range_a: Range = "02-40/2".parse().unwrap();
    // 02 04 06 08 ... 36 38 40
    let range_b: Range = "60-20/3".parse().unwrap();
    // 60 57 54 51 ... 27 24 21
    let inter = range_a.union(&range_b);
    // 02 04 18 20 21 22 24 26 27 28 30 32 33 34 36 38 39 40 42 45 48…
    // at least two possibilities
    // * 02-20/2, 21, 22-26/2, 27, 28-32/2, 33, 34-38/2, 39, 40-42/2, 45-60/3
    // * 02-20/2, 21-22, 24-26/2, 27-28, 30-32/2, 33-34, 36-38/2, 39-40, 42-60/3
    assert_eq!(
        inter,
        vec![
            Range {
                start: 2,
                end: 20,
                step: 2,
                pad: 2,
                curr: 1
            },
            Range {
                start: 21,
                end: 22,
                step: 1,
                pad: 2,
                curr: 21
            },
            Range {
                start: 24,
                end: 26,
                step: 2,
                pad: 2,
                curr: 24
            },
            Range {
                start: 27,
                end: 28,
                step: 1,
                pad: 2,
                curr: 27
            },
            Range {
                start: 30,
                end: 32,
                step: 2,
                pad: 2,
                curr: 30
            },
            Range {
                start: 33,
                end: 34,
                step: 1,
                pad: 2,
                curr: 33
            },
            Range {
                start: 36,
                end: 38,
                step: 2,
                pad: 2,
                curr: 36
            },
            Range {
                start: 39,
                end: 40,
                step: 1,
                pad: 2,
                curr: 39
            },
            Range {
                start: 42,
                end: 60,
                step: 3,
                pad: 2,
                curr: 42
            }
        ]
    );
}
