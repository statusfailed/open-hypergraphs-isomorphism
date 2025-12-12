//! Michael McLeod's traversal-based algorithm for isomorphism finding in monogamous connected
//! hypergraphs
use open_hypergraphs::lax::{EdgeId, NodeId, OpenHypergraph};
use std::collections::HashMap;
use std::hash::Hash;

use crate::{Isomorphism, Permutation};

#[derive(Debug)]
pub enum Error {
    /// A nogood check failed
    Nogood,
    NonMonogamous(NodeId),
    Unsatisfiable(NodeId),
    // InvalidMatch(node_f, node_g) means node_f was supposed to correspond to node_g but a
    // constraint was not satisfied
    InvalidNodeMatch(NodeId, NodeId),
    InvalidEdgeMatch(EdgeId, EdgeId),

    // f node had no corresponding g node.
    UnpairedNode(NodeId),
    UnpairedEdge(EdgeId),

    InvalidNodePermutation,
    InvalidEdgePermutation,
}

/// Pseudocode:
///
/// ```text
/// // Mapping f nodes to g nodes
/// let f_to_g: Vec<Option<NodeId>> = vec![None; n];
///
/// // Stack to traverse the hypergraph
/// let todo: Vec<NodeId> = initialize_from_interfaces()
///
/// while let (f_node, g_node) = todo.pop() {
///   check_same_label(f_node, g_node)?;
///
///   // sources: find edge, port of which this is a source
///   if let Some((f_edge_id, f_port)) = f_index.get_source(f_node) {
///     // Look up corresponding edge in g and try to identify
///     let (g_edge_id, g_port) = g_index.get_source(g_node, f_edge_label, f_edge_port)?;
///
///     identify_neighbourhoods(f_edge_id, g_edge_id)
///   }
/// }
/// ```
pub fn find_isomorphism<O: Eq + Clone + Hash, A: Eq + Clone + Hash>(
    f: &OpenHypergraph<O, A>,
    g: &OpenHypergraph<O, A>,
) -> Result<Isomorphism, Error> {
    let state = SearchState::new(f, g)?;
    let (node_mapping, edge_mapping) = state.find_isomorphism()?;

    let nodes = Permutation::new(node_mapping.into_iter().map(|x| x.0));
    let edges = Permutation::new(edge_mapping.into_iter().map(|x| x.0));

    let nodes = nodes.ok_or(Error::InvalidNodePermutation)?;
    let edges = edges.ok_or(Error::InvalidEdgePermutation)?;

    Ok(Isomorphism { nodes, edges })
}

/// Indexes for a pair of open hypergraphs
struct SearchState<'a, O, A> {
    f: &'a OpenHypergraph<O, A>,
    g: &'a OpenHypergraph<O, A>,

    f_index: Index,
    g_index: Index,
}

impl<'a, O: Eq + Clone + Hash, A: Eq + Clone + Hash> SearchState<'a, O, A> {
    pub fn new(
        f: &'a OpenHypergraph<O, A>,
        g: &'a OpenHypergraph<O, A>,
    ) -> Result<SearchState<'a, O, A>, Error> {
        let f_index = Index::new(&f.hypergraph);
        let g_index = Index::new(&g.hypergraph);

        // Verify that source/target nodes are not targets/sources, respectively
        // (monogamicity check!)
        for &source_node in &f.sources {
            if f_index.of_target.contains_key(&source_node) {
                return Err(Error::NonMonogamous(source_node));
            }
        }
        for &target_node in &f.targets {
            if f_index.of_source.contains_key(&target_node) {
                return Err(Error::NonMonogamous(target_node));
            }
        }

