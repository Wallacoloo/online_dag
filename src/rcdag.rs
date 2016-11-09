use super::ondag::OnDag;

use std::cell::RefCell;
use std::collections::HashSet;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use std::rc::Rc;


pub struct DagNodeHandle<N, E> {
    node: Rc<RefCell<DagNode<N, E>>>,
    /// keep a pointer to the tree owner to enforce mutability rules across multiple trees.
    owner: *const RcDag<N, E>,
}

#[derive(PartialEq, Eq, Clone)]
pub struct DagEdge<N, E> {
    to: DagNodeHandle<N, E>,
    weight: E,
}

struct DagNode<N, E> {
    value: N,
    children: HashSet<DagEdge<N, E>>,
}

// TODO: use a small-size optimized Set, e.g. smallset
// https://github.com/cfallin/rust-smallset/blob/master/src/lib.rs

/// Note: these graphs don't necessarily have explicit roots. It's the user's job to keep handles
/// to root nodes in order to iterate them, etc.
pub struct RcDag<N, E> {
    /// The DAG doesn't actually store any nodes/edges - it just creates them and hands out
    /// handles. PhantomData allows the type parameters to not be used in the struct (just impl)
    /// w/o error.
    node_type: PhantomData<N>,
    edge_type: PhantomData<E>,
}

impl <N : Eq + Hash, E : Eq + Hash + Clone> OnDag<N, E> for RcDag<N, E> {
    type NodeHandle = DagNodeHandle<N, E>;
    fn add_node(&mut self, node_data: N) -> Self::NodeHandle {
        let handle = Self::NodeHandle::new(self, DagNode::new(node_data));
        handle
    }
    fn add_edge(&mut self, from: &Self::NodeHandle, to: &Self::NodeHandle, data: E) -> Result<(),()> {
        // the edge must connect two nodes owned by *this* graph.
        assert_eq!(from.owner, self as *const Self);
        assert_eq!(to.owner, self as *const Self);
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
    fn rm_edge(&mut self, from: &Self::NodeHandle, to: &Self::NodeHandle, data: E) -> Result<(), ()> {
        // the edge must belong to *this* graph.
        assert_eq!(from.owner, self as *const Self);
        assert_eq!(to.owner, self as *const Self);
        // delete the parent -> child relationship:
        // TODO: should be possible to remove w/o cloning the references.
        from.node.borrow_mut().children.remove(&DagEdge::new(to.clone(), data));
        Ok(())
    }
}

impl <N: Eq, E: Eq + Hash> RcDag<N, E> {
    /// Return true if and only if `search` is reachable from (or is equal to) `base`
    fn is_reachable(&self, search: &DagNodeHandle<N, E>, base: &DagNodeHandle<N, E>) -> bool {
        (base == search) || base.node.borrow().children.iter().any(|ch| {
            self.is_reachable(search, &ch.to)
        })
    }
    /// Compute the topological ordering of `self`.
    pub fn iter_topo(&self, from: &DagNodeHandle<N, E>) -> impl Iterator<Item=DagNodeHandle<N, E>> {
        // can only iterate over nodes owned by *this* graph.
        assert_eq!(from.owner, self as *const Self);
        // just a depth-first sort, but then reverse the results.
        let mut ordered = vec![];
        self.depth_first_sort(from, &mut ordered, &mut HashSet::new());
        // The depth-first ordering goes highest -> least depth, so reverse that.
        ordered.into_iter().rev()
    }
    /// Compute the *reverse* topological ordering of `self`, i.e. children -> root
    pub fn iter_topo_rev(&self, from: &DagNodeHandle<N, E>) -> impl Iterator<Item=DagNodeHandle<N, E>> {
        // can only iterate over nodes owned by *this* graph.
        assert_eq!(from.owner, self as *const Self);
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
impl <N: Eq + Clone, E: Eq + Hash + Clone> RcDag<N, E> {
    /// iterate all of the outgoing edges of this node.
    pub fn children(&self, node: &DagNodeHandle<N, E>) -> impl Iterator<Item=DagEdge<N, E>> {
        // we must own the node of interest.
        assert_eq!(node.owner, self as *const Self);
        // TODO: make an iterator object that borrows self & avoids cloning children
        node.node.borrow().children.clone().into_iter()
    }
}

impl <N : Eq + Hash, E : Eq + Hash> RcDag<N, E> {
    pub fn new() -> Self {
        RcDag {
            node_type: PhantomData,
            edge_type: PhantomData,
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
        DagNodeHandle {
            node: self.node.clone(),
            owner: self.owner,
        }
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
    fn new(owner: &RcDag<N, E>, node: DagNode<N, E>) -> Self {
        DagNodeHandle {
            node: Rc::new(RefCell::new(node)),
            owner: owner,
        }
    }
}

impl<N : Clone, E> DagNodeHandle<N, E> {
    /// Access the node's data via cloning it (potentially costly). Doesn't require a ref to the tree.
    pub fn node(&self) -> N {
        self.node.borrow().value.clone()
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

