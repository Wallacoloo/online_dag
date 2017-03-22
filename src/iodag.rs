/// IODag is DAG where every modification is enforced.
/// However, edges are allowed to have one end at null.
/// There are InEdges, in which the `from` component is null,
/// and OutEdges, in which the `to` component is null.
/// (Plus, MidEdge, which has no null components).
/// Each Edge type allows a different type of weight.
///

use std::collections::HashMap;
use std::hash::Hash;

/// W=Weight
pub struct IODag<N, FromNodeW, FromNullW, ToNodeW, ToNullW> {
    /// To create unique NodeHandles, we just assign them unique u64's from this counter.
    node_counter: u64,
    /// Node states, including their outgoing edges.
    node_data: HashMap<NodeHandle, NodeData<N, FromNodeW, FromNullW, ToNodeW, ToNullW>>,
    /// Edges that start at null.
    edges_from_null: Vec<Edge<FromNodeW, FromNullW, ToNodeW, ToNullW>>,
}

pub struct NodeData<N, FromNodeW, FromNullW, ToNodeW, ToNullW> {
    /// userdata associated with this node
    data: N,
    /// Edges leaving this node
    outbound: Vec<Edge<FromNodeW, FromNullW, ToNodeW, ToNullW>>,
}

// Edges consist of two parts: the `from` and the `to`.
// Define this structure in a way that creates 4 type of edges.
pub struct FromNull<W> {
    weight: W,
}
pub struct FromNode<W> {
    node: NodeHandle,
    weight: W,
}

pub struct ToNull<W> {
    weight: W,
}
pub struct ToNode<W> {
    node: NodeHandle,
    weight: W,
}


pub enum EdgeFrom<FromNodeW, FromNullW> {
    Null(FromNull<FromNullW>),
    Node(FromNode<FromNodeW>),
}
pub enum EdgeTo<ToNodeW, ToNullW> {
    Null(ToNull<ToNullW>),
    Node(ToNode<ToNodeW>),
}

pub struct Edge<FromNodeW, FromNullW, ToNodeW, ToNullW> {
    from: EdgeFrom<FromNodeW, FromNullW>,
    to: EdgeTo<ToNodeW, ToNullW>,
}

#[derive(Eq, Hash, PartialEq)]
pub struct NodeHandle {
    index: u64,
}

impl<N, FromNodeW, FromNullW, ToNodeW, ToNullW> IODag<N, FromNodeW, FromNullW, ToNodeW, ToNullW> {
    pub fn new() -> Self {
        IODag {
            node_counter: 0,
            node_data : HashMap::new(),
            edges_from_null: Vec::new(),
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