        Ok(SearchState {
            f,
            g,
            f_index,
            g_index,
        })
    }

    fn find_isomorphism(&self) -> Result<(Vec<NodeId>, Vec<EdgeId>), Error> {
        // Run fast nogood checks
        crate::nogood::nogood(self.f, self.g).ok_or(Error::Nogood)?;

        let f = self.f;
        let g = self.g;

        let n = f.hypergraph.nodes.len();
        let e = f.hypergraph.edges.len();

        // The set of visited nodes, serving double duty as the assigned mapping to g.
        // Note that we never visit a node twice.
        let mut node_mapping: Vec<Option<NodeId>> = vec![None; n];
        let mut edge_mapping: Vec<Option<EdgeId>> = vec![None; e];

        // "stack" is our priority queue of unvisited f nodes.
        // Each is paired with a single g node.
        // Initialize to the *interfaces* of both open hypergraphs.
        let mut stack = vec![];
        stack.extend(f.sources.iter().copied().zip(g.sources.iter().copied()));
        stack.extend(f.targets.iter().copied().zip(g.targets.iter().copied()));

        // which nodes of f have been visited (either in stack, or in f_to_g)
        // Initialize to interfaces since they're already on the stack
        let mut visited: Vec<bool> = vec![false; n];
        for &source_node in &f.sources {
            visited[source_node.0] = true;
        }
        for &target_node in &f.targets {
            visited[target_node.0] = true;
        }

        // For each proposed pairing of nodes, ...
        while let Some((f_node_id, g_node_id)) = stack.pop() {
            // Check node labels are equal
            if self.f.hypergraph.nodes[f_node_id.0] != self.g.hypergraph.nodes[g_node_id.0] {
                return Err(Error::InvalidNodeMatch(f_node_id, g_node_id));
            }

            // If f_node_id is a source (resp. target) of some edge (monogamicity ⇒ zero or one)
            // then pair that edge with the corresponding one in g (if possible!)
            for (f_index, g_index) in [
                (&self.f_index.of_source, &self.g_index.of_source),
                (&self.f_index.of_target, &self.g_index.of_target),
            ] {
                if let Some((f_edge_id, f_port)) = f_index.get(&f_node_id) {
                    if let Some((g_edge_id, g_port)) = g_index.get(&g_node_id) {
                        // Check g node is at the same source position
                        if f_port != g_port {
                            return Err(Error::InvalidNodeMatch(f_node_id, g_node_id));
                        }

                        // Identify the f/g edges, and update edge mapping
                        self.identify_edges(&mut stack, &mut visited, *f_edge_id, *g_edge_id)?;
                        edge_mapping[f_edge_id.0] = Some(*g_edge_id);
                    } else {
                        return Err(Error::InvalidNodeMatch(f_node_id, g_node_id));
                    }
                }
            }

            // Finally, assign the node to the mapping
            node_mapping[f_node_id.0] = Some(g_node_id);
        }

        // Ensure node mapping is complete
        // We should have now visited all nodes in the hypergraph.
        // If some are unvisited, they must not have been reachable from an interface.
        // In this case, give an error.
        let node_mapping = node_mapping
            .iter()
            .enumerate()
            .map(|(i, g_node)| g_node.ok_or(Error::UnpairedNode(NodeId(i))))
            .collect::<Result<_, _>>()?;

        let edge_mapping = edge_mapping
            .iter()
            .enumerate()
            .map(|(i, g_edge)| g_edge.ok_or(Error::UnpairedEdge(EdgeId(i))))
            .collect::<Result<_, _>>()?;

        Ok((node_mapping, edge_mapping))
    }

    fn identify_edges(
        &self,
        stack: &mut Vec<(NodeId, NodeId)>,
        visited: &mut Vec<bool>,
        f_edge_id: EdgeId,
        g_edge_id: EdgeId,
    ) -> Result<(), Error> {
        // Verify f/g have type (including edge label)
        self.ensure_same_type(f_edge_id, g_edge_id)?;

        // Get neighbourhoods of both edges
        let (f_edge_neighbourhood, g_edge_neighbourhood) =
            self.neighbourhoods(f_edge_id, g_edge_id);

        // Add all pairs to 'stack' and 'visited'
        for (x, y) in f_edge_neighbourhood.iter().zip(g_edge_neighbourhood.iter()) {
            if !visited[x.0] {
                stack.push((*x, *y));
                visited[x.0] = true;
            }
        }

        Ok(())
    }

    /// Ensure that two edges in f and g have the exact same:
    ///  - label
    ///  - source *types*
    ///  - target *types*
    fn ensure_same_type(&self, f_edge_id: EdgeId, g_edge_id: EdgeId) -> Result<(), Error> {
        if self.f.hypergraph.edges[f_edge_id.0] != self.g.hypergraph.edges[g_edge_id.0] {
            return Err(Error::InvalidEdgeMatch(f_edge_id, g_edge_id));
        }

        // Check types are equal
        // TODO: we don't actually have to check types, just arity/coarity.
        // Node labels are checked in main loop!
        let f_adjacency = &self.f.hypergraph.adjacency[f_edge_id.0];
        let g_adjacency = &self.g.hypergraph.adjacency[g_edge_id.0];

        if !self.same_labels(&f_adjacency.sources, &g_adjacency.sources)
            || !self.same_labels(&f_adjacency.targets, &g_adjacency.targets)
        {
            return Err(Error::InvalidEdgeMatch(f_edge_id, g_edge_id));
        }

        Ok(())
    }

    fn same_labels(&self, f_nodes: &[NodeId], g_nodes: &[NodeId]) -> bool {
        let f_labels = f_nodes.iter().map(|s| self.f.hypergraph.nodes[s.0].clone());
        let g_labels = g_nodes.iter().map(|s| self.g.hypergraph.nodes[s.0].clone());
        f_labels.eq(g_labels)
    }

    fn neighbourhoods(&self, f_edge_id: EdgeId, g_edge_id: EdgeId) -> (Vec<NodeId>, Vec<NodeId>) {
        let f_adjacency = self.f.hypergraph.adjacency[f_edge_id.0].clone();
        let g_adjacency = self.g.hypergraph.adjacency[g_edge_id.0].clone();

        (
            [f_adjacency.sources, f_adjacency.targets].concat(),
            [g_adjacency.sources, g_adjacency.targets].concat(),
        )
    }
}

