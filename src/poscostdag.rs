use super::ondag::OnDag;
use super::rcdagbase::RcDagBase;

pub use super::rcdagbase::{HalfEdge, FullEdge, NodeHandle, WeakNodeHandle};

pub trait CostQueriable<N, E> {
    /// Return true if the cost of traversing this edge, in the context of traveling to `next`, is 0.
    /// `next` is included because some graphs have internal costs associated with the node -
    /// most graphs won't need to peek at `next`.
    fn is_zero_cost(edge: &HalfEdge<N, E>, next: &HalfEdge<N, E>, dag: &PosCostDag<N, E>) -> bool;
}


/// An Online Dag implementation that DOES allow cycles, provided the cumulative
/// edge weight of any cycles is > 0.
/// 
/// Note: these graphs don't necessarily have explicit roots. It's the user's job to keep handles
/// to root nodes in order to iterate them, etc.
pub struct PosCostDag<N, E> {
    dag: RcDagBase<N, E>,
}

impl <N, E : Eq + CostQueriable<N, E> + Clone> OnDag<N, E> for PosCostDag<N, E> {
    type NodeHandle = NodeHandle<N, E>;
    fn add_node(&mut self, node_data: N) -> Self::NodeHandle {
        self.dag.add_node(node_data)
    }
    fn add_edge(&mut self, from: &Self::NodeHandle, to: &Self::NodeHandle, data: E) -> Result<(),()> {
        // the edge must connect two nodes owned by *this* graph.
        from.check_owner(&self.dag);
        to.check_owner(&self.dag);

        self.dag.add_edge_unchecked(from, to, data.clone());
        let half_edge = HalfEdge::new(to.clone(), data.clone());
        // Theory:
        //  1. Before the new edge, there were no 0-cycles.
        //  2. If the new edge introduces a 0-cycle, that edge must be a component of the cycle.
        //  3. All nodes/edges in a 0-cycle have a 0-cycle to themselves.
        //  4. Therefore, a 0-cycle was introduced to the graph IFF there is a 0-cycle from
        //     the new edge to itself.
        //  Note: 0-cycle = zero cumulative cost cycle.
        if self.is_zero_cost(&half_edge, &half_edge) {
            // This edge introduced a 0-cycle
            self.dag.rm_edge(from, to, data);
            Err(())
        } else {
            // No 0-cycles.
            Ok(())
        }
    }
    fn rm_edge(&mut self, from: &Self::NodeHandle, to: &Self::NodeHandle, data: E) -> Result<(), ()> {
        self.dag.rm_edge(from, to, data);
        Ok(())
    }
}

impl <N, E> PosCostDag<N, E> {
    pub fn new() -> Self {
        PosCostDag {
            dag: RcDagBase::new()
        }
    }
}

impl <N, E : Eq> PosCostDag<N, E> {
    pub fn iter_topo(&self, from: &NodeHandle<N, E>) -> impl Iterator<Item=NodeHandle<N, E>> {
        self.dag.iter_topo(from)
    }
    pub fn iter_topo_rev(&self, from: &NodeHandle<N, E>) -> impl Iterator<Item=NodeHandle<N, E>> {
        self.dag.iter_topo_rev(from)
    }
}

impl <N, E : Eq + CostQueriable<N, E> + Clone> PosCostDag<N, E> {
    fn is_zero_cost(&self, search: &HalfEdge<N, E>, base: &HalfEdge<N, E>) -> bool {
        self.dag.children(base.to()).any(|edge| {
            let is_this_edge_0 = E::is_zero_cost(&base, &edge, &self);
            is_this_edge_0 && (&edge == search || self.is_zero_cost(search, &edge))
        })
    }
}

impl <N, E : Eq + Clone> PosCostDag<N, E> {
    pub fn children(&self, node: &NodeHandle<N, E>) -> impl Iterator<Item=HalfEdge<N, E>> {
        self.dag.children(node)
    }
}
