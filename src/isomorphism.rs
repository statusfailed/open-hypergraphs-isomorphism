use crate::permutation::*;
use open_hypergraphs::lax::OpenHypergraph;

#[derive(Clone, PartialEq, Debug)]
pub struct Isomorphism {
    pub nodes: Permutation,
    pub edges: Permutation,
}

impl Isomorphism {
    pub fn identity(num_nodes: usize, num_edges: usize) -> Self {
        Self {
            nodes: Permutation::identity(num_nodes),
            edges: Permutation::identity(num_edges),
        }
    }

    // TODO: create from two permutations, where
    pub fn validate<O: Eq, A: Eq>(
        &self,
        f: &OpenHypergraph<O, A>,
        g: &OpenHypergraph<O, A>,
    ) -> bool {
        // Check node labels preserved
        for (f_idx, &g_idx) in (&self.nodes).iter().enumerate() {
            if f.hypergraph.nodes[f_idx] != g.hypergraph.nodes[g_idx] {
                return false;
            }
        }

        // Check edge labels preserved
        for (f_idx, &g_idx) in (&self.edges).iter().enumerate() {
            if f.hypergraph.edges[f_idx] != g.hypergraph.edges[g_idx] {
                return false;
            }
        }

        // Check adjacency structure preserved
        for (f_edge_idx, &g_edge_idx) in (&self.edges).iter().enumerate() {
            let f_adjacency = &f.hypergraph.adjacency[f_edge_idx];
            let g_adjacency = &g.hypergraph.adjacency[g_edge_idx];

            // Check sources match under node permutation
            if f_adjacency.sources.len() != g_adjacency.sources.len() {
                return false;
            }
            for (port, &f_node_idx) in f_adjacency.sources.iter().enumerate() {
                if self.nodes[f_node_idx.0] != g_adjacency.sources[port].0 {
                    return false;
                }
            }

            // Check targets match under node permutation
            if f_adjacency.targets.len() != g_adjacency.targets.len() {
                return false;
            }
            for (port, &f_node_idx) in f_adjacency.targets.iter().enumerate() {
                if self.nodes[f_node_idx.0] != g_adjacency.targets[port].0 {
                    return false;
                }
            }
        }

        // Check interfaces are compatible under node permutation
        if f.sources.len() != g.sources.len() || f.targets.len() != g.targets.len() {
            return false;
        }

        // Check sources are mapped correctly
        for (port, &f_node_idx) in f.sources.iter().enumerate() {
            if self.nodes[f_node_idx.0] != g.sources[port].0 {
                return false;
            }
        }

        // Check targets are mapped correctly
        for (port, &f_node_idx) in f.targets.iter().enumerate() {
            if self.nodes[f_node_idx.0] != g.targets[port].0 {
                return false;
            }
        }

        true
    }

    pub fn apply<O: Clone, A: Clone>(&self, f: &OpenHypergraph<O, A>) -> OpenHypergraph<O, A> {
        let mut result = f.clone();

        // Apply node permutation to node labels
        let original_nodes = f.hypergraph.nodes.clone();
        for (i, &perm_idx) in (&self.nodes).iter().enumerate() {
            result.hypergraph.nodes[perm_idx] = original_nodes[i].clone();
        }

        // Apply edge permutation to edge labels
        let original_edges = f.hypergraph.edges.clone();
        for (i, &perm_idx) in (&self.edges).iter().enumerate() {
            result.hypergraph.edges[perm_idx] = original_edges[i].clone();
        }

        // Update adjacency structure with node permutation
        for adjacency in &mut result.hypergraph.adjacency {
            for source_node in &mut adjacency.sources {
                source_node.0 = self.nodes[source_node.0];
            }
            for target_node in &mut adjacency.targets {
                target_node.0 = self.nodes[target_node.0];
            }
        }

        // Update interface indices with node permutation
        for source in &mut result.sources {
            source.0 = self.nodes[source.0];
        }
        for target in &mut result.targets {
            target.0 = self.nodes[target.0];
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, PartialEq, Eq, Debug)]
    pub enum NodeType {
        Int,
        Float,
    }

    #[derive(Clone, PartialEq, Eq, Debug)]
    pub enum EdgeOp {
        Cast,
        Negate,
        Mul,
    }

    fn cast_or_negate_then_mul() -> OpenHypergraph<NodeType, EdgeOp> {
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
        let cast_or_negate = &cast | &negate;
        (&cast_or_negate >> &mul).expect("composition should succeed")
    }

    #[test]
    fn test_identity_isomorphism_validation() {
        let circuit = cast_or_negate_then_mul();

        // Create identity isomorphism
        let num_nodes = circuit.hypergraph.nodes.len();
        let num_edges = circuit.hypergraph.edges.len();
        let identity = Isomorphism::identity(num_nodes, num_edges);

        // Validate that the identity isomorphism is valid for (circuit, circuit)
        assert!(
            identity.validate(&circuit, &circuit),
            "Identity isomorphism should be valid"
        );
    }

    #[test]
    fn test_cyclic_node_permutation_validation() {
        let circuit = cast_or_negate_then_mul();

        let num_nodes = circuit.hypergraph.nodes.len();
        let num_edges = circuit.hypergraph.edges.len();

        // Create cyclic permutation (i+1) % n for nodes
        let cyclic_permutation: Vec<usize> = (0..num_nodes).map(|i| (i + 1) % num_nodes).collect();
        let node_permutation = Permutation::new(cyclic_permutation).expect("valid permutation");

        // Create isomorphism with the cyclic node permutation and identity edge permutation
        let isomorphism = Isomorphism {
            nodes: node_permutation,
            edges: Permutation::identity(num_edges),
        };

        // Apply the isomorphism to get the transformed circuit
        let circuit_copy = isomorphism.apply(&circuit);

        // Validate that the isomorphism correctly maps circuit to circuit_copy
        assert!(
            isomorphism.validate(&circuit, &circuit_copy),
            "Cyclic node permutation should be valid"
        );
    }
}
