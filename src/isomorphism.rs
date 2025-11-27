use crate::permutation::*;

use open_hypergraphs::lax::OpenHypergraph;

use std::collections::HashSet;
use std::hash::Hash;

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

    pub fn new<O: Eq + Clone + Hash, A: Eq + Clone + Hash>(
        f: &OpenHypergraph<O, A>,
        g: &OpenHypergraph<O, A>,
    ) -> Option<Isomorphism> {
        if f == g {
            return Some(Isomorphism::identity(
                f.hypergraph.nodes.len(),
                g.hypergraph.nodes.len(),
            ));
        }

        let (nodes, edges) = find_iso(f, g)?;
        Some(Isomorphism { nodes, edges })
    }
}

////////////////////////////////////////////////////////////////////////////////
// isomorphism for fully-connected open hypergraphs by constraint propagation

#[derive(Clone, PartialEq, Debug)]
pub enum State {
    Unconstrained,
    InSet(HashSet<usize>),
    Exactly(usize),
}

fn find_iso<O: Eq + Clone + Hash, A: Eq + Clone + Hash>(
    f: &OpenHypergraph<O, A>,
    g: &OpenHypergraph<O, A>,
) -> Option<(Permutation, Permutation)> {
    // Run fast nogood checks
    crate::nogood::nogood(f, g)?;

    let n = f.hypergraph.nodes.len();

    // *execute* f as a program where:

    // Node state is a set of constraints where:
    //  None           => completely unconstrained
    //  HashSet<usize> => must be in set
    let _state: Vec<Option<HashSet<usize>>> = vec![None; n];

    // TODO: Initialize known information (interfaces!)

    // TODO: return result
    None
}
