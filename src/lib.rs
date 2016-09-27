//#[cfg(test)]
//mod tests;
//
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::hash::{Hash, Hasher};
use std::rc::Rc;


pub trait Dag<NodeData : Eq, EdgeData : Eq> {
    type NodeHandle;
    fn add_node(&mut self, node: NodeData) -> Self::NodeHandle;
    fn add_edge(&mut self, from: Self::NodeHandle, to: Self::NodeHandle, data: EdgeData) -> Result<(),()>;
    fn del_edge(&mut self, from: Self::NodeHandle, to: Self::NodeHandle, data: EdgeData) -> Result<(),()>;
}

pub struct DagNodeHandle<T> {
    value : Rc<T>,
}

struct DagEdge<NodeData, EdgeData> {
    to: DagNodeHandle<NodeData>,
    user_data: EdgeData,
}


pub struct OnDag<NodeData, EdgeData> {
    edges: HashMap<DagNodeHandle<NodeData>, Vec<DagEdge<NodeData, EdgeData>>>,
}

impl <NodeData : Eq, EdgeData : Eq> Dag<NodeData, EdgeData> for OnDag<NodeData, EdgeData> {
    type NodeHandle = DagNodeHandle<NodeData>;
    fn add_node(&mut self, node: NodeData) -> Self::NodeHandle {
        let handle = Self::NodeHandle::new(node);
        handle
    }
    fn add_edge(&mut self, from: Self::NodeHandle, to: Self::NodeHandle, data: EdgeData) -> Result<(),()> {
        // TODO: Check for cycles
        let edge = DagEdge{ to: to, user_data: data };
        self.edges.entry(from)
            .or_insert_with(|| {vec![]})
            .push(edge);
        Ok(())
    }
    fn del_edge(&mut self, from: Self::NodeHandle, to: Self::NodeHandle, data: EdgeData) -> Result<(), ()> {
        match self.edges.entry(from) {
            Entry::Vacant(_) => Err(()), // edge was never in the graph
            Entry::Occupied(mut entry) => {
                // find the location of this edge inside the vector
                match entry.get().iter().enumerate().find(|&e| { e.1.to == to && e.1.user_data == data }) {
                    None => Err(()), //
                    Some((index,_)) => {
                        entry.get_mut().swap_remove(index);
                        // TODO: if there are no more outgoing edges on this node, we can delete the vec
                        // thereby allowing the node to be freed.
                        Ok(())
                    }
                }
            }
        }
    }
}

impl <NodeData, EdgeData> OnDag<NodeData, EdgeData> {
    pub fn new() -> Self {
        OnDag {
            edges: HashMap::new(),
        }
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

