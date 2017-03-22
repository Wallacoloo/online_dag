/// IODag is DAG where every modification is enforced.
/// However, edges are allowed to have one end at null.
/// There are InEdges, in which the `from` component is null,
/// and OutEdges, in which the `to` component is null.
/// (Plus, MidEdge, which has no null components).
/// Each Edge type allows a different type of weight.
///

use std::collections::{HashMap, HashSet};
use std::hash::Hash;

/// W=Weight
pub struct IODag<N, FromNodeW, FromNullW, ToNodeW, ToNullW>
    where FromNodeW: Hash + Eq + PartialEq, FromNullW: Hash + Eq + PartialEq, ToNodeW: Hash + Eq + PartialEq, ToNullW: Hash + Eq + PartialEq {
    /// To create unique NodeHandles, we just assign them unique u64's from this counter.
    node_counter: u64,
    /// Node states, including their outgoing edges.
    node_data: HashMap<NodeHandle, NodeData<N, FromNodeW, FromNullW, ToNodeW, ToNullW>>,
    /// Edges that start at null.
    edges_from_null: HashSet<Edge<FromNodeW, FromNullW, ToNodeW, ToNullW>>,
}

pub struct NodeData<N, FromNodeW, FromNullW, ToNodeW, ToNullW>
    where FromNodeW: Hash + Eq + PartialEq, FromNullW: Hash + Eq + PartialEq, ToNodeW: Hash + Eq + PartialEq, ToNullW: Hash + Eq + PartialEq {
    /// userdata associated with this node
    data: N,
    /// Edges leaving this node
    outbound: HashSet<Edge<FromNodeW, FromNullW, ToNodeW, ToNullW>>,
}

// Edges consist of two parts: the `from` and the `to`.
// Define this structure in a way that creates 4 types of edges.
#[derive(Eq, Hash, PartialEq)]
pub struct FromNull<W> {
    weight: W,
}
#[derive(Eq, Hash, PartialEq)]
pub struct FromNode<W> {
    node: NodeHandle,
    weight: W,
}

#[derive(Eq, Hash, PartialEq)]
pub struct ToNull<W> {
    weight: W,
}
#[derive(Eq, Hash, PartialEq)]
pub struct ToNode<W> {
    node: NodeHandle,
    weight: W,
}


#[derive(Eq, Hash, PartialEq)]
pub enum EdgeFrom<FromNodeW, FromNullW>
    where FromNodeW: Hash + Eq + PartialEq, FromNullW: Hash + Eq + PartialEq {
    Null(FromNull<FromNullW>),
    Node(FromNode<FromNodeW>),
}
#[derive(Eq, Hash, PartialEq)]
pub enum EdgeTo<ToNodeW, ToNullW>
    where ToNodeW: Hash + Eq + PartialEq, ToNullW: Hash + Eq + PartialEq {
    Null(ToNull<ToNullW>),
    Node(ToNode<ToNodeW>),
}

#[derive(Eq, Hash, PartialEq)]
pub struct Edge<FromNodeW, FromNullW, ToNodeW, ToNullW>
    where FromNodeW: Hash + Eq + PartialEq, FromNullW: Hash + Eq + PartialEq, ToNodeW: Hash + Eq + PartialEq, ToNullW: Hash + Eq + PartialEq {
    from: EdgeFrom<FromNodeW, FromNullW>,
    to: EdgeTo<ToNodeW, ToNullW>,
}

#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub struct NodeHandle {
    index: u64,
}

impl<N, FromNodeW, FromNullW, ToNodeW, ToNullW> IODag<N, FromNodeW, FromNullW, ToNodeW, ToNullW>
    where FromNodeW: Hash + Eq + PartialEq, FromNullW: Hash + Eq + PartialEq, ToNodeW: Hash + Eq + PartialEq, ToNullW: Hash + Eq + PartialEq {
    pub fn new() -> Self {
        IODag {
            node_counter: 0,
            node_data : HashMap::new(),
            edges_from_null: HashSet::new(),
        }
    }
    pub fn new_node(&mut self, node_data: N) -> NodeHandle {
        let handle = NodeHandle {
            index: self.node_counter,
        };
        self.node_counter = self.node_counter+1;
        // Create storage for the node's outgoing edges
        // Panic if the NodeHandle was somehow already in use.
        assert!(self.node_data.insert(handle, NodeData::new(node_data)).is_none());
        handle
    }
    pub fn add_edge(&mut self, edge: Edge<FromNodeW, FromNullW, ToNodeW, ToNullW>) -> Result<(), ()> {
        let from_handle = edge.from_handle();
        let to_handle = edge.to_handle();
        let safe_to_add = match from_handle {
            // Edges from Null cannot cycle
            None => true,
            Some(from) => match to_handle {
                // Edges to Null cannot cycle
                None => true,
                // if we can reach 'from' via 'to', then connecting from -> to creates cycle.
                Some(to) => !self.is_reachable(from, to),
            }
        };

        if safe_to_add {
            match from_handle {
                None => self.edges_from_null.insert(edge),
                Some(from) => self.node_data.get_mut(&from).unwrap().outbound.insert(edge),
            };
            Ok(())
        } else {
            Err(())
        }
    }
    /// Removes the edge (if it exists).
    /// Returns true if the edge was previously present
    pub fn del_edge(&mut self, edge: Edge<FromNodeW, FromNullW, ToNodeW, ToNullW>) -> bool {
        let from_handle = edge.from_handle();
        match from_handle {
            None => self.edges_from_null.remove(&edge),
            Some(from) => match self.node_data.get_mut(&from) {
                // The 'from' portion of the node isn't in this Dag.
                None => false,
                Some(node_data) => node_data.outbound.remove(&edge),
            },
        }
    }

    /// Return true if and only if `search` is reachable from (or is equal to) `base`
    fn is_reachable(&self, search: NodeHandle, base: NodeHandle) -> bool {
        (base == search) || self.node_data[&base].outbound.iter().any(|edge| {
            match edge.to_handle() {
                // Edge to Null
                None => false,
                Some(node_handle) => self.is_reachable(search, node_handle),
            }
        })
    }
}



impl<N, FromNodeW, FromNullW, ToNodeW, ToNullW> NodeData<N, FromNodeW, FromNullW, ToNodeW, ToNullW>
    where FromNodeW: Hash + Eq + PartialEq, FromNullW: Hash + Eq + PartialEq, ToNodeW: Hash + Eq + PartialEq, ToNullW: Hash + Eq + PartialEq {
    fn new(node_data: N) -> Self {
        Self {
            data: node_data,
            outbound: HashSet::new(),
        }
    }
}


impl<FromNodeW, FromNullW, ToNodeW, ToNullW> Edge<FromNodeW, FromNullW, ToNodeW, ToNullW>
    where FromNodeW: Hash + Eq + PartialEq, FromNullW: Hash + Eq + PartialEq, ToNodeW: Hash + Eq + PartialEq, ToNullW: Hash + Eq + PartialEq {
    /// Return the NodeHandle that this edge points from, or None if it points from Null.
    fn from_handle(&self) -> Option<NodeHandle> {
        match self.from {
            EdgeFrom::Null(_) => None,
            EdgeFrom::Node(ref from_node) => Some(from_node.node),
        }
    }
    /// Return the NodeHandle that this edge points to, or None if it points to Null.
    fn to_handle(&self) -> Option<NodeHandle> {
        match self.to {
            EdgeTo::Null(_) => None,
            EdgeTo::Node(ref to_node) => Some(to_node.node),
        }
    }
}
