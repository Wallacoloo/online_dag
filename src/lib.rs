//#![cfg(unstable)]
// Requires feature-gate for returning impl Iterator
#![feature(conservative_impl_trait)]

#[cfg(test)]
mod tests;

use std::cell::RefCell;
use std::collections::HashSet;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use std::rc::Rc;


pub trait Dag<N : Eq + Hash, E : Eq + Hash + Clone> {
    type NodeHandle;
    fn add_node(&mut self, node: N) -> Self::NodeHandle;
    fn add_edge(&mut self, from: Self::NodeHandle, to: Self::NodeHandle, data: E) -> Result<(),()>;
    fn rm_edge(&mut self, from: Self::NodeHandle, to: Self::NodeHandle, data: E) -> Result<(),()>;
}

pub struct DagNodeHandle<N, E> {
    node: Rc<RefCell<DagNode<N, E>>>,
}

#[derive(PartialEq, Eq, Clone)]
pub struct DagEdge<N, E> {
    to: DagNodeHandle<N, E>,
    weight: E,
}

struct DagNode<N, E> {
    value: N,
    children: HashSet<DagEdge<N, E>>,
    /// keep a pointer to the tree owner to enforce mutability rules across multiple trees.
    owner: *const OnDag<N, E>,
}

// TODO: use a small-size optimized Set, e.g. smallset
// https://github.com/cfallin/rust-smallset/blob/master/src/lib.rs

/// Note: these graphs don't necessarily have explicit roots. It's the user's job to keep handles
/// to root nodes in order to iterate them, etc.
pub struct OnDag<N, E> {
    /// The DAG doesn't actually store any nodes/edges - it just creates them and hands out
    /// handles. PhantomData allows the type parameters to not be used in the struct (just impl)
    /// w/o error.
    node_type: PhantomData<N>,
    edge_type: PhantomData<E>,
}

impl <N : Eq + Hash, E : Eq + Hash + Clone> Dag<N, E> for OnDag<N, E> {
    type NodeHandle = DagNodeHandle<N, E>;
    fn add_node(&mut self, node_data: N) -> Self::NodeHandle {
        let handle = Self::NodeHandle::new(DagNode::new(self, node_data));
        handle
    }
    fn add_edge(&mut self, from: Self::NodeHandle, to: Self::NodeHandle, data: E) -> Result<(),()> {
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
    fn rm_edge(&mut self, from: Self::NodeHandle, to: Self::NodeHandle, data: E) -> Result<(), ()> {
        // delete the parent -> child relationship:
        from.node.borrow_mut().children.remove(&DagEdge::new(to.clone(), data.clone()));
        Ok(())
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
    pub fn iter_topo(&self, from: &DagNodeHandle<N, E>) -> impl Iterator<Item=DagNodeHandle<N, E>> {
        // just a depth-first sort, but then reverse the results.
        let mut ordered = vec![];
        self.depth_first_sort(from, &mut ordered, &mut HashSet::new());
        // The depth-first ordering goes highest -> least depth, so reverse that.
        ordered.into_iter().rev()
    }
    /// Compute the *reverse* topological ordering of `self`, i.e. children -> root
    pub fn iter_topo_rev(&self, from: &DagNodeHandle<N, E>) -> impl Iterator<Item=DagNodeHandle<N, E>> {
        // just a depth-first sort:
        // TODO: we can achieve this with lower latency by moving it into an iterator.
        let mut ordered = vec![];
        self.depth_first_sort(from, &mut ordered, &mut HashSet::new());
        // The depth-first ordering goes highest -> least depth
        ordered.into_iter()
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

impl <N : Eq + Hash, E : Eq + Hash> OnDag<N, E> {
    pub fn new() -> Self {
        OnDag {
            node_type: PhantomData,
            edge_type: PhantomData,
        }
    }
}

impl<N : Eq + Hash, E : Eq + Hash> DagNode<N, E> {
    fn new(owner: &OnDag<N, E>, value: N) -> Self {
        DagNode {
            value: value,
            children: HashSet::new(),
            owner: owner as *const OnDag<N, E>,
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

