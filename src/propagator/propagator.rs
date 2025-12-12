use open_hypergraphs::lax::{EdgeId, Hyperedge, NodeId, OpenHypergraph};

use std::collections::{HashMap, HashSet};
use std::hash::Hash;

use super::constraint::*;
use super::types::Isomorphism;

// Basic idea: we "execute" the input hypergraph as a "constraint propagator".
// Each edge is thought of as an operation which reads the current constraints on its local
// environments, and further constrains them by looking at the open hypergraph g.
//
// NOTE: TODO: this is an unfinished WIP
//
// // next edge decided by nodes which changed last iteration
// while let f_edge_id = get_next_edge() {
//
//   // This is a custom "update rule" for each edge.
//   // We should have access to know which nodes changed, and also iterate
//   // through them in a chosen order.
//
//   // POSSIBILITY:
//   //  → The combined set of source/target nodes in g implied by f *constrains*
//   //    the set of edges that f could correspond to in g.
//   //  → We get this set of edges, E
//   //  → Each e_g ∈ E implies a set of *constraints*
//
//   // now iterate through nodes in some priority order(?)
//   for (f_node, f_port) in nodes_for(f_edge_id) {
//     let g_nodes = g_index.get_sources(g_node, f_edge_label, f_edge_port);
//
//     // Intersect constraints of f node with all those nodes in g of the same kind
//     *constraints[f_node] *= g_nodes;
//   }
// }

////////////////////////////////////////////////////////////////////////////////
// isomorphism for fully-connected open hypergraphs by constraint propagation

/// Find an isomorphism of open hypergraphs
/// Approach:
///     - Associate a HashSet<usize> to each node in f, representing the possible nodes in g it
///       could correspond to
///     - Initialize this to all nodes for g, but singleton sets for interfaces
///     - Propagate constraints: each operation does a 'local update'
pub fn find_iso<O: Eq + Clone + Hash, A: Eq + Clone + Hash>(
    f: &OpenHypergraph<O, A>,
    g: &OpenHypergraph<O, A>,
) -> Option<Isomorphism> {
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
#[allow(dead_code)]
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

////////////////////////////////////////////////////////////////////////////////
// Faster lookup information

/// Index datastructure for looking up which edges a node is a source or target of.
#[allow(dead_code)]
struct EdgeAdjacencyIndex<'a, O, A> {
    f: &'a OpenHypergraph<O, A>,
    // Key/value pair `(node, edge_label, port) ⇒ edge_id` exists when
    // `f.hypergraph.adjacency[node].sources[port] == node`
    // and
    // `f.hypergraph.edges[edge_id] == edge_label`
    source_node_adjacency: HashMap<(NodeId, A, usize), EdgeId>,
    // Same, but for targets.
    target_node_adjacency: HashMap<(NodeId, A, usize), EdgeId>,
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

    fn get_source(&self, _node_id: &NodeId, _edge_label: &A, _position: usize) -> Vec<EdgeId> {
        todo!("implement EdgeAdjacencyIndex::get_source")
    }

    fn get_target(&self, _node_id: &NodeId, _edge_label: &A, _position: usize) -> Vec<EdgeId> {
        todo!("implement EdgeAdjacencyIndex::get_target")
    }
}
