use super::ondag::OnDag;
use super::rcdagbase::RcDagBase;

pub use super::rcdagbase::{DagEdge, NodeHandle, WeakNodeHandle};

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
        assert_eq!(from.owner(), &self.dag as *const RcDagBase<N, E>);
        assert_eq!(to.owner(), &self.dag as *const RcDagBase<N, E>);

        self.dag.add_edge_unchecked(from, to, data.clone());
        // Theory:
        //  1. If a new edge introduces a 0-cycle, the cycle MUST have caused the delay of data
        // from some (unknown) node into `to` to decrease. Therefore, any cycle must involve at
        // least one edge going into `to`, and therefore `to` must be in this cycle.
        //  2. Because all edges in a 0-cycle must be zero cost, all nodes in a cycle have a
        //     0-cycle to themselves.
        //  3. Therefore, if this new edge causes a 0-cycle, there exists a 0-cycle from `to` to `to`.
        //  Note: 0-cycle = zero cumulative cost cycle.
        if self.is_zero_cost(&to, &to) {
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

impl <N, E : Eq + CostQueriable<PosCostDag<N, E>> + Clone> PosCostDag<N, E> {
    pub fn new() -> Self {
        PosCostDag {
            dag: RcDagBase::new()
        }
    }
    fn is_zero_cost(&self, search: &NodeHandle<N, E>, base: &NodeHandle<N, E>) -> bool {
        self.dag.children(base).any(|edge| {
            edge.weight().is_zero_cost(&self) && (edge.to() == search || self.is_zero_cost(search, &edge.to()))
        })
    }
    pub fn iter_topo(&self, from: &NodeHandle<N, E>) -> impl Iterator<Item=NodeHandle<N, E>> {
        self.dag.iter_topo(from)
    }
    pub fn iter_topo_rev(&self, from: &NodeHandle<N, E>) -> impl Iterator<Item=NodeHandle<N, E>> {
        self.dag.iter_topo_rev(from)
    }
}

impl <N, E : Eq + Clone> PosCostDag<N, E> {
    pub fn children(&self, node: &NodeHandle<N, E>) -> impl Iterator<Item=DagEdge<N, E>> {
        self.dag.children(node)
    }
}
