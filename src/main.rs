/* -*- coding: utf8 -*-
 *
 *  lib.rs: Implements all structure logic to deal with nodesets
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

mod range;
mod rangeset;

use crate::range::Range;
use crate::rangeset::RangeSet;
use std::process::exit;

/// rack[10-49]node[1-25/2,78-89,101,1001].panel[0-30/4]
/// Between ',' a Range :
/// * 10-49
/// * 1-25/2,
/// * 78-89,
/// * 101,
/// * 1001
/// * 0-30/4
/// Between '[]' a Set
/// A global name 'rack{}node{}.panel{}' and a vector of sets.

fn print_range(range_str: &str) {
    let range = match Range::new(range_str) {
        Ok(r) => r,
        Err(e) => {
            println!("Error: {}", e);
            exit(1);
        }
    };
    println!("Range: {}", range);
    println!("Range: {:?}", range);
    for i in range {
        println!("{}", i.to_string());
    }
}

fn print_rangeset(rangeset_str: &str) {
    let rangeset = match RangeSet::new(rangeset_str) {
        Ok(r) => r,
        Err(e) => {
            println!("Error: {}", e);
            exit(1);
        }
    };

    println!("RangeSet: {}", rangeset);
    println!("RangeSet: {:?}", rangeset);

    for i in rangeset {
        println!("{}", i);
    }
}

fn main() {
    print_range("1-14/4");
    print_range("38-42");
    print_range("1");
    print_range("097-103");
    print_range("42-38");
    print_rangeset("1,3-5,89");
    print_rangeset("9-2,101,2-8/2");
    print_rangeset("9-2,101,2-8/2");
}
