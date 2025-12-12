use crate::permutation::*;

#[derive(Clone, PartialEq, Debug)]
pub struct Isomorphism {
    nodes: Permutation,
    edges: Permutation,
}

impl Isomorphism {
    pub fn identity(num_nodes: usize, num_edges: usize) -> Self {
        Self {
            nodes: Permutation::identity(num_nodes),
            edges: Permutation::identity(num_edges),
        }
    }

    // TODO: create from two permutations, where
    pub fn new(_nodes: Permutation, _edges: Permutation) -> Option<Isomorphism> {
        // TODO: FIXME: check that nodes and edges actually define an isomorphism of open
        // hypergraphs!
        //Some(Isomorphism { nodes, edges })
        None
    }
}
