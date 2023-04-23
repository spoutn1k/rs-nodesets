/* -*- coding: utf8 -*-
 *
 *  lib.rs: Implements all structure logic to deal with NodeSets
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
#![doc = include_str!("../README.md")]

/// module to manage node(s). Expanding for instance `node[1-4]` to `node1 node2 node3 node4`
mod node;

/// module to manage range such as `1-4` or `1` or even `30-0/4`
mod range;

/// module to manage a set of range called rangeset such as `1-4,8-14/2,50`
mod rangeset;

pub use node::{Node, node_to_vec_string};
pub use range::{Range, guess_padding, vec_u32_intersection, fold_vec_u32_in_vec_range};
pub use rangeset::RangeSet;
