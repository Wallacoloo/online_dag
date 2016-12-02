/// Implements the OnDag trait, which defines the general interface
/// that all DAG implementations need implement.
/// Note: some implementations may require N or E to also be hashable, cloneable, or orderable.
pub trait OnDag<N, E> {
    type NodeHandle;
    type EdgeHandle;
    fn add_node(&mut self, node: N) -> Self::NodeHandle;
    fn add_edge(&mut self, from: &Self::NodeHandle, to: &Self::NodeHandle, data: E) -> Result<(),()>;
    fn rm_edge(&mut self, from: &Self::NodeHandle, to: &Self::NodeHandle, data: E) -> Result<(),()>;
    // fn iter_topo(&self, from: &NodeHandle) -> impl Iterator<Item=Self::NodeHandle>
    // fn iter_topo_rev(&self, from: &NodeHandle) -> impl Iterator<Item=Self::NodeHandle>
    // fn children(&self, node: &NodeHandle) -> impl Iterator<Item=Edge>
}
