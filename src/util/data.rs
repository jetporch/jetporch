use std::collections::HashSet;
use std::collections::HashMap;

pub fn deduplicate(with_duplicates: Vec<String>) -> Vec<String> {
    let mut seen: HashSet<String> = HashSet::new();
    let mut copy: Vec<String> = with_duplicates.iter().map(|x| x.clone()).collect();
    copy.retain(|x| {  let found = seen.contains(x); seen.insert(x.clone()); !found });
    return copy
}

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
