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

// TODO: use a small-size optimized Set, e.g. smallset
// https://github.com/cfallin/rust-smallset/blob/master/src/lib.rs
type DagEdgeMap<NodeData, EdgeData> = HashMap<DagNodeHandle<NodeData>, HashSet<DagEdge<NodeData, EdgeData>>>;


pub struct DagNodeHandle<T> {
    value : Rc<T>,
}

#[derive(PartialEq, Eq)]
struct DagEdge<NodeData, EdgeData> {
    to: DagNodeHandle<NodeData>,
    user_data: EdgeData,
}


pub struct OnDag<NodeData, EdgeData> {
    edges: DagEdgeMap<NodeData, EdgeData>,
    roots: Vec<DagNodeHandle<NodeData>>,
}

impl <NodeData : Eq, EdgeData : Eq + Hash> Dag<NodeData, EdgeData> for OnDag<NodeData, EdgeData> {
    type NodeHandle = DagNodeHandle<NodeData>;
    fn add_node(&mut self, node: NodeData) -> Self::NodeHandle {
        let handle = Self::NodeHandle::new(node);
        self.roots.push(handle.clone());
        handle
    }
    fn add_edge(&mut self, from: Self::NodeHandle, to: Self::NodeHandle, data: EdgeData) -> Result<(),()> {
        // TODO: Check for cycles
        // TODO: if `to` node in roots, remove it.
        let edge = DagEdge{ to: to, user_data: data };
        self.edges.entry(from)
            .or_insert_with(HashSet::new)
            .insert(edge);
        self.assert_acyclic()
    }
    fn del_edge(&mut self, from: Self::NodeHandle, to: Self::NodeHandle, data: EdgeData) -> Result<(), ()> {
        match self.edges.entry(from) {
            Entry::Vacant(_) => Err(()), // edge was never in the graph
            Entry::Occupied(mut entry) => {
                match entry.get_mut().remove(&DagEdge{ to: to, user_data: data}) {
                    true => Ok(()),
                    false => Err(()), // edge not in graph.
                }
                // TODO: if there are no more outgoing edges on this node, we can delete the vec
                // thereby allowing the node to be freed.
            }
        }
    }
}

impl <NodeData, EdgeData> OnDag<NodeData, EdgeData> {
    pub fn new() -> Self {
        OnDag {
            edges: DagEdgeMap::new(),
            roots: Vec::new(),
        }
    }
    fn assert_acyclic(&self) -> Result<(), ()> {
        // Kahn's algorithm:
        // init S to the set of all nodes with no parents.
        // while S is not empty:
        //   1. delete all edges leaving nodes in S.
        //   2. For all nodes that just had an incoming edge deleted, if they have no remaining inbound
        //        edges, add them to S.
        // If there are no remaining edges, then the graph is acyclic.
        /*let S = self.roots().clone();
        let edges = self.edges.clone();
        let glob_edge_count : usize = 0;
        while !S.empty() {
            for node in S.iter() {
                // if the node has edges, iter them:
                if let Occupied(entry) = self.edges.entry(node) {
                    for outgoing in entry.iter() {
                        // TODO: finish
                    }
                }
            }
        }*/
        Ok(())
    }
    /*fn num_edges(&self) -> usize {
        // sum the number of edges leaving each node:
        self.edges.iter().fold(0, |sum, (_key, val)| {
            sum + val.len()
        })
    }*/
    /*
    /// Returns the graph's edges, but with all outgoing edges turned into incoming edges
    /// and vice-versa.
    fn reversed_edges(&self) -> DagEdgeMap<NodeData, EdgeData> {
        let mut rev = DagEdgeMap::new();
        for (src_handle, out_edges) in self.edges.iter() {
            for dag_edge in out_edges.iter() {
                rev.entry(dag_edge.to.clone()).or_insert_with(Vec::new) /* create the vector for this node's incoming edges. */
                    .push(DagEdge{ to:src_handle.clone(), user_data: dag_edge.user_data });
            }
        }
        rev
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
