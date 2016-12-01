/// Base functions for implementing *various* DAG types on top of a Rc Node format.

use std::cell::RefCell;
use std::collections::HashSet;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use std::rc::{Rc, Weak};


pub struct NodeHandle<N, E> {
    node: Rc<RefCell<DagNode<N, E>>>,
    /// keep a pointer to the tree owner to enforce mutability rules across multiple trees.
    owner: *const RcDagBase<N, E>,
}

/// allows to uniquely identify a node, but without keeping it alive.
/// The primary use-case for this is in a map structure where the client maps
/// Node -> {data}, but wants to not keep the data alive if the node dies.
pub struct WeakNodeHandle<N, E> {
    node: Weak<RefCell<DagNode<N, E>>>,
    // need to preserve the raw ptr address for hashing, since we can't extract
    // *any* information from a dead Weak pointer.
    // TODO: Even a dead weak pointer has Shared memory allocation for the counts -
    //   we should therefore be able to hash & compare Weak pointers without storing
    //   the node_ptr separately.
    // NOTE: We need to store more than just the raw pointer because the memory
    // location of a pointer can be reused after the Rc dies.
    node_ptr: *const RefCell<DagNode<N, E>>,
}

pub struct DagEdge<N, E> {
    to: NodeHandle<N, E>,
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
pub struct RcDagBase<N, E> {
    /// The DAG doesn't actually store any nodes/edges - it just creates them and hands out
    /// handles. PhantomData allows the type parameters to not be used in the struct (just impl)
    /// w/o error.
    node_type: PhantomData<N>,
    edge_type: PhantomData<E>,
}

impl <N, E : Eq> RcDagBase<N, E> {
    pub fn add_node(&mut self, node_data: N) -> NodeHandle<N, E> {
        let handle = NodeHandle::new(self, DagNode::new(node_data));
        handle
    }
    pub fn add_edge_unchecked(&mut self, from: &NodeHandle<N, E>, to: &NodeHandle<N, E>, data: E) {
        // the edge must connect two nodes owned by *this* graph.
        assert_eq!(from.owner, self as *const Self);
        assert_eq!(to.owner, self as *const Self);
        // add the parent -> child link:
        from.node.borrow_mut().children.insert(DagEdge::new(to.clone(), data));
    }
    pub fn rm_edge(&mut self, from: &NodeHandle<N, E>, to: &NodeHandle<N, E>, data: E) {
        // the edge must belong to *this* graph.
        assert_eq!(from.owner, self as *const Self);
        assert_eq!(to.owner, self as *const Self);
        // delete the parent -> child relationship:
        // TODO: should be possible to remove w/o cloning the references.
        from.node.borrow_mut().children.remove(&DagEdge::new(to.clone(), data));
    }
}

impl <N, E: Eq> RcDagBase<N, E> {
    /// Return true if and only if `search` is reachable from (or is equal to) `base`
    pub fn is_reachable(&self, search: &NodeHandle<N, E>, base: &NodeHandle<N, E>) -> bool {
        (base == search) || base.node.borrow().children.iter().any(|ch| {
            self.is_reachable(search, &ch.to)
        })
    }
    /// Compute the topological ordering of `self`.
    pub fn iter_topo(&self, from: &NodeHandle<N, E>) -> impl Iterator<Item=NodeHandle<N, E>> {
        // can only iterate over nodes owned by *this* graph.
        assert_eq!(from.owner, self as *const Self);
        // just a depth-first sort, but then reverse the results.
        let mut ordered = vec![];
        self.depth_first_sort(from, &mut ordered, &mut HashSet::new());
        // The depth-first ordering goes highest -> least depth, so reverse that.
        ordered.into_iter().rev()
    }
    /// Compute the *reverse* topological ordering of `self`, i.e. children -> root
    pub fn iter_topo_rev(&self, from: &NodeHandle<N, E>) -> impl Iterator<Item=NodeHandle<N, E>> {
        // can only iterate over nodes owned by *this* graph.
        assert_eq!(from.owner, self as *const Self);
        // just a depth-first sort:
        // TODO: we can achieve this with lower latency by moving it into an iterator.
        let mut ordered = vec![];
        self.depth_first_sort(from, &mut ordered, &mut HashSet::new());
        // The depth-first ordering goes highest -> least depth
        ordered.into_iter()
    }
    fn depth_first_sort(&self, node: &NodeHandle<N, E>, ordered: &mut Vec<NodeHandle<N, E>>, marked: &mut HashSet<*const DagNode<N, E>>) {
        if !marked.contains(&(&*node.node.borrow() as *const DagNode<N, E>)) {
            for edge in node.node.borrow().children.iter() {
                self.depth_first_sort(&edge.to, ordered, marked);
            }
            marked.insert(&*node.node.borrow());
            ordered.push(node.clone());
        }
    }
}
impl <N, E: Eq + Clone> RcDagBase<N, E> {
    /// iterate all of the outgoing edges of this node.
    #[allow(dead_code)]
    pub fn children(&self, node: &NodeHandle<N, E>) -> impl Iterator<Item=DagEdge<N, E>> {
        // we must own the node of interest.
        assert_eq!(node.owner, self as *const Self);
        // TODO: make an iterator object that borrows self & avoids cloning children
        node.node.borrow().children.clone().into_iter()
    }
}

impl <N, E> RcDagBase<N, E> {
    #[allow(dead_code)]
    pub fn new() -> Self {
        RcDagBase {
            node_type: PhantomData,
            edge_type: PhantomData,
        }
    }
}

impl<N, E : Eq> DagNode<N, E> {
    fn new(value: N) -> Self {
        DagNode {
            value: value,
            children: HashSet::new(),
        }
    }
}

impl<N, E> Clone for NodeHandle<N, E> {
    fn clone(&self) -> Self {
        NodeHandle {
            node: self.node.clone(),
            owner: self.owner,
        }
    }
}

impl<N, E> Hash for NodeHandle<N, E> {
    fn hash<H>(&self, state: &mut H)  where H: Hasher {
        (&*self.node as *const RefCell<DagNode<N, E>>).hash(state)
    }
}
impl<N, E> PartialEq for NodeHandle<N, E> {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.node, &other.node)
        //&*self.node as *const RefCell<DagNode<N, E>> == &*other.node as *const RefCell<DagNode<N, E>>
    }
}
impl<N, E> Eq for NodeHandle<N, E> {}

