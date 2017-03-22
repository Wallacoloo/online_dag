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
// Define this structure in a way that creates 4 type of edges.
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

#[derive(Eq, Hash, PartialEq)]
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
        let old_counter = self.node_counter;
        self.node_counter = self.node_counter+1;
        NodeHandle {
            index: old_counter,
        }
    }
    pub fn add_edge(&mut self, edge: Edge<FromNodeW, FromNullW, ToNodeW, ToNullW>) -> Result<(), ()> {
        // TODO: implement
        Ok(())
    }
    /// Removes the edge (if it exists).
    /// If this leaves a floating node, any internal resources allocated to that node are gc'd.
    pub fn del_edge(&mut self, edge: Edge<FromNodeW, FromNullW, ToNodeW, ToNullW>) {
        // TODO: implement
    }
}
