/// IODag is a DAG where every modification is enforced.
/// However, edges are allowed to have one (or both) end at null.


use std::collections::{HashMap, HashSet};
use std::collections::hash_map;
use std::hash::Hash;

/// N=Node Data
/// W=Weight
pub struct IODag<N, W>
    where W: Hash + Eq + PartialEq {
    /// To create unique NodeHandles, we just assign them unique u64's from this counter.
    node_counter: u64,
    edges: HashMap<Option<NodeHandle>, EdgeSet<W>>,
    node_data: HashMap<NodeHandle, N>,
}

/// Include both the outbound and inbound edges associated with a Node.
struct EdgeSet<W>
    where W: Hash + Eq + PartialEq {
    outbound: HashSet<Edge<W>>,
    inbound: HashSet<Edge<W>>,
}

#[derive(Clone, Eq, Hash, PartialEq)]
pub struct Edge<W> {
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
    where W: Clone + Hash + Eq + PartialEq {
    pub fn new() -> Self {
        let mut edges = HashMap::new();
        edges.insert(None, EdgeSet::new());
        IODag{
            node_counter: 0,
            edges: edges,
            node_data : HashMap::new(),
        }
    }
    pub fn node_data(&self, node: NodeHandle) -> &N {
        &self.node_data[&node]
    }
    pub fn iter_outbound_edges<'a>(&'a self, node: Option<NodeHandle>) -> impl Iterator<Item=&Edge<W>> + 'a {
        self.edges[&node].outbound.iter()
    }
    pub fn iter_inbound_edges<'a>(&'a self, node: Option<NodeHandle>) -> impl Iterator<Item=&Edge<W>> + 'a {
        self.edges[&node].inbound.iter()
    }
    pub fn iter_nodes<'a>(&'a self) -> impl Iterator<Item=&NodeHandle> + 'a {
        self.node_data.keys()
    }
    pub fn iter_edges<'a>(&'a self) -> impl Iterator<Item=&Edge<W>> + 'a {
        // Note: we DON'T duplicate any edges here;
        // This captures all outbound edges, which handles the edge cases correctly (edges leaving
        // NULL AND edges leaving * and entering NULL).
        self.edges.iter().flat_map(|(_node, edges)| {
            edges.outbound.iter()
        })
    }
    pub fn add_node(&mut self, node_data: N) -> NodeHandle {
        let handle = NodeHandle {
            index: self.node_counter,
        };
        self.node_counter = self.node_counter+1;
        // Create storage for the node's outgoing edges
        // Panic if the NodeHandle was somehow already in use.
        assert!(self.edges.insert(Some(handle), EdgeSet::new()).is_none());
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
            self.edges.get_mut(&edge.from).unwrap().outbound.insert(edge.clone());
            self.edges.get_mut(&edge.to).unwrap().inbound.insert(edge);
            Ok(())
        } else {
            Err(())
        }
    }
    /// Removes the node (if it exists)
    /// Errors if the node has incoming or outgoing edges.
    pub fn del_node(&mut self, node: NodeHandle) -> Result<(), ()> {
        let ok_to_delete = match self.edges.entry(Some(node)) {
            // Already deleted
            hash_map::Entry::Vacant(_) => Ok(()),
            hash_map::Entry::Occupied(entry) => {
                if entry.get().is_empty() {
                    entry.remove();
                    Ok(())
                } else {
                    // Node has edges
                    Err(())
                }
            }
        };
        if let Ok(_) = ok_to_delete {
            // delete the data associated with this node
            self.node_data.remove(&node);
        }
        ok_to_delete
    }
    /// Removes the edge (if it exists).
    pub fn del_edge(&mut self, edge: Edge<W>) {
        if let Some(edge_set) = self.edges.get_mut(&edge.from) {
            edge_set.outbound.remove(&edge);
        }
        if let Some(edge_set) = self.edges.get_mut(&edge.to) {
            edge_set.inbound.remove(&edge);
        }
    }

    /// Return true if and only if `search` is reachable from (or is equal to) `base`
    fn is_reachable(&self, search: NodeHandle, base: NodeHandle) -> bool {
        (base == search) || self.edges[&Some(base)].outbound.iter().any(|edge| {
            match edge.to {
                // Edge to Null
                None => false,
                Some(node_handle) => self.is_reachable(search, node_handle),
            }
        })
    }
}

impl<W> Edge<W> {
    pub fn new(from: Option<NodeHandle>, to: Option<NodeHandle>, weight: W) -> Self {
        Edge {
            from: from,
            to: to,
            weight: weight,
        }
    }
    pub fn from(&self) -> &Option<NodeHandle> {
        &self.from
    }
    pub fn to(&self) -> &Option<NodeHandle> {
        &self.to
    }
    pub fn weight(&self) -> &W {
        &self.weight
    }
}

impl<W> EdgeSet<W>
    where W: Hash + Eq + PartialEq {
    fn new() -> Self {
        EdgeSet {
            outbound: HashSet::new(),
            inbound: HashSet::new(),
        }
    }
    fn is_empty(&self) -> bool {
        self.outbound.is_empty()
    }
}