impl<N, E> NodeHandle<N, E> {
    fn new(owner: &RcDagBase<N, E>, node: DagNode<N, E>) -> Self {
        NodeHandle {
            node: Rc::new(RefCell::new(node)),
            owner: owner,
        }
    }
}
impl<N: Default, E: Eq> NodeHandle<N, E> {
    /// Create a null handle that can be used instead by clients in place of Option<>.
    /// Note: trying to use this in a query to a tree that expects a non-null NodeHandle WILL
    /// error.
    pub fn null() -> Self {
        NodeHandle {
            node: Rc::new(RefCell::new(
                          DagNode::new(Default::default())
            )),
            owner: 0 as *const RcDagBase<N, E>,
        }
    }
}

impl<N : Clone, E> NodeHandle<N, E> {
    /// Access the node's data via cloning it (potentially costly). Doesn't require a ref to the tree.
    pub fn node_data(&self) -> N {
        self.node.borrow().value.clone()
    }
}

impl<N, E> NodeHandle<N, E> {
    pub fn weak(&self) -> WeakNodeHandle<N, E> {
        WeakNodeHandle{
            node: Rc::downgrade(&self.node),
            node_ptr: &*self.node,
        }
    }
    pub(super) fn owner(&self) -> *const RcDagBase<N, E> {
        self.owner
    }
}

impl<N, E> Hash for WeakNodeHandle<N, E> {
    fn hash<H>(&self, state: &mut H)  where H: Hasher {
        self.node_ptr.hash(state);
    }
}
impl<N, E> PartialEq for WeakNodeHandle<N, E> {
    fn eq(&self, other: &Self) -> bool {
        match self.node.upgrade() {
            None => {
                if let Some(_) = other.node.upgrade() {
                    false
                } else {
                    true
                }
            },
            Some(my_rc) => {
                if let Some(other_rc) = other.node.upgrade() {
                    Rc::ptr_eq(&my_rc, &other_rc)
                } else {
                    false
                }
            }
        }
    }
}
impl<N, E> Eq for WeakNodeHandle<N, E> {}

impl<N, E> DagEdge<N, E> {
    fn new(to: NodeHandle<N, E>, weight: E) -> Self {
        DagEdge{ to: to, weight: weight }
    }
}

impl<N, E> DagEdge<N, E> {
    #[allow(dead_code)]
    pub fn to(&self) -> &NodeHandle<N, E> {
        &self.to
    }
    #[allow(dead_code)]
    pub fn weight(&self) -> &E {
        &self.weight
    }
}

impl<N, E> Hash for DagEdge<N, E> {
    fn hash<H>(&self, state: &mut H)  where H: Hasher {
        // we don't really need to hash the edge; few use-cases have many
        // overlapping edges with different weights.
        self.to.hash(state);
    }
}

// Yes, this is identical to the default Clone implementation,
// but the default impl requires N to also be cloneable.
impl<N, E : Clone> Clone for DagEdge<N, E> {
    fn clone(&self) -> Self {
        DagEdge {
            to: self.to.clone(),
            weight: self.weight.clone(),
        }
    }
}

// Identical to default Eq, again, but we don't want N : Eq requirement.
impl<N, E : Eq> PartialEq for DagEdge<N, E> {
    fn eq(&self, other: &Self) -> bool {
        self.to == other.to && self.weight == other.weight
    }
}
impl<N, E : Eq> Eq for DagEdge<N, E>{}