////////////////////////////////////////////////////////////////////////////////
// Indexes used during search

struct Index {
    of_source: HashMap<NodeId, (EdgeId, usize)>,
    of_target: HashMap<NodeId, (EdgeId, usize)>,
}

impl Index {
    fn new<O, A>(hypergraph: &open_hypergraphs::lax::Hypergraph<O, A>) -> Self {
        let mut of_source = HashMap::new();
        let mut of_target = HashMap::new();

        for (edge_id, adjacency) in hypergraph.adjacency.iter().enumerate() {
            let edge_id = EdgeId(edge_id);

            for (port, &node_id) in adjacency.sources.iter().enumerate() {
                of_source.insert(node_id, (edge_id, port));
            }

            for (port, &node_id) in adjacency.targets.iter().enumerate() {
                of_target.insert(node_id, (edge_id, port));
            }
        }

        Index {
            of_source,
            of_target,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, PartialEq, Eq, Debug, Hash)]
    pub enum NodeType {
        Int,
        Float,
    }

    #[derive(Clone, PartialEq, Eq, Debug, Hash)]
    pub enum EdgeOp {
        Cast,
        Negate,
        Mul,
    }

    fn cast_and_negate_then_mul() -> OpenHypergraph<NodeType, EdgeOp> {
        // Create individual operations using singleton
        let cast =
            OpenHypergraph::singleton(EdgeOp::Cast, vec![NodeType::Int], vec![NodeType::Float]);
        let negate =
            OpenHypergraph::singleton(EdgeOp::Negate, vec![NodeType::Float], vec![NodeType::Float]);
        let mul = OpenHypergraph::singleton(
            EdgeOp::Mul,
            vec![NodeType::Float, NodeType::Float],
            vec![NodeType::Float],
        );

        // Compose (cast | negate) >> mul
        let cast_and_negate = &cast | &negate;
        (&cast_and_negate >> &mul).expect("composition should succeed")
    }

    /// open hypergraphs should be isomorphic to themselves
    #[test]
    fn test_find_identity_isomorphism() {
        use open_hypergraphs::category::*;
        let circuits = [
            (
                "id",
                OpenHypergraph::<NodeType, EdgeOp>::identity(vec![NodeType::Float]),
            ),
            (
                "singleton",
                OpenHypergraph::singleton(
                    EdgeOp::Negate,
                    vec![NodeType::Float],
                    vec![NodeType::Float],
                ),
            ),
            ("cntm", cast_and_negate_then_mul()),
        ];

        for (name, circuit) in circuits {
            println!("testing {name}");
            let num_nodes = circuit.hypergraph.nodes.len();
            let num_edges = circuit.hypergraph.edges.len();

            // Test with identity isomorphism
            let expected_isomorphism = Isomorphism::identity(num_nodes, num_edges);

            // Use find_isomorphism to discover the identity isomorphism
            let found_isomorphism =
                find_isomorphism(&circuit, &circuit).expect("should find identity isomorphism");

            // Verify we found the identity isomorphism
            assert_eq!(
                &*found_isomorphism.nodes, &*expected_isomorphism.nodes,
                "Should find identity node permutation"
            );
            assert_eq!(
                &*found_isomorphism.edges, &*expected_isomorphism.edges,
                "Should find identity edge permutation"
            );
        }
    }

    #[test]
    fn test_find_cyclic_isomorphism() {
        let circuit = cast_and_negate_then_mul();

        let num_nodes = circuit.hypergraph.nodes.len();
        let num_edges = circuit.hypergraph.edges.len();

        // Create cyclic permutation (i+1) % n for nodes
        let cyclic_permutation: Vec<usize> = (0..num_nodes).map(|i| (i + 1) % num_nodes).collect();
        let node_permutation = Permutation::new(cyclic_permutation).expect("valid permutation");

        // Create isomorphism with the cyclic node permutation and identity edge permutation
        let expected_isomorphism = Isomorphism {
            nodes: node_permutation,
            edges: Permutation::identity(num_edges),
        };

        // Apply the isomorphism to get the transformed circuit
        let circuit_copy = expected_isomorphism.apply(&circuit);

        // Use find_isomorphism to discover the isomorphism
        let found_isomorphism =
            find_isomorphism(&circuit, &circuit_copy).expect("should find isomorphism");

        // Verify we found the correct isomorphism
        assert_eq!(
            &*found_isomorphism.nodes, &*expected_isomorphism.nodes,
            "Node permutations should match"
        );
        assert_eq!(
            &*found_isomorphism.edges, &*expected_isomorphism.edges,
            "Edge permutations should match"
        );
    }
}
