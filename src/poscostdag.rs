use super::ondag::OnDag;

use super::rcdagbase::{NodeHandle, RcDagBase};

pub trait CostQueriable<Dag> {
    /// Return true if the cost of traversing this edge is 0.
    fn is_zero_cost(&self, dag: &Dag) -> bool;
}

/*
/// Collection of all traits needed for the baseline version of the DAG (i.e.
/// to implement every method of the Dag trait).
trait Edge<N, E> : Eq + CostQueriable<PosCostDag<N, E>> + Clone {}
/// Automatically implement this trait - other modules shouldn't have to know about it.
impl<N, E> Edge<N, E> for E where E : Eq + CostQueriable<PosCostDag<N, E>> + Clone {}
*/

/// An Online Dag implementation that DOES allow cycles, provided the cumulative
/// edge weight of any cycles is > 0.
/// 
/// Note: these graphs don't necessarily have explicit roots. It's the user's job to keep handles
/// to root nodes in order to iterate them, etc.
pub struct PosCostDag<N, E> {
    dag: RcDagBase<N, E>,
}

impl <N, E : Eq + CostQueriable<PosCostDag<N, E>> + Clone> OnDag<N, E> for PosCostDag<N, E> {
    type NodeHandle = NodeHandle<N, E>;
    fn add_node(&mut self, node_data: N) -> Self::NodeHandle {
        self.dag.add_node(node_data)
    }
    fn add_edge(&mut self, from: &Self::NodeHandle, to: &Self::NodeHandle, data: E) -> Result<(),()> {
        // the edge must connect two nodes owned by *this* graph.
        // NOTE: if to IS reachable from from, then &from and &to are at least in the same graph
        // (though not neccessarily this one). TODO: It *would* be better to make this assertion
        // unconditionally.
        // assert_eq!(from.owner, self as *const Self);
        // assert_eq!(to.owner, self as *const Self);
        if data.is_zero_cost(&self) && self.is_zero_cost(&from, &to) {
            // there is a path from `to` to `from`, so adding an edge `from` -> `to` will introduce
            // a cycle.
            Err(())
        } else {
            // add the parent -> child link:
            self.dag.add_edge_unchecked(from, to, data);
            Ok(())
        }
    }
    fn rm_edge(&mut self, from: &Self::NodeHandle, to: &Self::NodeHandle, data: E) -> Result<(), ()> {
        self.dag.rm_edge(from, to, data);
        Ok(())
    }
}

impl <N, E : Eq + CostQueriable<PosCostDag<N, E>> + Clone> PosCostDag<N, E> {
    pub fn new() -> Self {
        PosCostDag {
            dag: RcDagBase::new()
        }
    }
    fn is_zero_cost(&self, search: &NodeHandle<N, E>, base: &NodeHandle<N, E>) -> bool {
        (base == search) || self.dag.children(base).any(|edge| {
            edge.weight().is_zero_cost(&self) && self.is_zero_cost(search, &edge.to())
        })
    }
    pub fn iter_topo(&self, from: &NodeHandle<N, E>) -> impl Iterator<Item=NodeHandle<N, E>> {
        self.dag.iter_topo(from)
    }
    pub fn iter_topo_rev(&self, from: &NodeHandle<N, E>) -> impl Iterator<Item=NodeHandle<N, E>> {
        self.dag.iter_topo_rev(from)
    }
}
