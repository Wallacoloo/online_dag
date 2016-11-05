use ::OnDag;
use ::Dag;

type MyDag = OnDag<u32, u32>;


#[test]
/// After construction and adding a root, iter_topo should iterate only that root
fn test_root() {
    let mut dag = MyDag::new();
    let root = dag.add_node(12);
    assert_eq!(dag.iter_topo(&root).map(|handle| { handle.node() }).collect::<Vec<u32>>(), vec![12]);
}

#[test]
/// After adding some nodes, with no edges, they should *not* be included in a topo_sort
fn test_orphans() {
    let mut dag = MyDag::new();
    let root = dag.add_node(12);
    dag.add_node(2);
    dag.add_node(1);
    assert_eq!(dag.iter_topo(&root).map(|handle| { handle.node() }).collect::<Vec<u32>>(), vec![12]);
}

#[test]
/// After adding some nodes AND edges, they should be included in topo_sort
fn test_add_edges() {
    //     12
    //     v
    //     1
    //     v
    //     2
    let mut dag = MyDag::new();
    let root = dag.add_node(12);
    let n2 = dag.add_node(2);
    let n1 = dag.add_node(1);
    dag.add_edge(&root, &n1, 1001).expect("Failed to add edge");
    dag.add_edge(&n1, &n2, 1002).expect("Failed to add edge");
    assert_eq!(dag.iter_topo(&root).map(|handle| { handle.node() }).collect::<Vec<u32>>(), vec![12, 1, 2]);
}

#[test]
/// After adding, then deleteing edges, nodes shouldn't appear in topo_sort
fn test_rm_edges() {
    //     12
    //     v
    //     1
    //     v
    //     2
    let mut dag = MyDag::new();
    let root = dag.add_node(12);
    let n2 = dag.add_node(2);
    let n1 = dag.add_node(1);
    dag.add_edge(&root, &n1, 1001).expect("Failed to add edge");
    dag.add_edge(&n1, &n2, 1002).expect("Failed to add edge");
    assert_eq!(dag.iter_topo(&root).map(|handle| { handle.node() }).collect::<Vec<u32>>(), vec![12, 1, 2]);
    // rm link to 2.
    dag.rm_edge(&n1, &n2, 1002).expect("Failed to rm edge");
    assert_eq!(dag.iter_topo(&root).map(|handle| { handle.node() }).collect::<Vec<u32>>(), vec![12, 1]);
    // add link back & remove link to 1.
    dag.add_edge(&n1, &n2, 1002).expect("Failed to add edge");
    dag.rm_edge(&root, &n1, 1001).expect("Failed to rm edge");
    assert_eq!(dag.iter_topo(&root).map(|handle| { handle.node() }).collect::<Vec<u32>>(), vec![12]);
}

#[test]
/// Graph should not allow cycles - should be an error when adding a cycle & structure should be
/// unmodified.
fn test_cycles() {
    //     12
    //     v
    //     1
    //     v
    //     2
    let mut dag = MyDag::new();
    let root = dag.add_node(12);
    let n2 = dag.add_node(2);
    let n1 = dag.add_node(1);
    dag.add_edge(&root, &n1, 1001).expect("Failed to add edge");
    dag.add_edge(&n1, &n2, 1002).expect("Failed to add edge");
    dag.add_edge(&n2, &root, 1003).err().expect("Failed to detect cycle");
}
