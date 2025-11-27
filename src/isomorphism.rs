use crate::permutation::*;
use open_hypergraphs::lax::OpenHypergraph;

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

    pub fn new<O: PartialEq, A: PartialEq>(
        f: OpenHypergraph<O, A>,
        g: OpenHypergraph<O, A>,
    ) -> Isomorphism {
        if f == g {
            return Isomorphism::identity(f.hypergraph.nodes.len(), g.hypergraph.nodes.len());
        }

        todo!()
    }
}
