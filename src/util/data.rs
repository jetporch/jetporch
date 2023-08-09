// Jetporch
// Copyright (C) 2023 - Michael DeHaan <michael@michaeldehaan.net> + contributors
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// at your option) any later version.
// 
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
// 
// You should have received a copy of the GNU General Public License
// long with this program.  If not, see <http://www.gnu.org/licenses/>.

// data.rs: various functions to simplify datastructure handling

use std::collections::HashSet;

// return a vector without duplicate entries
pub fn deduplicate(with_duplicates: Vec<String>) -> Vec<String> {
    let mut seen: HashSet<String> = HashSet::new();
    let mut copy: Vec<String> = with_duplicates.iter().map(|x| x.clone()).collect();
    copy.retain(|x| {  let found = seen.contains(x); seen.insert(x.clone()); !found });
    return copy
}

// this keeps calling a function on results returned by a given function
// the function in question should know when to return empty vectors when
// it gets to the end.

/* 
   return recursive_descent(
        group.clone(), 
        &|x| { get_group_parent_groups(x) },
        0
    );
*/

pub fn recursive_descent(
    root: String,
    related_func: &dyn Fn(String) -> Vec<String>,
    depth: usize) -> Vec<String> {

    if depth > 1000 {
        panic!("maximum depth (1000) exceeded: {}", depth);
    }
     
    let sibling_names = related_func(root);
    let mut results: Vec<String> = Vec::new();
    for node in sibling_names.iter() {
        let descended = recursive_descent(node.clone(), related_func, depth + 1);
        for desc in descended.iter() {
            results.push(desc.clone());
        }
        results.push(node.clone());
    }

    // FIXME: doesn't work?
    return deduplicate(results);
}
