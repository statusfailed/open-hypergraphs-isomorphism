use std::fmt::Debug;

#[derive(Clone, PartialEq, Debug)]
pub struct Permutation(Vec<usize>);

impl Permutation {
    pub fn new(values: impl IntoIterator<Item = usize>) -> Option<Self> {
        let vec: Vec<usize> = values.into_iter().collect();
        
        if vec.is_empty() {
            return Some(Self(vec));
        }
        
        let n = vec.len();
        let mut seen = vec![false; n];
        
        for &value in &vec {
            if value >= n || seen[value] {
                return None;
            }
            seen[value] = true;
        }
        
        if seen.iter().all(|&x| x) {
            Some(Self(vec))
        } else {
            None
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct Isomorphism {
    nodes: Permutation,
    edges: Permutation,
}
