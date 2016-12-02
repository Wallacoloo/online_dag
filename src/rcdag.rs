use super::ondag::OnDag;
use super::rcdagbase::RcDagBase;

pub use super::rcdagbase::{DagEdge, NodeHandle};



/// Note: these graphs don't necessarily have explicit roots. It's the user's job to keep handles
/// to root nodes in order to iterate them, etc.
pub struct RcDag<N, E> {
    dag: RcDagBase<N, E>,
}

impl <N, E : Eq> OnDag<N, E> for RcDag<N, E> {
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
        if self.dag.is_reachable(&from, &to) {
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

impl <N, E : Eq> RcDag<N, E> {
    pub fn new() -> Self {
        RcDag {
            dag: RcDagBase::new()
        }
    }
    pub fn iter_topo(&self, from: &NodeHandle<N, E>) -> impl Iterator<Item=NodeHandle<N, E>> {
        self.dag.iter_topo(from)
    }
    pub fn iter_topo_rev(&self, from: &NodeHandle<N, E>) -> impl Iterator<Item=NodeHandle<N, E>> {
        self.dag.iter_topo_rev(from)
    }
}

impl <N, E : Eq + Clone> RcDag<N, E> {
    pub fn children(&self, node: &NodeHandle<N, E>) -> impl Iterator<Item=DagEdge<N, E>> {
        self.dag.children(node)
    }
}
