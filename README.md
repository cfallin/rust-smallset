`smallset`: a small unordered set
=================================

[crates.io](https://crates.io/crates/smallset/)

[Documentation](https://cfallin.github.io/rust-smallset/smallset/)

This crate implements a small unordered-set data structure implemented using
[smallvec](https://crates.io/crates/smallvec/). It stores set elements in a
simple unordered array, and when the set is smaller than a parameterizable
size, the elements are stored completely inline (i.e., with zero heap
allocations). The data structure is thus very space-efficient for sets of only
a few elements, much more so than a tree-based or hash-table-based set data
structure.  It is also fast when the set is small: queries and inserts perform
a linear scan, which is more cache-friendly than a pointer-chasing search
through a tree.

`smallset` should be used where minimizing heap allocations is of primary
importance and where it is expected that no more than a few elements will be
present. If the set grows large, then it will exhibit poor (`O(n)` queries and
inserts) performance.
