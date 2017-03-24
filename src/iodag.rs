/// IODag is a DAG where every modification is enforced.
/// However, edges are allowed to have one (or both) end at null.


use std::collections::{HashMap, HashSet};
use std::hash::Hash;

/// N=Node Data
/// W=Weight
pub struct IODag<N, W>
    where W: Hash + Eq + PartialEq {
    /// To create unique NodeHandles, we just assign them unique u64's from this counter.
    node_counter: u64,
    outbound_edges: HashMap<Option<NodeHandle>, HashSet<Edge<W>>>,
    node_data: HashMap<NodeHandle, N>,
}


#[derive(Eq, Hash, PartialEq)]
pub struct Edge<W>
    where W: Hash + Eq + PartialEq {
    from: Option<NodeHandle>,
    to: Option<NodeHandle>,
    weight: W,
}

#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub struct NodeHandle {
    // TODO: add NonZero attribute (or similar) to optimize Option<NodeHandle>
    index: u64,
}

impl<N, W> IODag<N, W>
    where W: Hash + Eq + PartialEq { 
    pub fn new() -> Self {
        let mut outbound_edges = HashMap::new();
        outbound_edges.insert(None, HashSet::new());
        IODag{
            node_counter: 0,
            outbound_edges: outbound_edges,
            node_data : HashMap::new(),
        }
    }
    pub fn add_node(&mut self, node_data: N) -> NodeHandle {
        let handle = NodeHandle {
            index: self.node_counter,
        };
        self.node_counter = self.node_counter+1;
        // Create storage for the node's outgoing edges
        // Panic if the NodeHandle was somehow already in use.
        assert!(self.outbound_edges.insert(Some(handle), HashSet::new()).is_none());
        // Store the node's data
        assert!(self.node_data.insert(handle, node_data).is_none());
        handle
    }
    pub fn add_edge(&mut self, edge: Edge<W>) -> Result<(), ()> {
        let safe_to_add = match edge.from {
            // Edges from Null cannot cycle
            None => true,
            Some(from) => match edge.to {
                // Edges to Null cannot cycle
                None => true,
                // if we can reach 'from' via 'to', then connecting from -> to creates cycle.
                Some(to) => !self.is_reachable(from, to),
            }
        };

        if safe_to_add {
            self.outbound_edges.get_mut(&edge.from).unwrap().insert(edge);
            Ok(())
        } else {
            Err(())
        }
    }
    /// Removes the edge (if it exists).
    /// Returns true if the edge was previously present
    pub fn del_edge(&mut self, edge: Edge<W>) -> bool {
        match self.outbound_edges.get_mut(&edge.from) {
            None => false,
            Some(edge_set) => edge_set.remove(&edge),
        }
    }

    /// Return true if and only if `search` is reachable from (or is equal to) `base`
    fn is_reachable(&self, search: NodeHandle, base: NodeHandle) -> bool {
        (base == search) || self.outbound_edges[&Some(base)].iter().any(|edge| {
            match edge.to {
                // Edge to Null
                None => false,
                Some(node_handle) => self.is_reachable(search, node_handle),
            }
        })
    }
}


