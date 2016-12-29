use ::ondag::OnDag;
use ::poscostdag::{CostQueriable, HalfEdge, PosCostDag};

type MyDag = PosCostDag<u32, u32>;


// Declare tests for common OnDag functionality (inserting/removing nodes and edges).
def_ondag_tests!{MyDag}


#[test]
/// Graph should not allow ZERO-COST cycles - should be an error when adding a cycle & structure should be
/// unmodified.
fn test_cycles() {
    // A --4--> B --0--> C
    // Adding a zero-path from C -> A should *not* introduce a 0-cycle
    // Adding above + a zero-path from A->B *should* introduce a 0-cycle
    // Here A, B, C become 11, 12, 13
    let mut dag = MyDag::new();
    let a = dag.add_node(11);
    let b = dag.add_node(12);
    let c = dag.add_node(13);
    dag.add_edge(&a, &b, 4).expect("Failed to add edge");
    dag.add_edge(&b, &c, 0).expect("Failed to add edge");
    dag.add_edge(&c, &a, 0).expect("Failed to add edge");
    dag.add_edge(&a, &b, 0).err().expect("Failed to detect cycle");
}

impl CostQueriable<u32, u32> for u32 {
    /// For testing, the edge cost is identical to its weight.
    fn is_zero_cost(edge: &HalfEdge<u32, u32>, _next: &HalfEdge<u32, u32>,_dag: &MyDag) -> bool {
        edge.weight() == &0
    }
}
