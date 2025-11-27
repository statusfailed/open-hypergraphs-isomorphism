use crate::permutation::*;

use open_hypergraphs::lax::{EdgeId, Hyperedge, NodeId, OpenHypergraph};

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

/// Find an isomorphism of open hypergraphs
/// Approach:
///     - Associate a HashSet<usize> to each node in f, representing the possible nodes in g it
///       could correspond to
///     - Initialize this to all nodes for g, but singleton sets for interfaces
///     - Propagate constraints: each operation does a 'local update'
fn find_iso<O: Eq + Clone + Hash, A: Eq + Clone + Hash>(
    f: &OpenHypergraph<O, A>,
    g: &OpenHypergraph<O, A>,
) -> Option<(Permutation, Permutation)> {
    // Run fast nogood checks
    crate::nogood::nogood(f, g)?;

    // Node state is a set of constraints where:
    //  None           => completely unconstrained
    //  HashSet<usize> => must be in set
    let n = f.hypergraph.nodes.len();
    let mut state: Vec<State> = vec![State::Any; n];

    // Initialize known information (interfaces!)
    for i in 0..f.sources.len() {
        state[i] = State::single(g.sources[i]);
    }
    for i in 0..f.targets.len() {
        state[i] = State::single(g.targets[i]);
    }

    let mut updated = true;
    while updated {
        updated = false;

        for (edge_id, edge_label, sources, targets) in iter_edges(f, &state) {
            updated = true;
        }
    }

    // TODO: return result
    None
}

/// Iterate through each edge, collecting associated information:
///     - Edge id
///     - Edge label
///     - Source node IDs, types, and values
///     - Target node IDs, types, and values
fn iter_edges<'a, O, A, T>(
    f: &'a OpenHypergraph<O, A>,
    s: &'a Vec<T>,
) -> impl Iterator<
    Item = (
        EdgeId,
        &'a A,
        Vec<(NodeId, &'a O, &'a T)>,
        Vec<(NodeId, &'a O, &'a T)>,
    ),
> + 'a {
    assert_eq!(
        s.len(),
        f.hypergraph.nodes.len(),
        "must have as many state values as nodes"
    );

    (0..f.hypergraph.edges.len()).map(move |edge_id| {
        let edge_label = &f.hypergraph.edges[edge_id];
        let Hyperedge { sources, targets } = &f.hypergraph.adjacency[edge_id];

        let sources = sources
            .iter()
            .map(|i| (*i, &f.hypergraph.nodes[i.0], &s[i.0]))
            .collect();
        let targets = targets
            .iter()
            .map(|i| (*i, &f.hypergraph.nodes[i.0], &s[i.0]))
            .collect();

        (EdgeId(edge_id), edge_label, sources, targets)
    })
}

#[derive(Clone, PartialEq, Debug)]
pub enum State {
    Any,
    Set(HashSet<NodeId>),
}

// TODO: add
impl State {
    fn single(i: NodeId) -> State {
        State::Set(HashSet::from([i]))
    }
}
