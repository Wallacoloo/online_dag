//#![cfg(unstable)]
// Requires feature-gate for returning impl Iterator
#![feature(conservative_impl_trait)]

#[cfg(test)]
mod tests;

use std::cell::RefCell;
use std::collections::HashSet;
use std::hash::{Hash, Hasher};
use std::rc::Rc;


pub trait Dag<NodeData : Eq + Hash, EdgeData : Eq + Hash + Clone> {
    type NodeHandle;
    fn add_node(&mut self, node: NodeData) -> Self::NodeHandle;
    fn add_edge(&mut self, from: Self::NodeHandle, to: Self::NodeHandle, data: EdgeData) -> Result<(),()>;
    fn rm_edge(&mut self, from: Self::NodeHandle, to: Self::NodeHandle, data: EdgeData) -> Result<(),()>;
    fn root(&self) -> Self::NodeHandle;
}

pub struct DagNodeHandle<NodeData, EdgeData> {
    node: Rc<RefCell<DagNode<NodeData, EdgeData>>>,
}

#[derive(PartialEq, Eq, Clone)]
pub struct DagEdge<NodeData, EdgeData> {
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

impl <NodeData : Eq + Hash, EdgeData : Eq + Hash + Clone> Dag<NodeData, EdgeData> for OnDag<NodeData, EdgeData> {
    type NodeHandle = DagNodeHandle<NodeData, EdgeData>;
    fn add_node(&mut self, node_data: NodeData) -> Self::NodeHandle {
        let handle = Self::NodeHandle::new(DagNode::new(node_data));
        handle
    }
    fn add_edge(&mut self, from: Self::NodeHandle, to: Self::NodeHandle, data: EdgeData) -> Result<(),()> {
        if self.is_reachable(&from, &to) {
            // there is a path from `to` to `from`, so adding an edge `from` -> `to` will introduce
            // a cycle.
            Err(())
        } else {
            // add the parent -> child link:
            from.node.borrow_mut().children.insert(DagEdge::new(to.clone(), data.clone()));
            Ok(())
        }
    }
    fn rm_edge(&mut self, from: Self::NodeHandle, to: Self::NodeHandle, data: EdgeData) -> Result<(), ()> {
        // delete the parent -> child relationship:
        from.node.borrow_mut().children.remove(&DagEdge::new(to.clone(), data.clone()));
        Ok(())
    }
    fn root(&self) -> Self::NodeHandle {
        self.root.clone()
    }
}

impl <N: Eq, E: Eq + Hash> OnDag<N, E> {
    /// Return true if and only if `search` is reachable from (or is equal to) `base`
    fn is_reachable(&self, search: &DagNodeHandle<N, E>, base: &DagNodeHandle<N, E>) -> bool {
        (base == search) || base.node.borrow().children.iter().any(|ch| {
            self.is_reachable(search, &ch.to)
        })
    }
    /// Compute the topological ordering of `self`.
    pub fn topo_sort(&self) -> impl Iterator<Item=DagNodeHandle<N, E>> {
        // internally, do a depth-first search & reverse the results.
        let mut ordered = vec![];
        self.depth_first_sort(&self.root, &mut ordered, &mut HashSet::new());
        // The depth-first ordering goes highest -> least depth, so reverse that.
        ordered.into_iter().rev()
    }
    fn depth_first_sort(&self, node: &DagNodeHandle<N, E>, ordered: &mut Vec<DagNodeHandle<N, E>>, marked: &mut HashSet<*const DagNode<N, E>>) {
        if !marked.contains(&(&*node.node.borrow() as *const DagNode<N, E>)) {
            for edge in node.node.borrow().children.iter() {
                self.depth_first_sort(&edge.to, ordered, marked);
            }
            marked.insert(&*node.node.borrow());
            ordered.push(node.clone());
        }
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

impl<N : Clone, E> DagNodeHandle<N, E> {
    /// Access the node's data via cloning it (potentially costly). Doesn't require a ref to the tree.
    pub fn node(&self) -> N {
        self.node.borrow().value.clone()
    }
}
impl<N : Clone, E : Clone> DagNodeHandle<N, E> {
    /// Access the node's children via cloning the data structure linking to them (potentially
    /// costly). Doesn't require a ref to the tree.
    fn children(&self) -> HashSet<DagEdge<N, E>> {
        self.node.borrow().children.clone()
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

