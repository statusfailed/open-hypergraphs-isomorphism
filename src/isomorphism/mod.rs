use crate::permutation::*;

use open_hypergraphs::lax::{EdgeId, Hyperedge, NodeId, OpenHypergraph};

use std::collections::{HashMap, HashSet};
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
    let e = f.hypergraph.edges.len();

    // Create some fast lookup information
    let index = EdgeAdjacencyIndex::new(f);

    let mut nodes: Vec<Constraint<NodeId>> = vec![Constraint::Any; n];
    let mut edges: Vec<Constraint<EdgeId>> = vec![Constraint::Any; e];

    // Initialize known information (interfaces!)
    for i in 0..f.sources.len() {
        nodes[i] = Constraint::single(g.sources[i]);
    }
    for i in 0..f.targets.len() {
        nodes[i] = Constraint::single(g.targets[i]);
    }

    let mut updated = true;
    while updated {
        updated = false;

        // PERFORMANCE: only visit those edges which updated a node in their local neighbourhood
        for edge_id in 0..edges.len() {
            let Hyperedge { sources, targets } = &f.hypergraph.adjacency[edge_id];
            let edge_label = &f.hypergraph.edges[edge_id];

            // For each source node of edge_id,
            let mut possible_edges: HashSet<EdgeId> = sources
                .iter()
                .enumerate()
                // TODO: map to all possible g nodes, as constrained by
                .map(|(i, s)| index.get_source(s, edge_label, i))
                .flatten()
                .collect::<HashSet<_>>();

            // Extend with targets
            possible_edges.extend(
                targets
                    .iter()
                    .enumerate()
                    .map(|(i, s)| index.get_target(s, edge_label, i))
                    .flatten(),
            );

            // Set possible edges
            edges[edge_id].intersection(possible_edges);

            // For each possible edge,
        }

        /*
        for (edge_id, edge_label, sources, targets) in iter_edges(f, &nodes) {
            // For each source node, figure out the "implied targets" in g
            for (source_id, _, g_nodes) in sources {
                // Look up all g edges having source_id as a source node
                for g_edge in has_source(g, source_id) {
                    if same_signature(f, g, f_edge_id, g_edge_id) {
                        candidates.push(todo!());
                    }
                }
            }

            for (target_id, _, g_nodes) in targets {
                todo!()
            }

            updated = true;
        }
        */
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
enum Constraint<T: Hash + Eq> {
    Any,
    Set(HashSet<T>),
}

impl<T: Hash + Eq> Constraint<T> {
    fn single(x: T) -> Constraint<T> {
        Constraint::Set(HashSet::from([x]))
    }
}

impl<T: Clone + Hash + Eq> Constraint<T> {
    fn intersection(&mut self, s: HashSet<T>) -> Constraint<T> {
        match self {
            Self::Any => Self::Set(s),
            Self::Set(t) => Self::Set(s.intersection(&t).cloned().collect()),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////
// Faster lookup information

/// Index datastructure for looking up which edges a node is a source or target of.
struct EdgeAdjacencyIndex<'a, O, A> {
    f: &'a OpenHypergraph<O, A>,
    // Key/value pair `(node, edge_label, port) ⇒ edge_id` exists when
    // `f.hypergraph.adjacency[node].sources[port] == node`
    // and
    // `f.hypergraph.edges[edge_id] == edge_label`
    pub source_node_adjacency: HashMap<(NodeId, A, usize), EdgeId>,
    // Same, but for targets.
    pub target_node_adjacency: HashMap<(NodeId, A, usize), EdgeId>,
}

impl<'a, O, A> EdgeAdjacencyIndex<'a, O, A> {
    fn new(f: &'a OpenHypergraph<O, A>) -> Self {
        // TODO: create these hashmaps correctly
        let source_node_adjacency = HashMap::new();
        let target_node_adjacency = HashMap::new();

        EdgeAdjacencyIndex {
            f,
            source_node_adjacency,
            target_node_adjacency,
        }
    }

    fn get_source(&self, node_id: &NodeId, edge_label: &A, position: usize) -> Vec<EdgeId> {
        todo!("implement EdgeAdjacencyIndex::get_source")
    }

    fn get_target(&self, node_id: &NodeId, edge_label: &A, position: usize) -> Vec<EdgeId> {
        todo!("implement EdgeAdjacencyIndex::get_target")
    }
}
