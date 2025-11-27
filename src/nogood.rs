use open_hypergraphs::lax::OpenHypergraph;
use std::hash::Hash;

pub(crate) fn nogood<O: Eq + Clone + Hash, A: Eq + Clone + Hash>(
    f: &OpenHypergraph<O, A>,
    g: &OpenHypergraph<O, A>,
) -> Option<()> {
    if !is_sorted_equal(&f.hypergraph.nodes, &g.hypergraph.nodes) {
        return None;
    }

    if !is_sorted_equal(&f.hypergraph.edges, &g.hypergraph.edges) {
        return None;
    }

    // check interfaces are equal sizes and types
    use open_hypergraphs::category::*;
    if f.source() != g.source() {
        return None;
    }

    if f.target() != g.target() {
        return None;
    }

    Some(())
}

/// Check that two vecs are equal once sorted (exact length and elements)
fn is_sorted_equal<T: Eq + Hash>(x: &Vec<T>, y: &Vec<T>) -> bool {
    if x.len() != y.len() {
        return false;
    }

    use std::collections::HashMap;
    let mut counts = HashMap::new();

    for item in x {
        *counts.entry(item).or_insert(0) += 1;
    }

    for item in y {
        match counts.get_mut(item) {
            Some(count) if *count > 0 => *count -= 1,
            _ => return false,
        }
    }

    true
}
