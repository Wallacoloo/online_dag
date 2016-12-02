About
======
This library provides a few different DAG structures that enforce the acyclic property with each insertion, returning `Ok()` on a successful insertion and `Err()` if the insertion would create a cycle (upon error, the graph is left in the same state as before the insertion).

Each DAG implementation is specialized for a different purpose. Refcount-based DAGs (the only type that exists at the moment) automatically drop nodes and associated edges when they are unreachable from any existing `NodeHandle`.

Additionally, different DAGs in this library have different criteria for what, exactly, is considered a cycle, making them suitable even for graphs where cycles are *sometimes* OK.

Design Goals
======
The DAG structure is designed to be as simple as possible. This often comes at the cost of performance. For example, a node's children set currently can't be accessed by reference, but instead much be copied on access. Future/alternate versions may attempt to fix this.
