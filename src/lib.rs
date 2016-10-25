//#![cfg(unstable)]
#![feature(conservative_impl_trait)]
//#[cfg(test)]
//mod tests;
//
use std::cell::{RefCell, Ref};
use std::collections::HashSet;
use std::collections::hash_set;
use std::hash::{Hash, Hasher};
use std::rc::Rc;


pub trait Dag<NodeData : Eq + Hash, EdgeData : Eq + Hash> {
    type NodeHandle;
    fn add_node(&mut self, node: NodeData) -> Self::NodeHandle;
    fn add_edge(&mut self, from: Self::NodeHandle, to: Self::NodeHandle, data: EdgeData) -> Result<(),()>;
    fn del_edge(&mut self, from: Self::NodeHandle, to: Self::NodeHandle, data: EdgeData) -> Result<(),()>;
}

pub struct DagNodeHandle<NodeData, EdgeData> {
    node: Rc<RefCell<DagNode<NodeData, EdgeData>>>,
}

#[derive(PartialEq, Eq)]
struct DagEdge<NodeData, EdgeData> {
    to: DagNodeHandle<NodeData, EdgeData>,
    weight: EdgeData,
}

struct DagNode<NodeData, EdgeData> {
    value: NodeData,
    children: HashSet<DagEdge<NodeData, EdgeData>>,
}

// TODO: use a small-size optimized Set, e.g. smallset
// https://github.com/cfallin/rust-smallset/blob/master/src/lib.rs

pub struct OnDag<NodeData, EdgeData> {
    // even though the root can't have any parents, we need to keep this as a
    // DagNodeHandle type for yielding during iteration (etc).
    root: DagNodeHandle<NodeData, EdgeData>,
}

impl <NodeData : Eq + Hash, EdgeData : Eq + Hash> Dag<NodeData, EdgeData> for OnDag<NodeData, EdgeData> {
    type NodeHandle = DagNodeHandle<NodeData, EdgeData>;
    fn add_node(&mut self, node_data: NodeData) -> Self::NodeHandle {
        let handle = Self::NodeHandle::new(DagNode::new(node_data));
        handle
    }
    fn add_edge(&mut self, from: Self::NodeHandle, to: Self::NodeHandle, data: EdgeData) -> Result<(),()> {
        let to_node = to.node.borrow();
        if self.is_reachable(&from, &to) {
            // there is a path from `to` to `from`, so adding an edge `from` -> `to` will introduce
            // a cycle.
            Err(())
        } else {
            // add the parent -> child link:
            from.node.borrow_mut().children.insert(DagEdge::new(to.clone(), data));
            Ok(())
        }
    }
    fn del_edge(&mut self, from: Self::NodeHandle, to: Self::NodeHandle, data: EdgeData) -> Result<(), ()> {
        // delete the parent -> child relationship:
        from.node.borrow_mut().children.remove(&DagEdge::new(to.clone(), data));
        Ok(())
    }
}

impl <N: Eq, E: Eq + Hash> OnDag<N, E> {
    /*fn iter_depth_first(&self) -> impl Iterator<Item=DagNodeHandle<NodeData, EdgeData>> {
        self.root.value.iter_depth_first()
    }*/
    /*fn iter_edges<'a>(&'a self, start_edge: &'a DagEdge<N, E>) -> impl Iterator<Item=&'a DagEdge<N, E>> + 'a {
            let ref node = start_edge.to.node;
            node.borrow().children.iter()
    }*/
    /// Return true if and only if `search` is reachable from (or is equal to) `base`
    fn is_reachable(&self, search: &DagNodeHandle<N, E>, base: &DagNodeHandle<N, E>) -> bool {
        (base == search) || base.node.borrow().children.iter().any(|ch| {
            self.is_reachable(search, &ch.to)
        })
    }
}

impl <NodeData : Eq + Hash, EdgeData : Eq + Hash> OnDag<NodeData, EdgeData> {
    pub fn new(node_data: NodeData) -> Self {
        OnDag {
            root: DagNodeHandle::new(DagNode::new(node_data)),
        }
    }
}

impl<N : Eq + Hash, E : Eq + Hash> DagNode<N, E> {
    fn new(value: N) -> Self {
        DagNode {
            value: value,
            children: HashSet::new(),
        }
    }
    /*fn iter_depth_first<'a>(&'a self) -> impl Iterator<Item=&'a DagEdge<N, E>> + 'a {
        // for each child, yield it and then iter its children
        self.children.iter().flat_map(|ref edge| {
            edge.to.node.borrow().iter_depth_first()
        })
    }*/
    /*fn iter_depth_first<'a>(&'a self) -> impl iterator<item=&'a dagnodehandle<n, e>> + 'a {
        // for each child, yield it and then iter its children
        self.children.iter().flat_map(|ref child| {
            let child_to_node = child.to.node.borrow();
            some(&child.to).into_iter()
            .chain(child_to_node.iter_depth_first())
        })
    }*/
}

impl<N, E> Clone for DagNodeHandle<N, E> {
    fn clone(&self) -> Self {
        DagNodeHandle{ node: self.node.clone() }
    }
}

impl<N, E> Hash for DagNodeHandle<N, E> {
    fn hash<H>(&self, state: &mut H)  where H: Hasher {
        (&*self.node.borrow() as *const DagNode<N, E>).hash(state)
    }
}
impl<N, E> PartialEq for DagNodeHandle<N, E> {
    fn eq(&self, other: &Self) -> bool {
        &*self.node.borrow() as *const DagNode<N, E> == &*other.node.borrow() as *const DagNode<N, E>
    }
}
impl<N, E> Eq for DagNodeHandle<N, E> {}

impl<N, E> DagNodeHandle<N, E> {
    fn new(node: DagNode<N, E>) -> Self {
        DagNodeHandle{ node: Rc::new(RefCell::new(node))}
    }
}

impl<N, E> DagEdge<N, E> {
    fn new(to: DagNodeHandle<N, E>, weight: E) -> Self {
        DagEdge{ to: to, weight: weight }
    }
}

impl<N, E : Hash> Hash for DagEdge<N, E> {
    fn hash<H>(&self, state: &mut H)  where H: Hasher {
        self.to.hash(state);
        self.weight.hash(state)
    }
}

