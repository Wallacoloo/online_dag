//#[cfg(test)]
//mod tests;
//
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::collections::HashSet;
use std::hash::{Hash, Hasher};
use std::rc::Rc;


pub trait Dag<NodeData : Eq, EdgeData : Eq + Hash> {
    type NodeHandle;
    fn add_node(&mut self, node: NodeData) -> Self::NodeHandle;
    fn add_edge(&mut self, from: Self::NodeHandle, to: Self::NodeHandle, data: EdgeData) -> Result<(),()>;
    fn del_edge(&mut self, from: Self::NodeHandle, to: Self::NodeHandle, data: EdgeData) -> Result<(),()>;
}


pub struct DagNodeHandle<T> {
    value : Rc<T>,
}

#[derive(PartialEq, Eq)]
struct DagEdge<NodeData, EdgeData> {
    to: DagNodeHandle<NodeData>,
    user_data: EdgeData,
}

// TODO: use a small-size optimized Set, e.g. smallset
// https://github.com/cfallin/rust-smallset/blob/master/src/lib.rs
type DagEdgeMap<NodeData, EdgeData> = HashMap<DagNodeHandle<NodeData>, HashSet<DagEdge<NodeData, EdgeData>>>;
type DagUnweightedEdgeMap<NodeData> = HashMap<DagNodeHandle<NodeData>, HashSet<DagNodeHandle<NodeData>>>;


pub struct OnDag<NodeData, EdgeData> {
    edges: DagEdgeMap<NodeData, EdgeData>,
    roots: HashSet<DagNodeHandle<NodeData>>,
}

impl <NodeData : Eq, EdgeData : Eq + Hash> Dag<NodeData, EdgeData> for OnDag<NodeData, EdgeData> {
    type NodeHandle = DagNodeHandle<NodeData>;
    fn add_node(&mut self, node: NodeData) -> Self::NodeHandle {
        let handle = Self::NodeHandle::new(node);
        self.roots.insert(handle.clone());
        handle
    }
    fn add_edge(&mut self, from: Self::NodeHandle, to: Self::NodeHandle, data: EdgeData) -> Result<(),()> {
        // if the node was a root, it is no longer.
        self.roots.remove(&to);
        let edge = DagEdge{ to: to, user_data: data };
        self.edges.entry(from)
            .or_insert_with(HashSet::new)
            .insert(edge);
        self.assert_acyclic()
    }
    fn del_edge(&mut self, from: Self::NodeHandle, to: Self::NodeHandle, data: EdgeData) -> Result<(), ()> {
        // TODO: if 'to' no longer has any parents, add it to `roots`
        match self.edges.entry(from) {
            Entry::Vacant(_) => Err(()), // edge was never in the graph
            Entry::Occupied(mut entry) => {
                match entry.get_mut().remove(&DagEdge{ to: to, user_data: data}) {
                    true => {
                        // If there are no more outgoing edges, remove it from the map, allowing
                        // the node to be freed.
                        if entry.get_mut().is_empty() {
                            entry.remove_entry();
                        }
                        Ok(())
                    },
                    false => Err(()), // edge not in graph.
                }
                // TODO: if there are no more outgoing edges on this node, we can delete the set
                // thereby allowing the node to be freed.
            }
        }
    }
}

impl <NodeData : Eq, EdgeData : Eq + Hash> OnDag<NodeData, EdgeData> {
    pub fn new() -> Self {
        OnDag {
            edges: DagEdgeMap::new(),
            roots: HashSet::new(),
        }
    }
    /*
    /// get a copy of the edges, but avoid cloning any user-provided values, by using refs.
    /// Also, omit the edge data.
    fn clone_edges_ref(&self) -> DagUnweightedEdgeMap<NodeData> {
        let mut r = DagUnweightedEdgeMap::new();
        for (ref node, ref edge_set) in self.edges.iter() {
            let mut unweighted_edges = HashSet::new();
            for edge in edge_set.iter() {
                unweighted_edges.insert(edge.to.clone());
            }
            // TODO: why can't we use node.clone() instead of manually filling DagNodeHandle?
            r.insert(DagNodeHandle{ value: node.value.clone() }, unweighted_edges);
        }
        r
    }
    */
    /// Create a map of (child -> {parents}) relationships, without any edge weights.
    fn get_incoming_edgemap(&self) -> DagUnweightedEdgeMap<NodeData> {
        let mut r = DagUnweightedEdgeMap::new();
        for (ref src_node, ref outgoing_edges) in self.edges.iter() {
            for outgoing in outgoing_edges.iter() {
                // TODO: why can't we use src_node.clone() instead of manually filling DagNodeHandle?
                r.entry(outgoing.to.clone()).or_insert_with(HashSet::new)
                    .insert(DagNodeHandle{ value: src_node.value.clone() });
            }
        }
        r
    }
    fn assert_acyclic(&self) -> Result<(), ()> {
        // Kahn's algorithm:
        // init `orphans` to the set of all nodes with no parents.
        // while `orphans` is not empty:
        //   1. delete all edges leaving nodes in `orphans`.
        //   2. For all nodes that just had an incoming edge deleted, if they have no remaining inbound
        //        edges, add them to `orphans`.
        // If there are no remaining edges, then the graph is acyclic.

        let mut orphans = self.roots.clone();
        // maps (child -> {parents})
        let mut incoming_edgemap = self.get_incoming_edgemap();
        while !orphans.is_empty() {
            let mut new_orphans = HashSet::new();
            for parent in orphans.drain() {
                // if the node has outgoing edges, iter them and remove the
                // symmetric incoming edges.
                if let Some(children) = self.edges.get(&parent) {
                    for outgoing_edge in children.iter() {
                        // delete the child -> parent relation
                        // note: unwrap = OK, else the incoming_edgemap was created incorrectly.
                        let mut parents = incoming_edgemap.get_mut(&outgoing_edge.to).unwrap();
                        parents.remove(&parent);
                        if parents.is_empty() {
                            // this node is now an orphan
                            new_orphans.insert(outgoing_edge.to.clone());
                        }
                    }
                }
            }
            orphans = new_orphans;
        }

        // if all nodes have no parents, we have no cycles.
        if incoming_edgemap.iter().all(|(_child, parent_edges)| { parent_edges.is_empty() }) {
            Ok(())
        } else {
            // cycle detected.
            Err(())
        }
    }
    /*fn num_edges(&self) -> usize {
        // sum the number of edges leaving each node:
        self.edges.iter().fold(0, |sum, (_key, val)| {
            sum + val.len()
        })
    }*/
}

impl<T> Clone for DagNodeHandle<T> {
    fn clone(&self) -> Self {
        DagNodeHandle{ value: self.value.clone() }
    }
}

impl<T> Hash for DagNodeHandle<T> {
    fn hash<H>(&self, state: &mut H)  where H: Hasher {
        (&*self.value as *const T).hash(state)
    }
}
impl<T> PartialEq for DagNodeHandle<T> {
    fn eq(&self, other: &Self) -> bool {
        &*self.value as *const T == &*other.value as *const T
    }
}
impl<T> Eq for DagNodeHandle<T> {}

impl<T> DagNodeHandle<T> {
    pub fn new(value: T) -> Self {
        DagNodeHandle{ value: Rc::new(value)}
    }
}


impl<N, E : Hash> Hash for DagEdge<N, E> {
    fn hash<H>(&self, state: &mut H)  where H: Hasher {
        self.to.hash(state);
        self.user_data.hash(state)
    }
}
