About
======
This library provides a DAG structure that enforces its acyclic property with each insertion, returning `Ok()` on a successful insertion and `Err()` if the insertion would create a cycle (upon error, the graph is left in the same state as before the insertion).

Additionally, nodes (and associated edges) are automatically dropped when unreachable (not reachable from the graph root or from any user-owned `NodeHandle`s).

Design Goals
======
The DAG structure is designed to be as simple as possible. This often comes at the cost of performance. For example, a node's children set currently can't be accessed by reference, but instead much be copied on access. Future/alternate versions may attempt to fix this.
