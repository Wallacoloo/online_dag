use ::ondag::OnDag;
use ::rcdag::RcDag;

type MyDag = RcDag<u32, u32>;


def_ondag_tests!{MyDag}



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
