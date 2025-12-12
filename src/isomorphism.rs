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
}
