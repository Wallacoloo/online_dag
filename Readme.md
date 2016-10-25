About
======
This library provides a DAG structure that enforces its acyclic property with each insertion, returning `Ok()` on a successful insertion and `Err()` if the insertion would create a cycle (upon error, the graph is left in the same state as before the insertion).

Additionally, nodes (and associated edges) are automatically dropped when unreachable (not reachable from the graph root or from any user-owned `NodeHandle`s).
