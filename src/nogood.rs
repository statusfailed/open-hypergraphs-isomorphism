use open_hypergraphs::lax::OpenHypergraph;
use std::hash::Hash;

// TODO: also check arity/coarity and types of each edge
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_sorted_equal_empty_vectors() {
        let x: Vec<i32> = vec![];
        let y: Vec<i32> = vec![];
        assert!(is_sorted_equal(&x, &y));
    }

    #[test]
    fn test_is_sorted_equal_single_element() {
        let x = vec![1];
        let y = vec![1];
        assert!(is_sorted_equal(&x, &y));
    }

    #[test]
    fn test_is_sorted_equal_different_lengths() {
        let x = vec![1, 2];
        let y = vec![1];
        assert!(!is_sorted_equal(&x, &y));
    }

    #[test]
    fn test_is_sorted_equal_same_elements_different_order() {
        let x = vec![3, 1, 2];
        let y = vec![2, 3, 1];
        assert!(is_sorted_equal(&x, &y));
    }

    #[test]
    fn test_is_sorted_equal_different_elements() {
        let x = vec![1, 2, 3];
        let y = vec![1, 2, 4];
        assert!(!is_sorted_equal(&x, &y));
    }

    #[test]
    fn test_is_sorted_equal_with_duplicates() {
        let x = vec![1, 2, 2, 3];
        let y = vec![3, 2, 1, 2];
        assert!(is_sorted_equal(&x, &y));
    }

    #[test]
    fn test_is_sorted_equal_different_duplicate_counts() {
        let x = vec![1, 2, 2];
        let y = vec![1, 1, 2];
        assert!(!is_sorted_equal(&x, &y));
    }

    #[test]
    fn test_is_sorted_equal_strings() {
        let x = vec!["hello", "world", "rust"];
        let y = vec!["rust", "hello", "world"];
        assert!(is_sorted_equal(&x, &y));
    }
}
